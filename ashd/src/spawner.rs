use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::channel::WorkflowEvent;

pub async fn spawn_and_relay(
    event: &WorkflowEvent,
    ws_listen: &str,
) -> std::io::Result<tokio::process::Child> {
    if !event.workflow.path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("workflow '{}' not found", event.workflow.path.display()),
        ));
    }

    let ws_url = if ws_listen.starts_with('/') || ws_listen.starts_with('.') {
        let encoded = ws_listen.trim_start_matches('.').replace('/', "%2F");
        format!("ws+unix://{}", encoded)
    } else {
        format!("ws://{}", ws_listen)
    };

    let mut cmd = Command::new("ash");
    cmd.arg(&event.workflow.path);
    for flag in &event.workflow.flags {
        cmd.arg(flag);
    }

    cmd.env("ASH_EVENT_SOURCE", &event.source_name);
    cmd.env("ASH_EVENT_PATH", &event.path);
    cmd.env("ASH_EVENT_ID", &event.event_id);
    cmd.env("ASH_EVENT_TIMESTAMP", &event.timestamp.to_rfc3339());
    cmd.env("ASHD_WS_URL", &ws_url);
    cmd.env("PATH", std::env::var("PATH").unwrap_or_default());
    cmd.env("HOME", std::env::var("HOME").unwrap_or_default());
    if let Ok(ll) = std::env::var("ASH_LOG") {
        cmd.env("ASH_LOG", ll);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            log::error!("event_id":% = event.event_id, "source":% = event.source_name, "workflow":% = event.workflow.path.to_string_lossy(), "error":% = e; "failed to spawn ash");
            return Err(e);
        }
    };

    let pid = child.id().unwrap_or(0);
    log::info!("event_id":% = event.event_id, "source":% = event.source_name, "workflow":% = event.workflow.path.to_string_lossy(), "pid":% = pid; "spawned workflow");

    let stdout = child.stdout.take().expect("stdout not piped");
    let stderr = child.stderr.take().expect("stderr not piped");

    let prefix = format!("[{}/{}]", event.source_name, event.event_id);
    let p = prefix.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            println!("{} {}", p, line);
        }
    });

    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            eprintln!("{} {}", prefix, line);
        }
    });

    Ok(child)
}
