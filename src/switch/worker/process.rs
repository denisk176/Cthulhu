use std::time::Duration;
use color_eyre::eyre::Context;
use regex::RegexBuilder;
use serde::Serialize;
use crate::switch::PortStatus;
use crate::switch::worker::action::Action;
use crate::switch::worker::state::{DeviceInformation, StateCondition, StateTransition};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default, Serialize)]
pub enum ProcessStage {
    #[default]
    SwitchDetect,

    //JunOS logic
    JunosWaitForBootloader,
    JunosEnterSingleUserMode,
    JunosAwaitBoot,
    JunosAwaitRecoveryShell,
    JunosAnswerZeroize,
    JunosAwaitZeroizeFinish,
    JunosLogin,
    JunosHappyCli,
    JunosBackupImageCli,
    JunosVersionOutput,
    JunosChassisOutput,
    JunosPoweroffConfirm,
    JunosWaitForPoweroff,

    //EX2300C logic
    Junos23WaitForBootloader,
    Junos23Bootloader1,
    Junos23Bootloader2,
    Junos23AwaitRecoveryShell,
    Junos23AnswerZeroize,
    Junos23AwaitZeroizeFinish,
    Junos23AwaitZeroizeFinish2,

    //Arista logic
    AristaWaitForBootloader,
    AristaWipeStartupConfig,
    AristaRebootAfterStartupConfigWipe,
    AristaWaitForReboot,
    AristaLoggingIn,
    AristaVersionOutput,

    //End the process.
    EndJob,
}

