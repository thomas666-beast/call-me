use crate::config::Config;
use anyhow::{Context, Result};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

pub async fn run(cfg: Config, tx: broadcast::Sender<serde_json::Value>) -> Result<()> {
    let mut backoff = Duration::from_secs(1);

    loop {
        tracing::info!("Connecting to AMI at {}:{}", cfg.ami_host, cfg.ami_port);
        match connect_and_stream(&cfg, &tx).await {
            Ok(_) => {
                tracing::warn!("AMI connection closed, reconnecting...");
                backoff = Duration::from_secs(1);
            }
            Err(e) => {
                tracing::error!("AMI connection failed: {:#}", e);
            }
        }
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(30));
    }
}

async fn connect_and_stream(
    cfg: &Config,
    tx: &broadcast::Sender<serde_json::Value>,
) -> Result<()> {
    let stream = TcpStream::connect((cfg.ami_host.as_str(), cfg.ami_port))
        .await
        .context("TCP connect failed")?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login = format!(
        "Action: Login\r\nUsername: {}\r\nSecret: {}\r\nEvents: on\r\n\r\n",
        cfg.ami_user, cfg.ami_pass
    );
    writer.write_all(login.as_bytes()).await?;

    let response = read_block(&mut reader).await?;
    tracing::info!("AMI login response: {:?}", response);
    if !response.contains("Success") {
        anyhow::bail!("AMI login failed: {}", response);
    }

    let mut block = String::new();
    loop {
        block.clear();
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                anyhow::bail!("AMI connection EOF");
            }
            if line == "\r\n" || line == "\n" {
                break;
            }
            block.push_str(&line);
        }

        if block.is_empty() {
            continue;
        }

        let mut obj = serde_json::Map::new();
        for l in block.lines() {
            if let Some((k, v)) = l.split_once(':') {
                obj.insert(
                    k.trim().to_string(),
                    serde_json::Value::String(v.trim().to_string()),
                );
            }
        }

        let json = serde_json::Value::Object(obj);
        let _ = tx.send(json);
    }
}

async fn read_block<R: tokio::io::AsyncBufRead + Unpin>(reader: &mut R) -> Result<String> {
    let mut block = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            anyhow::bail!("EOF during read_block");
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        block.push_str(&line);
    }
    Ok(block)
}
