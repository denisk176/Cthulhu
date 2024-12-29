use std::fmt::Display;
use std::future::Future;
use std::ops::Add;
use std::sync::Arc;
use chrono::{DateTime, TimeDelta, Utc};
use futures::future::join_all;
use futures::FutureExt;
use serde::Serialize;
use tokio::sync::RwLock;
use crate::switch::{PortStatus, PortUpdate, ProcessStage, SpawnedPort};

#[derive(Default, Debug, Serialize, Clone)]
pub struct PortManagerEntry {
    pub label: String,
    pub last_update: DateTime<Utc>,
    pub job_started: DateTime<Utc>,
    pub current_stage: ProcessStage,
    pub status: PortStatus,
}

impl PortStatus {
    pub fn get_css_backgroundcolor(&self) -> String {
        match self {
            PortStatus::Idle => "#ffffff".to_string(),
            PortStatus::IdleSuccess => "#00ff00".to_string(),
            PortStatus::IdleWarning => "#ff9933".to_string(),
            PortStatus::IdleError => "#ff0000".to_string(),
            PortStatus::Busy => "#33bbff".to_string(),
            PortStatus::RunningLong => "#bb33ff".to_string(),
            PortStatus::Fatal => "#ff33dd".to_string(),
        }
    }
}

impl Display for PortStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortStatus::Idle => write!(f, "Idle"),
            PortStatus::IdleSuccess => write!(f, "Idle (Success)"),
            PortStatus::IdleWarning => write!(f, "Idle (Warning"),
            PortStatus::IdleError => write!(f, "Idle (Error)"),
            PortStatus::Busy => write!(f, "Busy"),
            PortStatus::RunningLong => write!(f, "Slow"),
            PortStatus::Fatal => write!(f, "Borked"),
        }
    }
}

#[derive(Clone)]
pub struct PortManager {
    ports: Arc<RwLock<Vec<PortManagerEntry>>>,
}

impl PortManager {
    pub fn new() -> Self {
        Self {
            ports: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_ports(&self) -> Vec<PortManagerEntry> {
        let r = self.ports.read().await;
        r.iter().cloned().map(|mut p| {
            if p.last_update.add(TimeDelta::new(60*15, 0).unwrap()) < Utc::now() && !p.status.is_idle() {
                p.status = PortStatus::RunningLong;
            }
            p
        }).collect()
    }

    pub fn spawn(ports: Vec<SpawnedPort>) -> (Self, impl Future<Output = ()>) {
        let manager = Self::new();
        let manager_copy = manager.clone();
        let jobs = ports.into_iter()
            .map(|p|
                tokio::spawn({
                    let manager = manager.clone();
                    async move {
                    port_update_listener(p, manager).await
                }})
            );
        (manager_copy, join_all(jobs).map(|_| ()))
    }

    pub async fn accept_update(&self, port_label: &str, update_ts: DateTime<Utc>, update: PortUpdate) -> color_eyre::Result<()> {
        let mut ports = self.ports.write().await;
        let existing = if let Some(existing) = ports.iter_mut().find(|p| p.label == port_label) {
            existing
        } else {
            ports.push(PortManagerEntry {
                label: port_label.to_string(),
                ..Default::default()
            });
            ports.last_mut().unwrap()
        };

        existing.last_update = update_ts;
        match update {
            PortUpdate::PortStateTransition(_, new) => {
                existing.current_stage = new;
            }
            PortUpdate::PortStatusUpdate(new) => {
                existing.status = new;
            }
            PortUpdate::PortJobStart(s) => {
                existing.job_started = s;
            }
        }
        Ok(())
    }
}

#[allow(irrefutable_let_patterns)]
async fn port_update_listener(port: SpawnedPort, manager: PortManager) -> color_eyre::Result<()> {
    while let (ts, update) = port.receiver.recv().await? {
        manager.accept_update(&port.config.label, ts, update).await?;
    }
    Ok(())
}
