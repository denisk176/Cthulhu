use crate::AngelJob;
use crate::pfunc::ProcessFunction;
use cthulhu_common::devinfo::{DeviceInformation, DeviceInformationType};
use cthulhu_common::status::{JobUpdate, PortJobStatus};
use std::time::Duration;
use swexpect::SwitchExpect;
use tracing::info;

pub enum Action {
    Send(String),
    Flush,
    SendLine(String),
    SendControl(char),
    Function(ProcessFunction),
    Repeat(Vec<Action>, usize),
    Delay(Duration),
    UpdatePortStatus(PortJobStatus),
    AddDeviceInfo(DeviceInformation),
    FinishJob,
    SetupJob,
}

impl Action {
    pub async fn perform<T: AngelJob>(
        &self,
        job: &mut T,
        p: &mut SwitchExpect,
        data: &str,
        mat: &str,
    ) -> color_eyre::Result<()> {
        match self {
            Action::Send(s) => {
                p.send(s).await?;
                Ok(())
            }
            Action::Flush => {
                p.flush().await?;
                Ok(())
            }
            Action::SendLine(s) => {
                p.send_line(s).await?;
                Ok(())
            }
            Action::SendControl(c) => {
                p.send_control(*c).await?;
                Ok(())
            }
            Action::Function(pf) => pf.execute(job, p, data, mat).await,
            Action::FinishJob => {
                info!("Job finished!");
                info!("Information items:");
                for i in job.get_information() {
                    info!(" - {i:?}");
                }

                let new_status = match job.get_max_information_type() {
                    Some(DeviceInformationType::Info) => PortJobStatus::FinishSuccess,
                    Some(DeviceInformationType::Warning) => PortJobStatus::FinishWarning,
                    Some(DeviceInformationType::Error) => PortJobStatus::FinishError,
                    None => PortJobStatus::Idle,
                };

                job.send_update(JobUpdate::JobStatusUpdate(new_status))
                    .await?;
                //job.reset().await?;
                Ok(())
            }
            Action::Repeat(a, t) => {
                for _ in 0..*t {
                    for b in a.iter() {
                        Box::pin(b.perform(job, p, data, mat)).await?;
                    }
                }
                Ok(())
            }
            Action::Delay(d) => {
                std::thread::sleep(*d);
                Ok(())
            }
            Action::UpdatePortStatus(s) => {
                job.send_update(JobUpdate::JobStatusUpdate(*s)).await?;
                Ok(())
            }
            Action::AddDeviceInfo(i) => {
                job.add_information(i.clone()).await?;
                Ok(())
            }
            Action::SetupJob => {
                job.init_job().await?;
                Ok(())
            }
        }
    }
}
