use crate::logging::TracingTarget;
use crate::mqtt::MQTTSender;
use chrono::{DateTime, Utc};
use color_eyre::eyre::Context;
use cthulhu_angel_sm::AngelJob;
use cthulhu_angel_sm::process::ProcessStageTransition;
use cthulhu_angel_sm::state::{StateCondition, StateTransition};
use cthulhu_common::devinfo::{DeviceInformation, DeviceInformationType};
use cthulhu_common::stages::ProcessStage;
use cthulhu_common::status::JobUpdate;
use std::path::PathBuf;
use swexpect::SwitchExpect;
use swexpect::hay::ReadUntil;
use tracing::{debug, info};

pub struct ActiveJob {
    information: Vec<DeviceInformation>,
    job_started: DateTime<Utc>,
    pub stage: ProcessStage,
    pub mqtt: MQTTSender,
    tracing_target: TracingTarget,
    rawlog_target: TracingTarget,
    log_dir: Option<PathBuf>,
}

impl AngelJob for ActiveJob {
    async fn init_job(&mut self) -> color_eyre::Result<()> {
        if let Some(log_dir) = self.log_dir.as_ref() {
            {
                let mut tracing_log_file = log_dir.clone();
                tracing_log_file.push(format!(
                    "{}--{}.log",
                    self.job_started.format("%Y-%m-%d--%H:%M:%S"),
                    self.mqtt.id()
                ));
                self.tracing_target.open_file(tracing_log_file)?;
            }
            {
                let mut raw_log_file = log_dir.clone();
                raw_log_file.push(format!(
                    "{}--{}.raw.log",
                    self.job_started.format("%Y-%m-%d--%H:%M:%S"),
                    self.mqtt.id()
                ));
                self.rawlog_target.open_file(raw_log_file)?;
            }
        }
        info!("Job initialized!");
        Ok(())
    }

    async fn send_update(&mut self, update: JobUpdate) -> color_eyre::Result<()> {
        self.mqtt.send_update(update).await?;
        Ok(())
    }

    async fn reset(&mut self) -> color_eyre::Result<()> {
        info!("Resetting job...");
        let old_stage = self.stage;
        self.stage = ProcessStage::default();
        self.information = Vec::new();
        self.job_started = Utc::now();
        self.send_update(JobUpdate::JobStart(self.job_started.clone()))
            .await?;
        self.send_update(JobUpdate::JobStageTransition(old_stage, self.stage))
            .await?;
        Ok(())
    }

    async fn add_information(&mut self, information: DeviceInformation) -> color_eyre::Result<()> {
        info!("Recorded new switch information: {information:?}");
        self.information.push(information.clone());
        self.mqtt
            .send_update(JobUpdate::JobNewInfoItem(information))
            .await?;
        Ok(())
    }

    fn get_information(&self) -> &[DeviceInformation] {
        &self.information
    }

    fn get_max_information_type(&self) -> Option<DeviceInformationType> {
        self.information.iter().map(|i| i.get_type()).max()
    }
}

impl ActiveJob {
    pub fn create(
        mqtt: MQTTSender,
        log_dir: Option<PathBuf>,
        tracing_target: TracingTarget,
        rawlog_target: TracingTarget,
    ) -> Self {
        Self {
            stage: ProcessStage::default(),
            mqtt,
            information: Vec::new(),
            job_started: Utc::now(),
            log_dir,
            tracing_target,
            rawlog_target,
        }
    }

    pub async fn transition(
        &mut self,
        t: &StateTransition,
        p: &mut SwitchExpect,
        d: &str,
        m: &str,
    ) -> color_eyre::Result<()> {
        let old_state = self.stage.clone();
        self.stage = t.target_state;
        info!("State transition: {:?} -> {:?}", old_state, t.target_state);
        self.mqtt
            .send_update(JobUpdate::JobStageTransition(old_state, t.target_state))
            .await?;
        for action in &t.actions {
            action.perform(self, p, d, m).await?;
        }
        Ok(())
    }

    pub async fn step(&mut self, p: &mut SwitchExpect) -> color_eyre::Result<()> {
        let transitions = self.stage.get_transitions()?;

        if let Some(t) = transitions
            .iter()
            .find(|t| t.condition == StateCondition::Immediate)
        {
            self.transition(t, p, "", "")
                .await
                .context("process immediate transition")?;
        } else {
            let u = ReadUntil::Any(
                transitions
                    .iter()
                    .map(|t| t.condition.to_needle().map(|v| v.unwrap()))
                    .collect::<color_eyre::Result<Vec<_>>>()?,
            );

            // Try to handle a result from the switches.
            debug!("Waiting for needle {u:?}...");
            let (d, m) = p
                .expect(&u)
                .await
                .context("failed to read from serial port")?;
            't_test: for t in &transitions {
                if t.condition.matches_result(&m)? {
                    self.transition(&t, p, &d, &m)
                        .await
                        .context("process serial transition")?;
                    break 't_test;
                }
            }
        }

        Ok(())
    }
}
