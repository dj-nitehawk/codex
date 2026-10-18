use super::*;
use crate::app::session_lifecycle::ThreadAttachPresentation;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::UserInput;
use codex_model_provider_info::ModelProviderInfo;
use core_test_support::responses;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::streaming_sse::StreamingSseChunk;
use core_test_support::streaming_sse::start_streaming_sse_server;
use pretty_assertions::assert_eq;
use std::fs;

#[tokio::test]
async fn new_session_prompt_runs_once_in_fresh_resumable_session() -> Result<()> {
    // Exercise both ordinary startup and the deferred managed-worktree attachment.
    for (prefix, worktree) in [
        ("/new Prompt task ~", false),
        ("/new Prompt task ~", true),
        ("~", false),
        ("~", true),
    ] {
        let streams = ["Done.", r#"{"title":"Prompt task"}"#]
            .into_iter()
            .enumerate()
            .map(|(i, text)| {
                [
                    ev_response_created(&format!("response-{i}")),
                    ev_assistant_message(&format!("message-{i}"), text),
                    ev_completed(&format!("response-{i}")),
                ]
                .into_iter()
                .map(|event| StreamingSseChunk {
                    gate: None,
                    body: responses::sse(vec![event]),
                })
                .collect()
            })
            .collect();
        let (model_server, _completions) = start_streaming_sse_server(streams).await;
        let (mut app, mut events, _ops) = make_test_app_with_channels().await;
        let root = tempdir()?;
        let home = root.path().join("home");
        let source = root.path().join("source");
        fs::create_dir_all(&home)?;
        fs::create_dir_all(&source)?;
        fs::write(source.join("README.md"), "test repository")?;
        for args in [
            vec!["init", "--quiet"],
            vec!["add", "."],
            vec!["commit", "--quiet", "--no-gpg-sign", "-m", "initial"],
        ] {
            let output = std::process::Command::new("git")
                .current_dir(&source)
                .args([
                    "-c",
                    "user.name=Prompt Test",
                    "-c",
                    "user.email=test@example.invalid",
                ])
                .args(args)
                .output()?;
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        fs::write(
            home.join("config.toml"),
            format!(
                r#"
model = "gpt-5.1"
model_provider = "prompt-test"
[model_providers.prompt-test]
name = "Prompt test"
base_url = "{}/v1"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0
"#,
                model_server.uri()
            ),
        )?;
        crate::legacy_core::config::set_project_trust_level(
            &home,
            &source,
            codex_protocol::config_types::TrustLevel::Trusted,
        )
        .map_err(|error| color_eyre::eyre::eyre!(error.to_string()))?;
        app.config.codex_home = home.abs();
        app.config.sqlite = SqliteConfig::new_for_testing(app.config.codex_home.clone());
        app.config.cwd = source.clone().abs();
        app.config.model = Some("gpt-5.1".into());
        app.config.model_provider_id = "prompt-test".into();
        app.config.model_provider = ModelProviderInfo {
            name: "Prompt test".into(),
            base_url: Some(format!("{}/v1", model_server.uri())),
            request_max_retries: Some(0),
            stream_max_retries: Some(0),
            ..ModelProviderInfo::default()
        };
        let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
        let started = server.start_thread(&app.config).await?;
        let parent = started.session.thread_id;
        server.thread_inject_items(parent, vec![serde_json::from_value(serde_json::json!({
            "type":"message", "role":"assistant", "content":[{"type":"output_text", "text":"parent-only history"}]
        }))?]).await?;
        let before = server.thread_read(parent, true).await?;
        let mut tui = crate::tui::test_support::make_test_tui()?;
        app.replace_chat_widget_with_app_server_thread(
            &mut tui,
            started,
            ThreadAttachPresentation::Fresh,
            None,
        )
        .await?;
        while let Some(Ok(event)) = app.active_thread_rx.as_mut().map(|rx| rx.try_recv()) {
            app.handle_active_thread_event(&mut tui, &mut server, event)
                .await?;
        }
        while let Ok(event) = events.try_recv() {
            app.handle_event(&mut tui, &mut server, event).await?;
        }
        let prompt = "/review is literal here\nKeep ~ and ~/paths";
        app.chat_widget.insert_str(&format!("{prefix} {prompt}"));
        app.chat_widget
            .handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        app.chat_widget.handle_key_event(KeyEvent::new(
            KeyCode::Char(if worktree { '2' } else { '1' }),
            KeyModifiers::NONE,
        ));
        while let Ok(event) = events.try_recv() {
            app.handle_event(&mut tui, &mut server, event).await?;
        }
        if worktree {
            complete_managed_worktree_creation(&mut app, &mut tui, &mut server, &mut events)
                .await?;
        }
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                tokio::select! {
                    event = events.recv() => {
                        app.handle_event(&mut tui, &mut server, event.expect("app event")).await?;
                    }
                    event = server.next_event() => {
                        app.handle_backend_event(&server, event.expect("server event")).await;
                    }
                    event = async {
                        match app.active_thread_rx.as_mut() {
                            Some(rx) => rx.recv().await,
                            None => std::future::pending().await,
                        }
                    } => {
                        let event = event.expect("thread event");
                        let completed = matches!(&event,
                            crate::app::thread_events::ThreadBufferedEvent::Notification(notification)
                            if matches!(notification.as_ref(), ServerNotification::TurnCompleted(done)
                                if done.thread_id != parent.to_string()));
                        app.handle_active_thread_event(&mut tui, &mut server, event).await?;
                        if completed { break; }
                    }
                }
            }
            Ok::<_, color_eyre::Report>(())
        }).await??;
        let child = app.active_thread_id.expect("new session");
        assert_ne!(child, parent);
        assert_eq!(app.config.cwd.as_path() != source, worktree);
        let destination = server.thread_read(child, true).await?;
        if prefix.starts_with("/new") {
            assert_eq!(destination.name.as_deref(), Some("Prompt task"));
        }
        assert!(destination.forked_from_id.is_none());
        assert_eq!(server.thread_read(parent, true).await?.turns, before.turns);
        server.thread_unsubscribe(child).await?;
        let resumed = server
            .resume_thread(
                &app.local_settings,
                app.config.clone(),
                child,
                crate::app_server_session::ResumeModelSettings::RestoreFromThread,
            )
            .await?;
        let user_text: Vec<_> = resumed
            .turns
            .iter()
            .flat_map(|turn| &turn.items)
            .filter_map(|item| match item {
                ThreadItem::UserMessage { content, .. } => Some(content),
                _ => None,
            })
            .flatten()
            .filter_map(|input| match input {
                UserInput::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(user_text, vec![prompt]);
        server.shutdown().await?;
        model_server.shutdown().await;
    }
    Ok(())
}

#[tokio::test]
async fn new_session_prompt_restored_when_permission_update_blocks_startup() -> Result<()> {
    let (mut app, _events, _ops) = make_test_app_with_channels().await;
    let (mut server, requests, proxy) = start_recording_app_server(&app.config, None, None).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    app.pending_server_profiles.insert(
        ThreadId::new(),
        PermissionProfileSelection {
            profile_id: "server-only".into(),
            approval_policy: None,
            approvals_reviewer: None,
            display_label: "server-only".into(),
        },
    );
    app.handle_event(
        &mut tui,
        &mut server,
        AppEvent::NewSession {
            name: Some("Blocked".into()),
            initial_prompt: Some("Preserve my prompt".into()),
        },
    )
    .await?;
    assert!(recorded_params(&requests, "thread/start").is_empty());
    assert_eq!(
        app.chat_widget.composer_text_with_pending(),
        "Preserve my prompt"
    );
    server.shutdown().await?;
    proxy.await??;
    Ok(())
}
