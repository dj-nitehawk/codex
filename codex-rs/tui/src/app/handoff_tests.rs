use super::*;
use crate::app::session_lifecycle::ThreadAttachPresentation;
use crate::app::tests::make_test_app_with_channels;
use codex_app_server_protocol::ServerNotification;
use codex_model_provider_info::ModelProviderInfo;
use core_test_support::responses;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::streaming_sse::StreamingSseChunk;
use core_test_support::streaming_sse::start_streaming_sse_server;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;

fn user_item(text: &str) -> ThreadItem {
    ThreadItem::UserMessage {
        id: text.to_string(),
        client_id: None,
        content: vec![UserInput::Text {
            text: text.to_string(),
            text_elements: Vec::new(),
        }],
    }
}

fn pending(app: &App, thread_id: ThreadId) -> PendingHandoff {
    PendingHandoff {
        thread_id,
        request_id: Uuid::new_v4(),
        task: "finish parser".into(),
        model: app.chat_widget.current_model().into(),
        cwd: app.chat_widget.config_ref().cwd.display().to_string(),
        completed_turns: app.recap.progress().completed_turns,
        cancellation: CancellationToken::new(),
    }
}

#[test]
fn handoff_history_keeps_older_exchanges_and_tool_evidence_without_reasoning() {
    let mut blocks = Vec::new();
    let mut bytes = 0;
    let tool: ThreadItem = serde_json::from_value(json!({
        "type": "functionCallOutput", "id": "tool", "name": "exec",
        "namespace": null, "output": "cargo test: 12 passed",
    }))
    .unwrap();
    assert!(push_history_item(&mut blocks, &mut bytes, &tool));
    for i in (0..20).rev() {
        assert!(push_history_item(
            &mut blocks,
            &mut bytes,
            &user_item(&format!("request {i}"))
        ));
    }
    assert!(push_history_item(
        &mut blocks,
        &mut bytes,
        &ThreadItem::Reasoning {
            id: "private".into(),
            summary: vec!["private reasoning".into()],
            content: Vec::new(),
        }
    ));
    let history = finish_history(blocks, false);
    assert!(history.starts_with("User: request 0"));
    assert!(history.contains("User: request 19"));
    assert!(history.contains("cargo test: 12 passed"));
    assert!(!history.contains("private reasoning"));
}

#[test]
fn handoff_history_and_unicode_excerpts_are_bounded() {
    let text = format!("BEGIN{}END", "🦀".repeat(10_000));
    let block = excerpt(&text, ITEM_MAX_BYTES);
    assert!(block.len() <= ITEM_MAX_BYTES);
    assert!(block.starts_with("BEGIN"));
    assert!(block.ends_with("END"));
    let mut blocks = Vec::new();
    let mut bytes = 0;
    while push_history_item(&mut blocks, &mut bytes, &user_item(&text)) {}
    let history = finish_history(blocks, true);
    assert!(history.len() <= HISTORY_MAX_BYTES);
    assert!(history.starts_with(OMITTED));
}

#[test]
fn handoff_prompt_and_seed_separate_the_task_from_context() {
    let prompt = handoff_prompt("finish parser", "User: use src/parser.rs");
    assert!(prompt.contains("Retain ONLY concrete facts relevant to the task"));
    assert!(prompt.contains("untrusted reference data"));
    let parent = ThreadId::new();
    let seed = seed_prompt(parent, "src/parser.rs, 12 tests passed", "finish parser");
    assert!(seed.contains(&format!("Source session: {parent}")));
    assert!(seed.ends_with("## Task\nfinish parser"));
    assert_eq!(handoff_output_schema()["required"], json!(["note"]));
}

