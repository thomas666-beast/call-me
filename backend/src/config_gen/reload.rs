use crate::ami;
use crate::config::Config;
use crate::AppState;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

/// Which AMI command to run after writing the config file.
#[derive(Debug, Clone, Copy)]
pub enum ReloadCommand {
    /// `module reload res_pjsip.so` via AMI Command action.
    Pjsip,
    /// `QueueReload` AMI action.
    Queues,
}

/// Result of a reload attempt.
#[derive(Debug)]
pub enum ReloadOutcome {
    /// File written, reload succeeded.
    Applied,
    /// File written, reload failed, rollback succeeded.
    RolledBack { error: String },
    /// File written, reload failed, rollback also failed. Manual intervention required.
    Critical { reload_error: String, rollback_error: String },
}

/// Write `new_content` to `path`, reload Asterisk, roll back on failure.
///
/// Steps:
///   1. Write new content to `<path>.tmp`.
///   2. Copy current `<path>` to `<path>.bak` (if it exists).
///   3. Rename `<path>.tmp` over `<path>`.
///   4. Run the reload via AMI.
///   5. On success: delete `<path>.bak`. Return `Applied`.
///   6. On failure: rename `<path>.bak` over `<path>`, reload again.
///      - If rollback reload succeeds: return `RolledBack`.
///      - If rollback reload also fails: return `Critical`.
pub async fn safe_reload(
    state: &AppState,
    path: &Path,
    new_content: &str,
    reload_cmd: ReloadCommand,
) -> Result<ReloadOutcome> {
    let correlation_id = Uuid::new_v4();
    tracing::info!(
        "[reload {}] START file={} cmd={:?}",
        correlation_id,
        path.display(),
        reload_cmd
    );

    let tmp_path = path.with_extension("conf.tmp");
    let bak_path = path.with_extension("conf.bak");

    // Step 1: write new content to tmp.
    fs::write(&tmp_path, new_content.as_bytes())
        .await
        .with_context(|| format!("[reload {}] failed to write tmp {}", correlation_id, tmp_path.display()))?;

    // Step 2: back up current file if it exists.
    let had_backup = path.exists();
    if had_backup {
        fs::copy(path, &bak_path)
            .await
            .with_context(|| format!("[reload {}] failed to back up {}", correlation_id, path.display()))?;
    }

    // Step 3: rename tmp over the target.
    fs::rename(&tmp_path, path)
        .await
        .with_context(|| format!("[reload {}] failed to rename tmp -> {}", correlation_id, path.display()))?;

    // Step 4: reload.
    match run_reload(&state.cfg, reload_cmd).await {
        Ok(output) => {
            tracing::info!("[reload {}] APPLIED\n{}", correlation_id, output);
            if had_backup {
                let _ = fs::remove_file(&bak_path).await;
            }
            Ok(ReloadOutcome::Applied)
        }
        Err(reload_error) => {
            let reload_error = format!("{:#}", reload_error);
            tracing::error!("[reload {}] RELOAD FAILED: {}", correlation_id, reload_error);

            // Step 6: roll back if we had a backup.
            if !had_backup {
                tracing::error!(
                    "[reload {}] no backup available — file left in new state, reload failed",
                    correlation_id
                );
                return Ok(ReloadOutcome::Critical {
                    reload_error,
                    rollback_error: "no backup available".to_string(),
                });
            }

            if let Err(e) = fs::rename(&bak_path, path).await {
                let rollback_error = format!("failed to restore backup: {:#}", e);
                tracing::error!("[reload {}] ROLLBACK FAILED: {}", correlation_id, rollback_error);
                return Ok(ReloadOutcome::Critical {
                    reload_error,
                    rollback_error,
                });
            }
            tracing::warn!("[reload {}] rolled back file, retrying reload", correlation_id);

            match run_reload(&state.cfg, reload_cmd).await {
                Ok(_) => {
                    tracing::warn!("[reload {}] ROLLED BACK — restored previous config", correlation_id);
                    Ok(ReloadOutcome::RolledBack { error: reload_error })
                }
                Err(e) => {
                    let rollback_error = format!("{:#}", e);
                    tracing::error!(
                        "[reload {}] CRITICAL — rollback reload also failed: {}",
                        correlation_id,
                        rollback_error
                    );
                    Ok(ReloadOutcome::Critical {
                        reload_error,
                        rollback_error,
                    })
                }
            }
        }
    }
}

async fn run_reload(cfg: &Config, cmd: ReloadCommand) -> Result<String> {
    match cmd {
        ReloadCommand::Pjsip => ami::actions::run_command(cfg, "module reload res_pjsip.so").await,
        ReloadCommand::Queues => {
            // QueueReload action + dialplan reload (the latter picks up new
            // queue numbers in the _2XXX pattern).
            let queue_out = ami::actions::queue_reload(cfg, false).await?;
            let _ = ami::actions::run_command(cfg, "dialplan reload").await;
            Ok(queue_out)
        }
    }
}
