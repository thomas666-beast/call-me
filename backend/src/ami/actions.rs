use crate::config::Config;
use anyhow::{Context, Result};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use uuid::Uuid;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);

/// Run an AMI `Command` action on a short-lived connection.
pub async fn run_command(cfg: &Config, command: &str) -> Result<String> {
    let cfg = cfg.clone();
    let command_owned = command.to_string();

    let mut handle = tokio::spawn(async move { run_command_inner(&cfg, &command_owned).await });

    tokio::select! {
        res = &mut handle => match res {
            Ok(inner) => inner,
            Err(join_err) => anyhow::bail!("AMI command task panicked: {}", join_err),
        },
        _ = tokio::time::sleep(COMMAND_TIMEOUT) => {
            handle.abort();
            anyhow::bail!("AMI command '{}' exceeded {:?}", command, COMMAND_TIMEOUT);
        }
    }
}

async fn run_command_inner(cfg: &Config, command: &str) -> Result<String> {
    let stream = TcpStream::connect((cfg.ami_host.as_str(), cfg.ami_port))
        .await
        .context("AMI TCP connect failed")?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login = format!(
        "Action: Login\r\nUsername: {}\r\nSecret: {}\r\nEvents: off\r\n\r\n",
        cfg.ami_user, cfg.ami_pass
    );
    writer.write_all(login.as_bytes()).await?;
    let login_resp = read_block(&mut reader).await?;
    if !login_resp.contains("Success") {
        anyhow::bail!("AMI login failed: {}", login_resp);
    }

    let action_id = Uuid::new_v4().to_string();
    let cmd = format!(
        "Action: Command\r\nActionID: {}\r\nCommand: {}\r\n\r\n",
        action_id, command
    );
    writer.write_all(cmd.as_bytes()).await?;

    let mut output = String::new();
    let first = read_block(&mut reader).await?;
    output.push_str(&first);
    output.push('\n');

    if first.contains("Response: Error") {
        let _ = writer.write_all(b"Action: Logoff\r\n\r\n").await;
        anyhow::bail!("AMI command error: {}", output);
    }

    let looks_complete = first.contains("--END COMMAND--") || first.contains("Output:");
    if !looks_complete {
        for _ in 0..128 {
            let block = read_block(&mut reader).await?;
            output.push_str(&block);
            output.push('\n');
            if block.contains("--END COMMAND--") {
                break;
            }
        }
    }

    let _ = writer.write_all(b"Action: Logoff\r\n\r\n").await;
    Ok(output)
}

/// Send an AMI `QueueReload` action. Reloads queues from queues.conf.
pub async fn queue_reload(cfg: &Config, members: bool) -> Result<String> {
    let cfg = cfg.clone();
    let action = format!(
        "Action: QueueReload\r\nQueue: \r\nMembers: {}\r\nRules: false\r\nParameters: false\r\n\r\n",
        if members { "true" } else { "false" }
    );

    let mut handle = tokio::spawn(async move { run_raw_action(&cfg, &action).await });
    tokio::select! {
        res = &mut handle => match res {
            Ok(inner) => inner,
            Err(e) => anyhow::bail!("QueueReload task panicked: {}", e),
        },
        _ = tokio::time::sleep(COMMAND_TIMEOUT) => {
            handle.abort();
            anyhow::bail!("QueueReload exceeded {:?}", COMMAND_TIMEOUT);
        }
    }
}

