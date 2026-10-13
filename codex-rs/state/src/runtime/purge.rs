use super::StateRuntime;

impl StateRuntime {
    /// Clear session metadata without removing project registrations or authentication settings.
    /// The caller must stop session writers before calling this, and close the runtime afterward.
    pub async fn purge_session_metadata(&self) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        for statement in [
            "DELETE FROM thread_dynamic_tools",
            "DELETE FROM thread_attachments",
            "DELETE FROM guardian_review_feedback",
            "DELETE FROM thread_spawn_edges",
            "DELETE FROM thread_read_receipts",
            "DELETE FROM threads",
            "DELETE FROM rollout_migration_state",
            "DELETE FROM rollout_migration_skipped_rollouts",
            "UPDATE backfill_state SET status = 'pending', last_watermark = NULL, last_success_at = NULL",
        ] {
            sqlx::query(statement).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteConfig;
    use crate::runtime::test_support::test_thread_metadata;
    use crate::runtime::test_support::unique_temp_dir;
    use codex_protocol::ThreadId;
    use codex_utils_absolute_path::AbsolutePathBuf;

    #[tokio::test]
    async fn purge_session_metadata_preserves_settings_and_projects() -> anyhow::Result<()> {
        let home = unique_temp_dir();
        let sqlite = SqliteConfig::new_for_testing(AbsolutePathBuf::from_absolute_path(&home)?);
        let state = StateRuntime::init(sqlite, "test".to_string()).await?;
        let id = ThreadId::new();
        state
            .upsert_thread(&test_thread_metadata(&home, id, home.clone()))
            .await?;
        sqlx::query("INSERT INTO projects (id, name, position, created_at_ms, updated_at_ms) VALUES ('project', 'keep', 0, 0, 0)")
            .execute(state.pool.as_ref()).await?;
        sqlx::query("INSERT INTO remote_control_enrollments (websocket_url, account_id, app_server_client_name, server_id, environment_id, server_name, updated_at) VALUES ('ws://localhost', 'account', 'client', 'server', 'environment', 'keep', 0)")
            .execute(state.pool.as_ref()).await?;
        sqlx::query(
            "INSERT INTO thread_read_receipts (thread_id, revision) VALUES (?, 'revision')",
        )
        .bind(id.to_string())
        .execute(state.pool.as_ref())
        .await?;
        sqlx::query("INSERT INTO thread_dynamic_tools (thread_id, position, name, description, input_schema) VALUES (?, 0, 'tool', 'private', '{}')")
            .bind(id.to_string()).execute(state.pool.as_ref()).await?;
        sqlx::query("INSERT INTO thread_attachments (id, thread_id, attachment_type, identity_key, payload, created_at) VALUES ('attachment', ?, 'test', 'key', 'private', 0)")
            .bind(id.to_string()).execute(state.pool.as_ref()).await?;
        sqlx::query("INSERT INTO guardian_review_feedback (id, thread_id, record) VALUES ('feedback', ?, x'01')")
            .bind(id.to_string()).execute(state.pool.as_ref()).await?;
        sqlx::query("INSERT INTO thread_spawn_edges (parent_thread_id, child_thread_id, status) VALUES (?, ?, 'closed')")
            .bind(id.to_string()).bind(ThreadId::new().to_string()).execute(state.pool.as_ref()).await?;
        state.purge_session_metadata().await?;
        state.purge_session_metadata().await?;
        assert!(state.get_thread(id).await?.is_none());
        for query in [
            "SELECT COUNT(*) FROM threads",
            "SELECT COUNT(*) FROM thread_read_receipts",
            "SELECT COUNT(*) FROM thread_spawn_edges",
            "SELECT COUNT(*) FROM thread_attachments",
            "SELECT COUNT(*) FROM guardian_review_feedback",
        ] {
            let count: i64 = sqlx::query_scalar(query)
                .fetch_one(state.pool.as_ref())
                .await?;
            assert_eq!(count, 0, "{query}");
        }
        let sections: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thread_sections")
            .fetch_one(state.pool.as_ref())
            .await?;
        assert!(sections > 0, "section settings must survive");
        for query in [
            "SELECT COUNT(*) FROM projects",
            "SELECT COUNT(*) FROM remote_control_enrollments",
        ] {
            let count: i64 = sqlx::query_scalar(query)
                .fetch_one(state.pool.as_ref())
                .await?;
            assert_eq!(count, 1, "{query} settings must survive");
        }
        state.close().await;
        Ok(())
    }
}
