use crate::devinfo::DeviceInformation;
use crate::stages::ProcessStage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum PortJobStatus {
    /// Initial state
    #[default]
    Idle,
    /// Waiting for new switch; previous success
    FinishSuccess,
    /// Waiting for new switch; previous warning
    FinishWarning,
    /// Waiting for new switch; previous error
    FinishError,
    /// Working on a switch.
    Busy,
    /// This job is taking too long.
    RunningLong,
    /// This thread has crashed.
    Fatal,
}

impl PortJobStatus {
    pub fn is_idle(&self) -> bool {
        match self {
            PortJobStatus::Idle => true,
            PortJobStatus::FinishSuccess => true,
            PortJobStatus::FinishWarning => true,
            PortJobStatus::FinishError => true,
            _ => false,
        }
    }

    pub fn is_finished(&self) -> bool {
        match self {
            PortJobStatus::FinishSuccess => true,
            PortJobStatus::FinishWarning => true,
            PortJobStatus::FinishError => true,
            _ => false,
        }
    }
}

impl Display for PortJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortJobStatus::Idle => write!(f, "😴"),
            PortJobStatus::FinishSuccess => write!(f, "✅"),
            PortJobStatus::FinishWarning => write!(f, "⚠️"),
            PortJobStatus::FinishError => write!(f, "❗"),
            PortJobStatus::Busy => write!(f, "⏳"),
            PortJobStatus::RunningLong => write!(f, "⏰"),
            PortJobStatus::Fatal => write!(f, "😵"),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobUpdate {
    JobStageTransition(ProcessStage, ProcessStage),
    JobStatusUpdate(PortJobStatus),
    JobStart(DateTime<Utc>),
    JobNewInfoItem(DeviceInformation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobCommand {
    ResetJob,
}