/// Send an AMI `QueuePause` action.
///
/// `interface` is the full member interface (e.g. `PJSIP/1002`).
/// `paused` = true to pause, false to unpause.
/// `reason` is optional; stored in our DB but not sent to Asterisk
/// in a way Asterisk stores (Asterisk's Reason field is accepted but
/// only logged).
pub async fn queue_pause(
    cfg: &Config,
    queue: &str,
    interface: &str,
    paused: bool,
    reason: Option<&str>,
) -> Result<String> {
    let cfg = cfg.clone();
    let queue = queue.to_string();
    let interface = interface.to_string();
    let reason_owned = reason.map(|s| s.to_string());

    let action = {
        let mut s = String::new();
        s.push_str("Action: QueuePause\r\n");
        s.push_str(&format!("Queue: {}\r\n", queue));
        s.push_str(&format!("Interface: {}\r\n", interface));
        s.push_str(&format!("Paused: {}\r\n", if paused { "true" } else { "false" }));
        if let Some(r) = &reason_owned {
            s.push_str(&format!("Reason: {}\r\n", r));
        }
        s.push_str("\r\n");
        s
    };

    let mut handle = tokio::spawn(async move { run_raw_action(&cfg, &action).await });
    tokio::select! {
        res = &mut handle => match res {
            Ok(inner) => inner,
            Err(e) => anyhow::bail!("QueuePause task panicked: {}", e),
        },
        _ = tokio::time::sleep(COMMAND_TIMEOUT) => {
            handle.abort();
            anyhow::bail!("QueuePause exceeded {:?}", COMMAND_TIMEOUT);
        }
    }
}

/// Info about one queue member returned by `QueueStatus`.
#[derive(Debug, Clone)]
pub struct QueueMemberInfo {
    pub queue: String,
    pub interface: String,
    pub name: String,
    pub paused: bool,
    pub status: i32,
    pub paused_reason: Option<String>,
}

/// Query all queue members via AMI `QueueStatus`. Returns every member
/// across every queue. Filter by interface to get one agent's memberships.
pub async fn queue_status(cfg: &Config) -> Result<Vec<QueueMemberInfo>> {
    let cfg = cfg.clone();
    let action = "Action: QueueStatus\r\n\r\n".to_string();

    let mut handle = tokio::spawn(async move { run_queue_status(&cfg, &action).await });
    tokio::select! {
        res = &mut handle => match res {
            Ok(inner) => inner,
            Err(e) => anyhow::bail!("QueueStatus task panicked: {}", e),
        },
        _ = tokio::time::sleep(COMMAND_TIMEOUT) => {
            handle.abort();
            anyhow::bail!("QueueStatus exceeded {:?}", COMMAND_TIMEOUT);
        }
    }
}

