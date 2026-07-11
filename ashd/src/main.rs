mod channel;
mod config;
mod connection;
mod registry;
mod relay;
mod router;
mod sinks;
mod sources;
mod spawner;
mod ws;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::channel::WorkflowEvent;
use crate::config::{AshdConfig, ConcurrencyMode};
use crate::connection::ConnectionMap;
use crate::registry::ProcessRegistry;
use crate::relay::RelayPipeline;
use crate::router::FrameRouter;
use crate::sources::folder::FolderSource;
use crate::spawner::spawn_and_relay;
use crate::ws::{WsServer, CLOSE_SENTINEL};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse --check flag (simple arg scan, no clap dependency)
    let args: Vec<String> = std::env::args().collect();
    let check = args.iter().any(|a| a == "--check" || a == "-c");

    if check {
        return check_config();
    }

    let config = AshdConfig::load()?;
    log::info!("sources":% = config.sources.len(); "config loaded");

    // Shared components
    let (frame_tx, frame_rx) = mpsc::channel(1024);
    let (event_tx, mut event_rx) = mpsc::channel::<WorkflowEvent>(256);
    let (queue_notify_tx, mut queue_notify_rx) = tokio::sync::broadcast::channel::<String>(64);
    let connection_map = Arc::new(ConnectionMap::new());
    let process_registry = Arc::new(ProcessRegistry::new());
    let relay_pipeline = RelayPipeline::new(config.telemetry_relay.as_ref());

    // Build workflow path -> source name mapping for the router
    let mut wf_to_source = HashMap::new();
    for source in &config.sources {
        for wf in &source.workflows {
            wf_to_source.insert(wf.path.to_string_lossy().to_string(), source.name.clone());
        }
    }
    let workflow_to_source = Arc::new(wf_to_source);

    // Start WebSocket server
    let ws_server = WsServer::new(config.daemon.ws_listen.clone(), frame_tx.clone());
    let ws_handle = tokio::spawn(async move { ws_server.start().await });

    // Start frame router
    let router = FrameRouter::new(
        relay_pipeline.clone(),
        process_registry.clone(),
        connection_map.clone(),
        queue_notify_tx.clone(),
        workflow_to_source.clone(),
    );
    let router_handle = tokio::spawn(async move { router.run(frame_rx).await });

    // Track active child process PIDs for graceful shutdown
    let active_pids: Arc<std::sync::Mutex<HashSet<u32>>> =
        Arc::new(std::sync::Mutex::new(HashSet::new()));

    // Start folder sources
    let mut source_handles = vec![];
    for source_cfg in &config.sources {
        if source_cfg.source_type == "folder" {
            match FolderSource::from_config(source_cfg) {
                Ok(src) => {
                    let tx = event_tx.clone();
                    let handle = tokio::spawn(async move { src.run(tx).await });
                    source_handles.push(handle);
                }
                Err(e) => {
                    log::error!("failed to create folder source '{}': {}", source_cfg.name, e);
                }
            }
        }
    }
    log::info!("count":% = source_handles.len(); "source(s) started");

    // Event loop: receive events and spawn workflows
    let ws_listen = config.daemon.ws_listen.clone();
    let cfg = Arc::new(config);
    let spawn_ws_listen = ws_listen.clone();
    let spawn_cfg = cfg.clone();
    let spawn_pids = active_pids.clone();
    let spawn_handle = tokio::spawn(async move {
        let mut sequential_queues: HashMap<String, VecDeque<WorkflowEvent>> = HashMap::new();

        loop {
            tokio::select! {
                Some(event) = event_rx.recv() => {
                    let source_cfg = spawn_cfg.sources.iter()
                        .find(|s| s.name == event.source_name)
                        .unwrap();

                    if matches!(source_cfg.concurrency, ConcurrencyMode::Sequential)
                        && process_registry.is_source_busy(&event.source_name)
                    {
                        sequential_queues
                            .entry(event.source_name.clone())
                            .or_default()
                            .push_back(event);
                        continue;
                    }

                    spawn_workflow(event, &spawn_cfg, &process_registry, &spawn_ws_listen, &spawn_pids).await;
                    drain_sequential_queues(
                        &mut sequential_queues,
                        &spawn_cfg,
                        &process_registry,
                        &spawn_ws_listen,
                        &spawn_pids,
                    ).await;
                }
                result = queue_notify_rx.recv() => {
                    match result {
                        Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            drain_sequential_queues(
                                &mut sequential_queues,
                                &spawn_cfg,
                                &process_registry,
                                &spawn_ws_listen,
                                &spawn_pids,
                            ).await;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                else => break,
            }
        }
    });

    let listen_addr = ws_listen.clone();
    log::info!("ashd listening on {}", listen_addr);

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    log::info!("shutting down...");

    // ---- Graceful shutdown sequence ----

    // 1. Stop watchers (abort source handles)
    for handle in source_handles {
        handle.abort();
    }
    log::info!("sources stopped");

    // 2. Drop event_tx — no new events accepted
    drop(event_tx);
    log::info!("event channel closed");

    // 3. Drain event channel (brief wait for in-flight events to be consumed)
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 4. Abort spawn handle
    spawn_handle.abort();
    let _ = spawn_handle.await;
    log::info!("event loop stopped");

    // 5. Send SIGTERM to child processes
    {
        let pids = active_pids.lock().unwrap();
        if !pids.is_empty() {
            log::info!("terminating {} child process(es)", pids.len());
            for pid in pids.iter() {
                log::debug!("sending SIGTERM to pid={}", pid);
                let _ = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output();
            }
        }
    }

    // 6. Wait grace_period_secs
    let grace = cfg.daemon.grace_period_secs;
    log::info!("waiting {}s for child processes to exit...", grace);
    tokio::time::sleep(Duration::from_secs(grace)).await;

    // 7. SIGKILL any remaining children
    {
        let pids = active_pids.lock().unwrap();
        if !pids.is_empty() {
            log::warn!("force-killing {} remaining child process(es)", pids.len());
            for pid in pids.iter() {
                let _ = std::process::Command::new("kill")
                    .arg("-KILL")
                    .arg(pid.to_string())
                    .output();
            }
        }
    }

    // 8. Send WS close frames
    connection_map.broadcast(CLOSE_SENTINEL);
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 9. Abort WS + router handles
    ws_handle.abort();
    router_handle.abort();
    let _ = ws_handle.await;
    let _ = router_handle.await;

    // 10. Flush relay pipeline
    drop(relay_pipeline);
    tokio::time::sleep(Duration::from_millis(200)).await;

    log::info!("shutdown complete");
    Ok(())
}

fn check_config() -> anyhow::Result<()> {
    match AshdConfig::load() {
        Ok(config) => {
            println!("Configuration OK");
            println!("  WebSocket: {}", config.daemon.ws_listen);
            println!("  Sources: {}", config.sources.len());
            for src in &config.sources {
                println!("    - {} ({})", src.name, src.source_type);
                if src.source_type == "folder" {
                    if let Some(ref path) = src.path {
                        println!("      path: {}", path.display());
                    }
                }
            }
            if let Some(ref relay) = config.telemetry_relay {
                if let Some(ref kafka) = relay.kafka {
                    if kafka.enabled {
                        println!(
                            "  Kafka relay: {} -> {}",
                            kafka.brokers.join(","),
                            kafka.topic
                        );
                    }
                }
                if let Some(ref splunk) = relay.splunk {
                    if splunk.enabled {
                        println!("  Splunk relay: {}", splunk.endpoint);
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn spawn_workflow(
    event: WorkflowEvent,
    _cfg: &AshdConfig,
    _process_registry: &ProcessRegistry,
    ws_listen: &str,
    active_pids: &Arc<std::sync::Mutex<HashSet<u32>>>,
) {
    match spawn_and_relay(&event, ws_listen).await {
        Ok(mut child) => {
            let pid = child.id().unwrap_or(0);
            {
                let mut pids = active_pids.lock().unwrap();
                pids.insert(pid);
            }
            let event_path = event.path.clone();
            let pids = active_pids.clone();
            tokio::spawn(async move {
                let exit_status = child.wait().await;
                {
                    let mut pids = pids.lock().unwrap();
                    pids.remove(&pid);
                }
                let processing_parent = event_path.parent().and_then(|p| p.parent());
                let success = exit_status.map(|s| s.success()).unwrap_or(false);
                let dest = if success {
                    processing_parent.map(|p| p.join(".done"))
                } else {
                    processing_parent.map(|p| p.join(".done").join("failed"))
                };
                if let Some(dest) = dest {
                    std::fs::create_dir_all(&dest).ok();
                    let dir_name = event_path.file_name().unwrap();
                    if let Err(e) = std::fs::rename(&event_path, dest.join(dir_name)) {
                        log::warn!("failed to move completed event: {}", e);
                    }
                }
            });
        }
        Err(e) => {
            log::error!("failed to spawn workflow for {}: {}", event.event_id, e);
        }
    }
}

async fn drain_sequential_queues(
    queues: &mut HashMap<String, VecDeque<WorkflowEvent>>,
    cfg: &AshdConfig,
    process_registry: &ProcessRegistry,
    ws_listen: &str,
    active_pids: &Arc<std::sync::Mutex<HashSet<u32>>>,
) {
    let source_names: Vec<String> = queues.keys().cloned().collect();
    for source_name in source_names {
        if let Some(queue) = queues.get_mut(&source_name) {
            while queue.front().is_some() {
                if process_registry.is_source_busy(&source_name) {
                    break;
                }
                let event = queue.pop_front().unwrap();
                spawn_workflow(event, cfg, process_registry, ws_listen, active_pids).await;
            }
            if queue.is_empty() {
                queues.remove(&source_name);
            }
        }
    }
}