#[test]
fn handoff_notes_reject_empty_invalid_and_oversized_responses() {
    assert_eq!(
        parse_handoff_note(r#"{"note":"  Relevant facts  "}"#).unwrap(),
        "Relevant facts"
    );
    for response in [
        "not json",
        r#"{"note":" "}"#,
        r#"{"summary":"wrong field"}"#,
    ] {
        assert!(parse_handoff_note(response).is_err());
    }
    assert!(
        parse_handoff_note(&json!({"note": "x".repeat(NOTE_MAX_BYTES + 1)}).to_string()).is_err()
    );
}

#[test]
fn handoff_history_omits_image_payloads_and_empty_snapshots() {
    assert!(finish_history(Vec::new(), true).is_empty());
    let mut value = json!({
        "content": [
            {"type": "image", "data": "private-image-data", "mimeType": "image/png"},
            {"type": "text", "text": "text evidence"},
        ],
        "image_url": "data:image/png;base64,secret-image-data",
    });
    omit_image_payloads(&mut value);
    assert!(!value.to_string().contains("private-image-data"));
    assert!(!value.to_string().contains("secret-image-data"));
    assert!(value.to_string().contains("text evidence"));
}

#[tokio::test]
async fn handoff_empty_history_does_not_create_a_destination() -> color_eyre::Result<()> {
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let started = server.start_thread(&app.config).await?;
    let parent = started.session.thread_id;
    app.active_thread_id = Some(parent);
    app.request_handoff(&server, parent, "finish parser".into());
    assert!(app.handoff.is_some());
    let started = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Some(event @ AppEvent::HandoffStarted { .. }) = rx.recv().await {
                break event;
            }
        }
    })
    .await?;
    assert!(
        matches!(&started, AppEvent::HandoffStarted { result: Err(error), .. }
        if error.contains("No conversation")),
        "Unexpected empty-history response: {started:?}",
    );
    let mut tui = crate::tui::test_support::make_test_tui()?;
    app.handle_event(&mut tui, &mut server, started).await?;
    assert!(app.handoff.is_none());
    assert_eq!(app.active_thread_id, Some(parent));
    server.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn handoff_freshness_and_cancellation_preserve_the_parent() {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    let parent = ThreadId::new();
    app.active_thread_id = Some(parent);
    let request = pending(&app, parent);
    let id = request.request_id;
    let token = request.cancellation.clone();
    app.handoff = Some(request);
    assert!(app.handoff_is_current(id));
    assert!(!app.handoff_is_current(Uuid::new_v4()));
    app.active_thread_id = Some(ThreadId::new());
    assert!(!app.handoff_is_current(id));
    app.active_thread_id = Some(parent);
    app.recap.note_turn_finished(
        &codex_app_server_protocol::TurnStatus::Completed,
        std::time::Instant::now(),
    );
    assert!(!app.handoff_is_current(id));
    app.cancel_handoff();
    assert!(token.is_cancelled());
    assert!(app.handoff.is_none());
    assert_eq!(app.active_thread_id, Some(parent));
}

