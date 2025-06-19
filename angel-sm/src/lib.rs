use cthulhu_common::devinfo::{DeviceInformation, DeviceInformationType};
use cthulhu_common::status::JobUpdate;

pub mod action;
pub mod builder;
pub mod data_structure;
pub mod pfunc;
pub mod state;
pub mod trigger;

mod util;

//pub mod process;
//pub mod state;

//TODO: Figure out how to properly fix the warning.
#[allow(async_fn_in_trait)]
pub trait AngelJob {
    async fn init_job(&mut self) -> color_eyre::Result<()>;
    async fn send_update(&mut self, update: JobUpdate) -> color_eyre::Result<()>;
    async fn reset(&mut self) -> color_eyre::Result<()>;
    async fn add_information(&mut self, information: DeviceInformation) -> color_eyre::Result<()>;
    fn get_information(&self) -> &[DeviceInformation];
    fn get_max_information_type(&self) -> Option<DeviceInformationType>;
}
