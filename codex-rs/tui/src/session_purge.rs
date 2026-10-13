//! Local session cleanup, run only after the embedded app-server has stopped.

use crate::legacy_core::config::Config;
use codex_state::StateRuntime;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct PurgePaths {
    home: PathBuf,
    log_dir: PathBuf,
    sqlite: codex_state::SqliteConfig,
}

impl PurgePaths {
    pub(crate) fn from_config(config: &Config) -> Self {
        Self {
            home: config.codex_home.to_path_buf(),
            log_dir: config.log_dir.clone(),
            sqlite: config.sqlite_config().clone(),
        }
    }
}

pub(crate) async fn purge_sessions(
    paths: &PurgePaths,
    state: Option<&StateRuntime>,
) -> io::Result<()> {
    // Preserve the primary database: it also holds remote-control settings and projects.
    if let Some(state) = state {
        if state.sqlite() != &paths.sqlite {
            return Err(io::Error::other(
                "session state database path changed; purge cancelled",
            ));
        }
        let result = state.purge_session_metadata().await;
        state.close().await;
        result.map_err(io::Error::other)?;
    } else if paths.sqlite.state_db_path().exists() {
        return Err(io::Error::other(
            "session state database is unavailable; no session files were purged",
        ));
    }

    for database in paths.sqlite.runtime_db_paths() {
        if database.path == paths.sqlite.state_db_path() {
            continue;
        }
        // SQLite WAL and rollback journals may also contain session payloads.
        for suffix in ["", "-wal", "-shm", "-journal"] {
            let mut name = database.path.as_os_str().to_os_string();
            name.push(suffix);
            remove_path(Path::new(&name))?;
        }
    }
    // Historical runtime databases and automatic recovery copies can retain old transcripts.
    match fs::read_dir(paths.sqlite.home()) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                if entry.path() != paths.sqlite.state_db_path()
                    && entry.file_name().to_str().is_some_and(is_runtime_database)
                {
                    // Do not unlink the retained primary database's active sidecars.
                    let primary = paths.sqlite.state_db_path();
                    let primary_name = primary.file_name().unwrap_or_default().to_string_lossy();
                    if !entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(&format!("{primary_name}-"))
                    {
                        remove_path(&entry.path())?;
                    }
                }
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(path_error(paths.sqlite.home(), error)),
    }
    remove_path(&paths.sqlite.home().join("db-backups"))?;
    for name in [
        codex_rollout::SESSIONS_SUBDIR,
        codex_rollout::ARCHIVED_SESSIONS_SUBDIR,
        "history.jsonl",
        "session_index.jsonl",
        "shell_snapshots",
        "memories",
        "visualizations",
        "visualization-viewers",
    ] {
        remove_path(&paths.home.join(name))?;
    }

    // The global tracing subscriber can still hold this file open on Windows.
    // Clear its contents rather than deleting the configured log directory.
    clear_log_file(&paths.log_dir.join(crate::TUI_LOG_FILE_NAME))?;
    match fs::read_dir(&paths.log_dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry?;
                let name = entry.file_name();
                if name.to_str().is_some_and(is_session_recording) {
                    remove_path(&entry.path())?;
                }
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(path_error(&paths.log_dir, error)),
    }
    Ok(())
}

fn is_runtime_database(name: &str) -> bool {
    [
        "state_",
        "logs_",
        "goals_",
        "memories_",
        "memories_v2_",
        "queue_",
        "thread_history_",
    ]
    .iter()
    .any(|prefix| {
        let Some(rest) = name.strip_prefix(prefix) else {
            return false;
        };
        let Some((version, suffix)) = rest.split_once(".sqlite") else {
            return false;
        };
        !version.is_empty()
            && version.bytes().all(|byte| byte.is_ascii_digit())
            && matches!(suffix, "" | "-wal" | "-shm" | "-journal")
    })
}

