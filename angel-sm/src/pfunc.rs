use crate::AngelJob;
use color_eyre::eyre::Context;
use cthulhu_common::devinfo::DeviceInformation;
use regex::RegexBuilder;
use swexpect::SwitchExpect;

pub enum ProcessFunction {
    FixFS,
    CaptureJunosVersion,
    CaptureChassisOutput,
    CaptureAristaVersion,
}

impl ProcessFunction {
    pub async fn execute<T: AngelJob>(
        &self,
        job: &mut T,
        p: &mut SwitchExpect,
        data: &str,
        _mat: &str,
    ) -> color_eyre::Result<()> {
        match self {
            ProcessFunction::FixFS => {
                let bdevregex = RegexBuilder::new(r"ufs: (?<device>[/a-zA-Z0-9]+) \(.*\)$")
                    .crlf(true)
                    .multi_line(true)
                    .build()?;
                let devices: Vec<String> = bdevregex
                    .captures_iter(data)
                    .map(|c| c.name("device").unwrap().as_str().to_string())
                    .collect();

                p.exp_string("#").await.context("failed to fix fs")?;

                for dev in devices.iter() {
                    p.send_line(format!("fsck -y {dev}").as_str()).await?;
                    p.exp_string("#").await.context("failed to fix fs")?;
                }
                p.send_line("reboot").await.context("failed to fix fs")?;
                job.add_information(DeviceInformation::AttemptedToFixFilesystemIssues)
                    .await?;
                Ok(())
            }
            ProcessFunction::CaptureJunosVersion => {
                let r = RegexBuilder::new(r"(?:Model: (?<model>[a-zA-Z0-9\-]+)$)|(?:Junos: (?<version>[0-9a-zA-Z\-\.]+)$)")
                    .multi_line(true).crlf(true).build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(model) = cap.name("model") {
                        job.add_information(DeviceInformation::Model(model.as_str().to_string()))
                            .await?;
                    }
                    if let Some(version) = cap.name("version") {
                        job.add_information(DeviceInformation::SoftwareVersion(
                            version.as_str().to_string(),
                        ))
                        .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureChassisOutput => {
                let r = RegexBuilder::new(r"^Chassis\s+(?<serial>[A-Za-z0-9]+)\s+.*$")
                    .multi_line(true)
                    .crlf(true)
                    .build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(serial) = cap.name("serial") {
                        job.add_information(DeviceInformation::SerialNumber(
                            serial.as_str().to_string(),
                        ))
                        .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureAristaVersion => {
                let r = RegexBuilder::new(r"(?:^Arista (?<model>[a-zA-Z \-0-9]+)$)|(?:^Serial number:\s+(?<serial>[A-Za-z0-9]+)$)|(?:Software image version: (?<version>[0-9\.A-Za-z]+)$)")
                    .multi_line(true).crlf(true).build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(model) = cap.name("model") {
                        job.add_information(DeviceInformation::Model(model.as_str().to_string()))
                            .await?;
                    }
                    if let Some(serial) = cap.name("serial") {
                        job.add_information(DeviceInformation::SerialNumber(
                            serial.as_str().to_string(),
                        ))
                        .await?;
                    }
                    if let Some(version) = cap.name("version") {
                        job.add_information(DeviceInformation::SoftwareVersion(
                            version.as_str().to_string(),
                        ))
                        .await?;
                    }
                }
                Ok(())
            }
        }
    }
}
