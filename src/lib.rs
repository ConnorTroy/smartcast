mod constant;
mod device;
mod discover;
mod error;
use constant::*;

pub use device::{Button, ButtonEvent, Device, Input, ObjectType, SubSetting};
pub use error::{Error, Result};

/// Discover Vizio devices on network
///
/// This function uses SSDP to find Vizio devices
/// connected to the local network. It will return a vector of
/// [`Device`]s
pub async fn discover_devices() -> Result<Vec<Device>> {
    Ok(discover::ssdp(SSDP_IP, SSDP_URN, DEFAULT_SSDP_MAXTIME).await?)
}
