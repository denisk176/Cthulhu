use std::time::Duration;
use tracing::info;
use crate::config::PortConfig;
use crate::switch::{PortStatus, PortUpdate, PortUpdateSender};
use crate::switch::worker::{make_log_writer, RexpectSession};
use crate::switch::worker::logging::ContainedWriter;
use crate::switch::worker::state::{ActionFunction, DeviceInformation, DeviceInformationType, SwitchData};

pub enum Action {
    Send(String),
    Flush,
    SendLine(String),
    SendControl(char),
    Function(ActionFunction),
    Repeat(Vec<Action>, usize),
    Delay(Duration),
    UpdatePortStatus(PortStatus),
    AddDeviceInfo(DeviceInformation),
    FinishJob,
}

impl Action {
    pub fn perform(&self, config: &PortConfig, state: &mut SwitchData, log_writer: ContainedWriter, update_sender: &PortUpdateSender, p: &mut RexpectSession, data: &str, mat: &str) -> color_eyre::Result<()> {
        match self {
            Action::Send(s) => {
                p.send(s)?;
                Ok(())
            }
            Action::Flush => {
                p.flush()?;
                Ok(())
            }
            Action::SendLine(s) => {
                p.send_line(s)?;
                Ok(())
            }
            Action::SendControl(c) => {
                p.send_control(*c)?;
                Ok(())
            }
            Action::Function(cb) => {
                cb(state, p, data, mat)
            }
            Action::FinishJob => {
                info!("Job finished!");
                info!("Information items:");
                for i in state.get_information() {
                    info!(" - {i:?}");
                }

                let new_status = match state.get_max_type() {
                    Some(DeviceInformationType::Info) => PortStatus::IdleSuccess,
                    Some(DeviceInformationType::Warning) => PortStatus::IdleWarning,
                    Some(DeviceInformationType::Error) => PortStatus::IdleError,
                    None => PortStatus::Idle,
                };

                update_sender.send(PortUpdate::PortStatusUpdate(new_status))?;

                *state = SwitchData::default();
                log_writer.replace(make_log_writer(config, state)?);

                update_sender.send(PortUpdate::PortJobStart(state.get_started()))?;
                Ok(())
            }
            Action::Repeat(a, t) => {
                for _ in 0..*t {
                    for b in a.iter() {
                        b.perform(config, state, log_writer.clone(), update_sender, p, data, mat)?;
                    }
                }
                Ok(())
            }
            Action::Delay(d) => {
                std::thread::sleep(*d);
                Ok(())
            }
            Action::UpdatePortStatus(s) => {
                update_sender.send(PortUpdate::PortStatusUpdate(*s))?;
                Ok(())
            }
            Action::AddDeviceInfo(i) => {
                state.add_information(i.clone());
                Ok(())
            },
        }
    }
}