#[tokio::test]
async fn handoff_escape_and_failed_note_do_not_start_a_destination() -> color_eyre::Result<()> {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    let parent = ThreadId::new();
    app.active_thread_id = Some(parent);
    app.handoff = Some(pending(&app, parent));
    let token = app.handoff.as_ref().unwrap().cancellation.clone();
    let cancelled_id = app.handoff.as_ref().unwrap().request_id;
    app.handle_key_event(
        &mut tui,
        &mut server,
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
    )
    .await;
    assert!(token.is_cancelled());
    assert!(app.handoff.is_none());
    assert_eq!(app.active_thread_id, Some(parent));

    app.handoff = Some(pending(&app, parent));
    let id = app.handoff.as_ref().unwrap().request_id;
    app.handle_handoff_generated(
        &mut tui,
        &mut server,
        cancelled_id,
        Ok(r#"{"note":"stale"}"#.into()),
    )
    .await;
    assert_eq!(app.handoff.as_ref().unwrap().request_id, id);
    assert_eq!(app.active_thread_id, Some(parent));
    app.handle_handoff_generated(&mut tui, &mut server, id, Ok(r#"{"note":" "}"#.into()))
        .await;
    assert!(app.handoff.is_none());
    assert_eq!(app.active_thread_id, Some(parent));
    server.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn handoff_preserves_a_new_draft_while_the_note_was_generating() -> color_eyre::Result<()> {
    let (mut app, _rx, _op_rx) = make_test_app_with_channels().await;
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    let parent = ThreadId::new();
    app.active_thread_id = Some(parent);
    app.handoff = Some(pending(&app, parent));
    let id = app.handoff.as_ref().unwrap().request_id;
    app.chat_widget
        .restore_user_message_to_composer("new draft".to_string().into());
    app.handle_handoff_generated(
        &mut tui,
        &mut server,
        id,
        Ok(r#"{"note":"valid note"}"#.into()),
    )
    .await;
    assert!(app.handoff.is_none());
    assert_eq!(app.active_thread_id, Some(parent));
    assert!(app.chat_widget.has_unsent_composer_input());
    assert_eq!(app.chat_widget.composer_text_with_pending(), "new draft");
    server.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn handoff_generates_context_and_seeds_a_fresh_resumable_session() -> color_eyre::Result<()> {
    const PARENT_TURNS: usize = 30;
    let streams = std::iter::repeat_n("Twelve tests passed.", PARENT_TURNS).chain([
        r#"{"note":"Relevant context: src/parser.rs uses strict empty-input validation. Twelve tests passed."}"#,
        "Continuing the parser task.",
        r#"{"title":"Finish parser"}"#,
    ]).enumerate().map(|(i, text)| {
        [ev_response_created(&format!("response-{i}")), ev_assistant_message(&format!("message-{i}"), text),
            ev_completed(&format!("response-{i}"))].into_iter().map(|event| StreamingSseChunk {
                gate: None, body: responses::sse(vec![event]),
            }).collect()
    }).collect();
    let (model_server, _completions) = start_streaming_sse_server(streams).await;
    let (mut app, mut rx, _op_rx) = make_test_app_with_channels().await;
    let home = tempfile::tempdir()?;
    let model = "gpt-5.1";
    let provider = "handoff-test";
    std::fs::write(
        home.path().join("config.toml"),
        format!(
            r#"
model = "{model}"
model_provider = "{provider}"
[model_providers.{provider}]
name = "Handoff test"
base_url = "{}/v1"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0
"#,
            model_server.uri()
        ),
    )?;
    app.config.codex_home = codex_utils_absolute_path::AbsolutePathBuf::try_from(home.path())?;
    app.config.sqlite = codex_state::SqliteConfig::new_for_testing(app.config.codex_home.clone());
    app.config.model = Some(model.into());
    app.config.model_provider_id = provider.into();
    app.config.model_provider = ModelProviderInfo {
        name: "Handoff test".into(),
        base_url: Some(format!("{}/v1", model_server.uri())),
        request_max_retries: Some(0),
        stream_max_retries: Some(0),
        ..ModelProviderInfo::default()
    };
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let mut started = server.start_thread(&app.config).await?;
    let parent = started.session.thread_id;
    server
        .thread_set_name(parent, "Parent session".into())
        .await?;
    started.session.thread_name = Some("Parent session".into());
    for i in 0..PARENT_TURNS {
        let text = if i == 0 {
            "Use strict empty-input validation in src/parser.rs".into()
        } else {
            format!("unrelated history {i}")
        };
        let response: codex_app_server_protocol::TurnStartResponse = server
            .request_handle()
            .request_typed(ClientRequest::TurnStart {
                request_id: RequestId::String(format!("parent-turn-{i}")),
                params: codex_app_server_protocol::TurnStartParams {
                    thread_id: parent.to_string(),
                    input: vec![UserInput::Text {
                        text,
                        text_elements: Vec::new(),
                    }],
                    ..Default::default()
                },
            })
            .await?;
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Some(codex_app_server_client::AppServerEvent::ServerNotification(
                    notification,
                )) = server.next_event().await
                    && matches!(notification.as_ref(), ServerNotification::TurnCompleted(done)
                        if done.turn.id == response.turn.id)
                {
                    break;
                }
            }
        })
        .await?;
    }
    let before = read_handoff_history(&server.request_handle(), parent)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e))?;
    assert!(before.contains("strict empty-input validation"));
    assert!(before.contains("Twelve tests passed"));
    let parent_config = app.config.clone();
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
    while let Ok(event) = rx.try_recv() {
        app.handle_event(&mut tui, &mut server, event).await?;
    }
    app.handle_event(
        &mut tui,
        &mut server,
        AppEvent::GenerateHandoff {
            thread_id: parent,
            task: "finish parser".into(),
        },
    )
    .await?;
    let id = app.handoff.as_ref().unwrap().request_id;
    app.handle_event(
        &mut tui,
        &mut server,
        AppEvent::GenerateHandoff {
            thread_id: parent,
            task: "duplicate".into(),
        },
    )
    .await?;
    assert_eq!(app.handoff.as_ref().unwrap().request_id, id);

    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            tokio::select! {
                event = rx.recv() => {
                    app.handle_event(&mut tui, &mut server, event.expect("app event stream")).await?;
                }
                event = server.next_event() => {
                    app.handle_app_server_event(&server, event.expect("server event stream")).await;
                }
                event = async {
                    match app.active_thread_rx.as_mut() {
                        Some(receiver) => receiver.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    let event = event.expect("thread event stream");
                    let child_completed = matches!(&event,
                        crate::app::thread_events::ThreadBufferedEvent::Notification(notification)
                            if matches!(notification.as_ref(), ServerNotification::TurnCompleted(done)
                                if done.thread_id != parent.to_string()
                                    && app.active_thread_id.is_some_and(|id| id.to_string() == done.thread_id))
                    );
                    app.handle_active_thread_event(&mut tui, &mut server, event).await?;
                    if child_completed { break; }
                }
            }
        }
        Ok::<_, color_eyre::Report>(())
    }).await??;
    let handoff_cell = app
        .transcript_cells
        .iter()
        .find(|cell| {
            cell.as_any()
                .downcast_ref::<crate::history_cell::UserHistoryCell>()
                .is_some_and(|cell| cell.message.contains("# Handoff context"))
        })
        .expect("rendered handoff seed");
    for width in [24, 80, 160] {
        let rendered = handoff_cell
            .display_lines(width)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            rendered.contains("finish parser"),
            "handoff seed at width {width}"
        );
    }
    let child = app.active_thread_id.expect("destination thread");
    assert_ne!(child, parent);
    assert!(app.handoff.is_none());
    assert_eq!(app.config.cwd, parent_config.cwd);
    assert_eq!(app.config.model, parent_config.model);
    assert_eq!(
        app.config.model_provider_id,
        parent_config.model_provider_id
    );
    assert_eq!(
        app.config.legacy_sandbox_policy(),
        parent_config.legacy_sandbox_policy()
    );
    assert_eq!(
        app.config.permissions.approval_policy.get(),
        parent_config.permissions.approval_policy.get()
    );
    let after = read_handoff_history(&server.request_handle(), parent)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e))?;
    assert_eq!(before, after);
    let destination = server.thread_read(child, false).await?;
    assert!(destination.forked_from_id.is_none());
    assert!(destination.path.is_some());
    let requests = model_server.requests().await;
    let note_request: Value = serde_json::from_slice(&requests[PARENT_TURNS])?;
    assert_eq!(note_request["tools"], json!([]));
    let note_prompt = request_user_text(&note_request);
    assert!(note_prompt.contains("strict empty-input validation"));
    assert!(note_prompt.contains("Twelve tests passed"));
    let child_request: Value = serde_json::from_slice(&requests[PARENT_TURNS + 1])?;
    let seed = request_user_text(&child_request);
    assert!(seed.contains("# Handoff context"));
    assert!(seed.contains(&parent.to_string()));
    assert!(seed.ends_with("## Task\nfinish parser"));
    assert!(!seed.contains("unrelated history"));

    server.thread_unsubscribe(child).await?;
    let resumed = server
        .resume_thread(
            &app.local_settings,
            app.config.clone(),
            child,
            crate::app_server_session::ResumeModelSettings::RestoreFromThread,
        )
        .await?;
    assert!(
        resumed
            .turns
            .iter()
            .flat_map(|turn| &turn.items)
            .any(|item| {
                matches!(item, ThreadItem::UserMessage { content, .. }
            if content.iter().any(|input| matches!(input, UserInput::Text { text, .. }
                if text.contains("# Handoff context") && text.ends_with("## Task\nfinish parser"))))
            })
    );
    server.shutdown().await?;
    model_server.shutdown().await;
    Ok(())
}

fn request_user_text(request: &Value) -> String {
    request["input"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["role"] == "user")
        .flat_map(|item| item["content"].as_array().unwrap())
        .filter_map(|item| item["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