async fn run_queue_status(cfg: &Config, action_body: &str) -> Result<Vec<QueueMemberInfo>> {
    let stream = TcpStream::connect((cfg.ami_host.as_str(), cfg.ami_port))
        .await
        .context("AMI TCP connect failed")?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login = format!(
        "Action: Login\r\nUsername: {}\r\nSecret: {}\r\nEvents: off\r\n\r\n",
        cfg.ami_user, cfg.ami_pass
    );
    writer.write_all(login.as_bytes()).await?;
    let login_resp = read_block(&mut reader).await?;
    if !login_resp.contains("Success") {
        anyhow::bail!("AMI login failed: {}", login_resp);
    }

    writer.write_all(action_body.as_bytes()).await?;

    let mut members: Vec<QueueMemberInfo> = Vec::new();
    let mut current: Option<QueueMemberInfo> = None;

    for _ in 0..8192 {
        let block = read_block(&mut reader).await?;
        if block.is_empty() {
            continue;
        }
        if block.contains("QueueStatusComplete") {
            break;
        }
        if block.contains("Response: Error") {
            let _ = writer.write_all(b"Action: Logoff\r\n\r\n").await;
            anyhow::bail!("QueueStatus error: {}", block);
        }

        // Determine exact event type from the "Event:" line.
        let event_type = block
            .lines()
            .find_map(|l| l.strip_prefix("Event:").map(|v| v.trim().to_string()));

        match event_type.as_deref() {
            Some("QueueMember") => {
                // Flush any pending member and start a new one.
                if let Some(m) = current.take() {
                    members.push(m);
                }
                let mut m = QueueMemberInfo {
                    queue: String::new(),
                    interface: String::new(),
                    name: String::new(),
                    paused: false,
                    status: 0,
                    paused_reason: None,
                };
                for line in block.lines() {
                    let Some((k, v)) = line.split_once(':') else { continue };
                    let key = k.trim();
                    let val = v.trim();
                    match key {
                        "Queue" => m.queue = val.to_string(),
                        "Location" | "Interface" => m.interface = val.to_string(),
                        "Name" | "MemberName" => m.name = val.to_string(),
                        "Paused" => {
                            m.paused = val == "1"
                                || val.eq_ignore_ascii_case("true")
                                || val.eq_ignore_ascii_case("yes")
                        }
                        "Status" => m.status = val.parse().unwrap_or(0),
                        "PausedReason" | "Reason" => {
                            if !val.is_empty() {
                                m.paused_reason = Some(val.to_string());
                            }
                        }
                        _ => {}
                    }
                }
                current = Some(m);
            }
            Some("QueueMemberPause") | Some("QueueMemberPaused") => {
                // This is an update event for an existing member. Flush
                // any pending member first, then apply the update.
                if let Some(m) = current.take() {
                    members.push(m);
                }

                let mut queue = String::new();
                let mut interface = String::new();
                let mut paused = false;
                let mut paused_reason: Option<String> = None;

                for line in block.lines() {
                    let Some((k, v)) = line.split_once(':') else { continue };
                    let key = k.trim();
                    let val = v.trim();
                    match key {
                        "Queue" => queue = val.to_string(),
                        "Location" | "Interface" => interface = val.to_string(),
                        "Paused" => {
                            paused = val == "1"
                                || val.eq_ignore_ascii_case("true")
                                || val.eq_ignore_ascii_case("yes")
                        }
                        "PausedReason" | "Reason" => {
                            if !val.is_empty() {
                                paused_reason = Some(val.to_string());
                            }
                        }
                        _ => {}
                    }
                }

                // Apply the update to the existing member entry.
                let mut found = false;
                for m in members.iter_mut() {
                    if m.queue == queue && m.interface == interface {
                        m.paused = paused;
                        m.paused_reason = paused_reason.clone();
                        found = true;
                        break;
                    }
                }
                // If no existing entry, create one (shouldn't happen, but safe).
                if !found && !queue.is_empty() {
                    members.push(QueueMemberInfo {
                        queue,
                        interface,
                        name: String::new(),
                        paused,
                        status: 0,
                        paused_reason,
                    });
                }
            }
            _ => {
                // QueueParams, QueueEntry, etc. — ignore.
            }
        }
    }

    if let Some(m) = current.take() {
        members.push(m);
    }

    let _ = writer.write_all(b"Action: Logoff\r\n\r\n").await;

    tracing::info!("QueueStatus parsed: {} members", members.len());
    for m in &members {
        tracing::info!(
            "  member: queue={} interface={} paused={} reason={:?}",
            m.queue,
            m.interface,
            m.paused,
            m.paused_reason
        );
    }

    Ok(members)
}

/// Send a raw AMI action and read the first response block.
async fn run_raw_action(cfg: &Config, action_body: &str) -> Result<String> {
    let stream = TcpStream::connect((cfg.ami_host.as_str(), cfg.ami_port))
        .await
        .context("AMI TCP connect failed")?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login = format!(
        "Action: Login\r\nUsername: {}\r\nSecret: {}\r\nEvents: off\r\n\r\n",
        cfg.ami_user, cfg.ami_pass
    );
    writer.write_all(login.as_bytes()).await?;
    let login_resp = read_block(&mut reader).await?;
    if !login_resp.contains("Success") {
        anyhow::bail!("AMI login failed: {}", login_resp);
    }

    writer.write_all(action_body.as_bytes()).await?;
    let response = read_block(&mut reader).await?;

    let _ = writer.write_all(b"Action: Logoff\r\n\r\n").await;

    if response.contains("Response: Error") {
        anyhow::bail!("AMI action failed: {}", response);
    }

    Ok(response)
}

async fn read_block<R: tokio::io::AsyncBufRead + Unpin>(reader: &mut R) -> Result<String> {
    let mut block = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            anyhow::bail!("EOF while reading AMI block");
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        block.push_str(&line);
    }
    Ok(block)
}