impl ProcessStage {
    pub fn get_transitions(&self) -> color_eyre::Result<Vec<StateTransition>> {
        match self {
            ProcessStage::SwitchDetect => Ok(vec![
                //EX3300/EX2200
                StateTransition {
                    target_state: ProcessStage::JunosWaitForBootloader,
                    condition: StateCondition::WaitForString("U-Boot 1.1".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortStatus::Busy),
                    ],
                },
                //EX4400
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("Booting from Flash A".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortStatus::Busy),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("Primary BIOS version CDEN_P_EX1".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortStatus::Busy),
                    ],
                },
                //EX2300C
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("U-Boot 2016".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortStatus::Busy),
                    ],
                },
                //EX4100
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("U-Boot 2021".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortStatus::Busy),
                    ],
                },
                //QFX10k
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("Juniper Linux".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortStatus::Busy),
                    ],
                },
                //Arista
                StateTransition {
                    target_state: ProcessStage::AristaWaitForBootloader,
                    condition: StateCondition::WaitForString("Aboot".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortStatus::Busy),
                    ],
                },
                // This transition is to account for the console server in the bitlair rack.
                StateTransition {
                    target_state: ProcessStage::SwitchDetect,
                    condition: StateCondition::WaitForString("A non-empty Data Buffering File was found.".to_string()),
                    actions: vec![
                        Action::SendLine("E".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosWaitForBootloader => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosEnterSingleUserMode,
                    condition: StateCondition::WaitForString("space bar for command prompt".to_string()),
                    actions: vec![
                        Action::Repeat(vec![
                            Action::Send(" ".to_string()),
                            Action::Flush,
                        ], 10),
                    ]
                },
            ]),
            ProcessStage::JunosEnterSingleUserMode => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosAwaitBoot,
                    condition: StateCondition::WaitForString("loader>".to_string()),
                    actions: vec![
                        Action::Repeat(vec![
                            Action::SendControl('h'),
                            Action::Flush,
                        ], 10),
                        Action::SendLine("boot -s".to_string()),
                    ]
                }
            ]),
            ProcessStage::JunosAwaitBoot => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosAwaitRecoveryShell,
                    condition: StateCondition::WaitForString("Enter full pathname of shell or 'recovery' for root password recovery or RETURN for /bin/sh:".to_string()),
                    actions: vec![
                        Action::SendLine("recovery".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosAwaitRecoveryShell => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosAnswerZeroize,
                    condition: StateCondition::WaitForRegex(r"\{[a-z0-9]+:0\}".to_string()),
                    actions: vec![
                        Action::Delay(Duration::from_secs(3)),
                        Action::SendLine("request system zeroize".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosWaitForBootloader,
                    condition: StateCondition::WaitForString("error: filesystem consistency checks (fsck -p -y) failed".to_string()),
                    actions: vec![
                        Action::SendLine("shell".to_string()),
                        Action::Function(Box::new(|state, p, data, _| {
                            let bdevregex = RegexBuilder::new(r"ufs: (?<device>[/a-zA-Z0-9]+) \(.*\)$")
                                .crlf(true).multi_line(true).build()?;
                            let devices: Vec<String> = bdevregex.captures_iter(data)
                                .map(|c| c.name("device").unwrap().as_str().to_string())
                                .collect();

                            loop {
                                match p.exp_string("#") {
                                    Ok(_) => break,
                                    Err(rexpect::error::Error::Timeout {..}) => {},
                                    Err(e) => return Err(e).context("failed to fix FS"),
                                }
                            }
                            for dev in devices.iter() {
                                p.send_line(format!("fsck -y {dev}").as_str())?;
                                loop {
                                    match p.exp_string("#") {
                                        Ok(_) => break,
                                        Err(rexpect::error::Error::Timeout {..}) => {},
                                        Err(e) => return Err(e).context("failed to fix FS"),
                                    }
                                }
                            }
                            p.send_line("reboot")?;
                            state.add_information(DeviceInformation::AttemptedToFixFilesystemIssues);
                            Ok(())
                        })),
                        Action::Delay(Duration::from_secs(3)),
                    ],
                },
            ]),
            ProcessStage::JunosAnswerZeroize => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosAwaitZeroizeFinish,
                    condition: StateCondition::WaitForString("Erase all data, including configuration and log files? [yes,no] (no)".to_string()),
                    actions: vec![
                        Action::SendLine("yes".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosAwaitZeroizeFinish => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosLogin,
                    condition: StateCondition::WaitForString("login:".to_string()),
                    actions: vec![
                        Action::SendLine("root".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosLogin => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosHappyCli,
                    condition: StateCondition::WaitForString("root@:RE:0%".to_string()),
                    actions: vec![
                        Action::SendLine("echo \"y\" | crontab -r".to_string()),
                        Action::SendLine("rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.*".to_string()),
                        Action::SendLine("cli".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosHappyCli,
                    condition: StateCondition::WaitForRegex(r"root@[A-Za-z0-9\-]+:RE:0%".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::KeptHostname),
                        Action::SendLine("echo \"y\" | crontab -r".to_string()),
                        Action::SendLine("rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.*".to_string()),
                        Action::SendLine("cli".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosBackupImageCli,
                    condition: StateCondition::WaitForString("Please re-install JUNOS".to_string()),
                    actions: vec![
                        Action::SendLine("echo \"y\" | crontab -r".to_string()),
                        Action::SendLine("rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.*".to_string()),
                        Action::SendLine("cli".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosHappyCli => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosVersionOutput,
                    condition: StateCondition::WaitForString("root>".to_string()),
                    actions: vec![
                        Action::SendLine("show version | no-more".to_string())
                    ],
                }
            ]),
            ProcessStage::JunosBackupImageCli => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosHappyCli,
                    condition: StateCondition::WaitForString("root>".to_string()),
                    actions: vec![
                        Action::SendLine("request system snapshot slice alternate".to_string())
                    ],
                }
            ]),

            ProcessStage::JunosVersionOutput => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosChassisOutput,
                    condition: StateCondition::WaitForString("root>".to_string()),
                    actions: vec![
                        Action::Function(Box::new(|state, _, data, _| {
                            let r = RegexBuilder::new(r"(?:Model: (?<model>[a-zA-Z0-9\-]+)$)|(?:Junos: (?<version>[0-9a-zA-Z\-\.]+)$)")
                                .multi_line(true).crlf(true).build()?;
                            for cap in r.captures_iter(&data) {
                                if let Some(model) = cap.name("model") {
                                    state.add_information(DeviceInformation::Model(model.as_str().to_string()));
                                }
                                if let Some(version) = cap.name("version") {
                                    state.add_information(DeviceInformation::SoftwareVersion(version.as_str().to_string()));
                                }
                            }
                            Ok(())
                        })),
                        Action::SendLine("show chassis hardware | no-more".to_string()),
                    ],
                }
            ]),

            ProcessStage::JunosChassisOutput => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosPoweroffConfirm,
                    condition: StateCondition::WaitForString("root>".to_string()),
                    actions: vec![
                        Action::Function(Box::new(|state, _, data, _| {
                            let r = RegexBuilder::new(r"^Chassis\s+(?<serial>[A-Za-z0-9]+)\s+.*$")
                                .multi_line(true).crlf(true).build()?;
                            for cap in r.captures_iter(&data) {
                                if let Some(serial) = cap.name("serial") {
                                    state.add_information(DeviceInformation::SerialNumber(serial.as_str().to_string()));
                                }
                            }
                            Ok(())
                        })),
                        Action::SendLine("request system power-off in 0".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosPoweroffConfirm => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosWaitForPoweroff,
                    condition: StateCondition::WaitForString("[yes,no] (no)".to_string()),
                    actions: vec![
                        Action::SendLine("yes".to_string()),
                    ],
                },
                // Some platforms only take halt, not power-off
                StateTransition {
                    target_state: ProcessStage::JunosPoweroffConfirm,
                    condition: StateCondition::WaitForString("error: command is not valid".to_string()),
                    actions: vec![
                        Action::SendLine("request system halt in 0".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosWaitForPoweroff => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("has halted.".to_string()),
                    actions: vec![],
                },
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("acpi0: Powering system off".to_string()),
                    actions: vec![],
                },
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("Rebooting...".to_string()),
                    actions: vec![],
                },
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("reboot: Power down".to_string()),
                    actions: vec![],
                },
            ]),

            ProcessStage::AristaWaitForBootloader => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaWipeStartupConfig,
                    condition: StateCondition::WaitForString("Press Control-C now to enter Aboot shell".to_string()),
                    actions: vec![
                        Action::SendControl('c'),
                    ],
                },
            ]),

            ProcessStage::AristaWipeStartupConfig => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaRebootAfterStartupConfigWipe,
                    condition: StateCondition::WaitForString("Aboot#".to_string()),
                    actions: vec![
                        Action::SendLine("rm -r /mnt/flash/.persist".to_string()),
                        Action::SendLine("rm /mnt/flash/startup-config".to_string()),
                    ],
                }
            ]),

            ProcessStage::AristaRebootAfterStartupConfigWipe => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaWaitForReboot,
                    condition: StateCondition::WaitForString("Aboot#".to_string()),
                    actions: vec![
                        Action::SendLine("exit".to_string()),
                    ],
                }
            ]),

            ProcessStage::AristaWaitForReboot => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaLoggingIn,
                    condition: StateCondition::WaitForString("login:".to_string()),
                    actions: vec![
                        Action::SendLine("admin".to_string()),
                    ],
                }
            ]),

            ProcessStage::AristaLoggingIn => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaVersionOutput,
                    condition: StateCondition::WaitForString("localhost>".to_string()),
                    actions: vec![
                        Action::SendLine("show version".to_string()),
                        Action::SendLine("bash rm -rfv /var/core/core.*".to_string())
                    ],
                }
            ]),

            ProcessStage::AristaVersionOutput => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("localhost>".to_string()),
                    actions: vec![
                        Action::Function(Box::new(|state, _, data, _| {
                            let r = RegexBuilder::new(r"(?:^Arista (?<model>[a-zA-Z \-0-9]+)$)|(?:^Serial number:\s+(?<serial>[A-Za-z0-9]+)$)|(?:Software image version: (?<version>[0-9\.A-Za-z]+)$)")
                                .multi_line(true).crlf(true).build()?;
                            for cap in r.captures_iter(&data) {
                                if let Some(model) = cap.name("model") {
                                    state.add_information(DeviceInformation::Model(model.as_str().to_string()));
                                }
                                if let Some(serial) = cap.name("serial") {
                                    state.add_information(DeviceInformation::SerialNumber(serial.as_str().to_string()));
                                }
                                if let Some(version) = cap.name("version") {
                                    state.add_information(DeviceInformation::SoftwareVersion(version.as_str().to_string()));
                                }
                            }
                            Ok(())
                        })),
                    ],
                }
            ]),

            ProcessStage::Junos23WaitForBootloader => Ok(vec![
                StateTransition {
                    condition: StateCondition::WaitForString("seconds... (press Ctrl-C to interrupt)".to_string()),
                    target_state: ProcessStage::Junos23Bootloader1,
                    actions: vec![
                        Action::SendControl('c'),
                    ],
                },
                // StateTransition {
                //     condition: StateCondition::WaitForString("localhost login:".to_string()),
                //     target_state: ProcessStage::Junos23Bootloader1,
                //     actions: vec![
                //         Action::SendControl('c'),
                //     ],
                // },
            ]),

            ProcessStage::Junos23Bootloader1 => Ok(vec![StateTransition {
                condition: StateCondition::WaitForString("Choice:".to_string()),
                target_state: ProcessStage::Junos23Bootloader2,
                actions: vec![
                    Action::Send("5".to_string()),
                    Action::Flush,
                ],
            }]),
            ProcessStage::Junos23Bootloader2 => Ok(vec![StateTransition {
                condition: StateCondition::WaitForString("Choice:".to_string()),
                target_state: ProcessStage::Junos23AwaitRecoveryShell,
                actions: vec![
                    Action::Send("2".to_string()),
                    Action::Flush,
                ],
            }]),
            ProcessStage::Junos23AwaitRecoveryShell => Ok(vec![
                StateTransition {
                    condition: StateCondition::WaitForRegex(r"\{[a-z0-9]+:0\}".to_string()),
                    target_state: ProcessStage::Junos23AnswerZeroize,
                    actions: vec![
                        Action::SendLine("request system zeroize".to_string()),
                    ],
                }
            ]),
            ProcessStage::Junos23AnswerZeroize => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::Junos23AwaitZeroizeFinish,
                    condition: StateCondition::WaitForString("Erase all data, including configuration and log files?. In case of Dual RE system, both Routing Engines will be zeroized".to_string()),
                    actions: vec![
                        Action::SendLine("yes".to_string()),
                    ],
                }
            ]),

            ProcessStage::Junos23AwaitZeroizeFinish => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::Junos23AwaitZeroizeFinish2,
                    condition: StateCondition::WaitForRegex(r"FreeBSD/[Aa]".to_string()),
                    actions: vec![
                    ],
                }
            ]),

            ProcessStage::Junos23AwaitZeroizeFinish2 => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosLogin,
                    condition: StateCondition::WaitForString("login:".to_string()),
                    actions: vec![
                        Action::SendLine("root".to_string()),
                    ],
                }
            ]),

            ProcessStage::EndJob => Ok(vec![
                StateTransition {
                    condition: StateCondition::Immediate,
                    target_state: ProcessStage::SwitchDetect,
                    actions: vec![
                        Action::FinishJob,
                    ],
                }
            ]),
        }
    }
}