fn is_session_recording(name: &str) -> bool {
    let Some(timestamp) = name
        .strip_prefix("session-")
        .and_then(|name| name.strip_suffix(".jsonl"))
    else {
        return false;
    };
    let bytes = timestamp.as_bytes();
    bytes.len() == 16
        && bytes[8] == b'T'
        && bytes[15] == b'Z'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 8 || index == 15 || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use tempfile::TempDir;

    fn paths(home: &Path, sqlite_home: &Path) -> PurgePaths {
        PurgePaths {
            home: home.to_path_buf(),
            log_dir: home.join("log"),
            sqlite: codex_state::SqliteConfig::new_for_testing(
                AbsolutePathBuf::from_absolute_path(sqlite_home).unwrap(),
            ),
        }
    }

    #[tokio::test]
    async fn purge_removes_all_session_files_and_preserves_user_files() -> io::Result<()> {
        let home = TempDir::new()?;
        let sqlite = TempDir::new()?;
        let paths = paths(home.path(), sqlite.path());
        for directory in [
            "sessions/2026/10/13",
            "archived_sessions",
            "memories",
            "shell_snapshots",
            "visualizations",
            "visualization-viewers",
        ] {
            fs::create_dir_all(home.path().join(directory))?;
            fs::write(home.path().join(directory).join("payload"), "private")?;
        }
        for file in ["history.jsonl", "session_index.jsonl"] {
            fs::write(home.path().join(file), "private")?;
        }
        fs::create_dir_all(sqlite.path().join("db-backups/sqlite-1-0"))?;
        fs::write(
            sqlite.path().join("db-backups/sqlite-1-0/state_5.sqlite"),
            "private",
        )?;
        fs::write(sqlite.path().join("state_4.sqlite"), "private")?;
        fs::write(sqlite.path().join("state_notes.sqlite"), "keep")?;
        fs::create_dir(home.path().join("log"))?;
        fs::write(paths.log_dir.join(crate::TUI_LOG_FILE_NAME), "private")?;
        fs::write(
            paths.log_dir.join("session-20261013T012345Z.jsonl"),
            "private",
        )?;
        fs::write(paths.log_dir.join("session-not-a-recording.jsonl"), "keep")?;
        for database in paths.sqlite.runtime_db_paths() {
            if database.path == paths.sqlite.state_db_path() {
                continue;
            }
            for suffix in ["", "-wal", "-shm", "-journal"] {
                fs::write(format!("{}{suffix}", database.path.display()), "private")?;
            }
        }
        for file in ["auth.json", "config.toml", "AGENTS.md", "project.txt"] {
            fs::write(home.path().join(file), "keep")?;
        }
        fs::create_dir_all(home.path().join("skills/example"))?;
        fs::write(home.path().join("skills/example/SKILL.md"), "keep")?;
        purge_sessions(&paths, None).await?;
        purge_sessions(&paths, None).await?;
        for file in [
            "sessions",
            "archived_sessions",
            "history.jsonl",
            "session_index.jsonl",
            "memories",
            "shell_snapshots",
            "visualizations",
            "visualization-viewers",
        ] {
            assert!(!home.path().join(file).exists(), "{file} remained");
        }
        for database in paths.sqlite.runtime_db_paths() {
            assert!(!database.path.exists());
        }
        assert!(!sqlite.path().join("db-backups").exists());
        assert!(!sqlite.path().join("state_4.sqlite").exists());
        assert_eq!(
            fs::read_to_string(sqlite.path().join("state_notes.sqlite"))?,
            "keep"
        );
        for file in [
            "auth.json",
            "config.toml",
            "AGENTS.md",
            "project.txt",
            "skills/example/SKILL.md",
            "log/session-not-a-recording.jsonl",
        ] {
            assert_eq!(fs::read_to_string(home.path().join(file))?, "keep");
        }
        assert_eq!(
            fs::read(paths.log_dir.join(crate::TUI_LOG_FILE_NAME))?,
            Vec::<u8>::new()
        );
        assert!(
            !paths
                .log_dir
                .join("session-20261013T012345Z.jsonl")
                .exists()
        );
        Ok(())
    }

    #[tokio::test]
    async fn unavailable_state_database_prevents_file_cleanup() -> io::Result<()> {
        let home = TempDir::new()?;
        let paths = paths(home.path(), home.path());
        fs::write(paths.sqlite.state_db_path(), "unavailable")?;
        fs::write(home.path().join("history.jsonl"), "private")?;
        assert!(purge_sessions(&paths, None).await.is_err());
        assert_eq!(
            fs::read_to_string(home.path().join("history.jsonl"))?,
            "private"
        );
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn purge_does_not_follow_symlinks_outside_session_store() -> io::Result<()> {
        use std::os::unix::fs::symlink;
        let home = TempDir::new()?;
        let outside = TempDir::new()?;
        fs::write(outside.path().join("keep"), "keep")?;
        symlink(outside.path(), home.path().join("sessions"))?;
        fs::create_dir_all(home.path().join("archived_sessions"))?;
        symlink(outside.path(), home.path().join("archived_sessions/link"))?;
        fs::create_dir(home.path().join("log"))?;
        symlink(
            outside.path().join("keep"),
            home.path().join("log/codex-tui.log"),
        )?;
        purge_sessions(&paths(home.path(), home.path()), None).await?;
        assert_eq!(fs::read_to_string(outside.path().join("keep"))?, "keep");
        Ok(())
    }
}

fn remove_path(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(path_error(path, error)),
    };
    // Never traverse a link to a directory outside the session store.
    let result = if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    result.map_err(|error| path_error(path, error))
}

fn clear_log_file(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => remove_path(path),
        Ok(_) => fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .map(|_| ())
            .map_err(|error| path_error(path, error)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(path_error(path, error)),
    }
}

fn path_error(path: &Path, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("failed to purge {}: {error}", path.display()),
    )
}
