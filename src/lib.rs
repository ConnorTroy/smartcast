
mod device;
mod pair;
mod commands;
mod constants;
mod discover;
mod error;
use constants::*;

pub use device::Device;
pub use error::{Error, Result};

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn simple() {
//         async {
//             crate::get().await;
//         }
//     }
// }

/// Discover Vizio devices on network
///
/// This function uses SSDP to find Vizio devices
/// connected to the local network. It will return a vector of
/// [`Device`](Device)s
pub async fn discover_devices() -> Result<Vec<Device>> {
    Ok(discover::ssdp(
            DEFAULT_SSDP_IP,
            DEFAULT_SSDP_SERVICE,
            DEFAULT_SSDP_MAXTIME
        ).await?
    )
}
