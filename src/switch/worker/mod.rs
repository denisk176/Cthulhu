use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use color_eyre::eyre::WrapErr;
use io_tee::TeeReader;
use tracing::{info, span, Level};
use rexpect::ReadUntil;
use crate::switch::{DeviceInformation, PortCommand, PortCommandReceiver, PortConfig, PortUpdate, PortUpdateSender};
use crate::switch::worker::action::Action;
use crate::switch::worker::logging::{ContainedWriter, TracingWriter};
use crate::switch::worker::process::ProcessStage;
use crate::switch::worker::state::{StateCondition, StateTransition, SwitchData};

pub mod process;
pub mod state;
pub mod logging;
pub mod action;
type RexpectSession = rexpect::session::StreamSession<Box<dyn Write + Send + 'static>>;

fn make_log_writer(config: &PortConfig, state: &SwitchData) -> color_eyre::Result<Box<dyn Write + Send + 'static>> {
    let date = state.get_started();
    if !std::fs::exists("logs")? {
        std::fs::create_dir("logs")?;
    }
    let b = PathBuf::new().join("logs/").join(format!("{}-{}.log", config.label, date.format("%Y-%m-%d-%H-%M-%S")));
    Ok(Box::new(OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(b)?))
}


pub fn worker_function(config: &PortConfig, update_sender: PortUpdateSender, command_receiver: PortCommandReceiver, reader: Box<dyn Read + Send + 'static>, writer: Box<dyn Write + Send + 'static>) -> color_eyre::Result<()> {
    let mut state = SwitchData::default();
    let mut device_state = ProcessStage::default();

    // This next block is the very convoluted logging setup.
    let containedwriter: ContainedWriter = ContainedWriter::new(make_log_writer(&config, &state)?);
    // We configure one sub per thread so we can log to files properly.
    let subscriber = {
        let containedwriter = containedwriter.clone();
        tracing_subscriber::fmt()
            .with_writer(move || containedwriter.clone())
            .finish()
    };
    let _guard = tracing::subscriber::set_default(subscriber);

    let my_span = span!(Level::INFO, "port_worker", port = config.label.as_str());
    let _guard = my_span.enter();


    update_sender.send(PortUpdate::PortJobStart(state.get_started()))?;

    // Send a copy of the serial data to the logging system.
    let logwriter = TracingWriter::new(containedwriter.clone());
    //let logwriter = std::io::stdout();
    let logwriter = strip_ansi_escapes::Writer::new(logwriter);
    let reader = TeeReader::new(reader, logwriter);

    let mut p = rexpect::session::StreamSession::new(reader, writer, rexpect::reader::Options {
        // This value ensures we get a return to our code every 100ms.
        // This depends on an implementation details of rexpect.
        // It forces the loop in reader.rs#251 to exit after two loops.
        timeout_ms: Some(50),
        strip_ansi_escape_codes: true,
    });

    let transition_to_state = |device_state: &mut ProcessStage, state: &mut SwitchData, p: &mut RexpectSession, t: &StateTransition, d: &str, m: &str| -> color_eyre::Result<()> {
        let old_state = device_state.clone();
        *device_state = t.target_state;
        for action in &t.actions {
            action.perform(config, state, containedwriter.clone(), &update_sender, p, &d, &m)?;
        }
        info!("State transition: {:?} -> {:?}", old_state, t.target_state);
        update_sender.send(PortUpdate::PortStateTransition(old_state, t.target_state))?;
        Ok(())
    };

    info!("Entering loop...");

    loop {
        let transitions = device_state.get_transitions()?;
        if let Some(t) = transitions.iter().find(|t| t.condition == StateCondition::Immediate) {
            transition_to_state(&mut device_state, &mut state, &mut p, t, "", "")?;
        } else {
            let u = ReadUntil::Any(transitions.iter().map(|t| t.condition.to_needle().map(|v| v.unwrap())).collect::<color_eyre::Result<Vec<_>>>()?);
            'read_loop: loop {
                if let Some(v) = command_receiver.try_recv()? {
                    info!("Received command: {v:?}");
                    match v {
                        PortCommand::ResetJob => {
                            device_state = ProcessStage::default();
                            let a = vec![
                                Action::AddDeviceInfo(DeviceInformation::Aborted),
                                Action::FinishJob,
                            ];
                            for action in a {
                                action.perform(config, &mut state, containedwriter.clone(), &update_sender, &mut p, "", "")?;
                            }
                        }
                    }
                    break 'read_loop;
                }

                // Try to handle a result from the switches.
                let r_res = p.exp(&u);
                match r_res {
                    Ok((d, m)) => {
                        't_test: for t in &transitions {
                            if t.condition.matches_result(&m)? {
                                transition_to_state(&mut device_state, &mut state, &mut p, t, &d, &m)?;
                                break 't_test;
                            }
                        }
                        break 'read_loop;
                    }
                    Err(rexpect::error::Error::Timeout { .. }) => {
                        // Ignore
                    }
                    Err(e) => return Err(e).context("failed to read from serial port"),
                }
            }
        }
    }
}
