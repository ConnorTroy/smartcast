#[macro_use]
mod macros;
mod emulated_device;

use emulated_device::EmulatedDevice;
pub use emulated_device::{DeviceType, PortOption};

use smartcast::{Device, Error};

use std::time::Duration;

/// Function to begin emulation of a device
pub async fn emulate(port: PortOption, device_type: DeviceType) -> tokio::task::JoinHandle<()> {
    // Build emulated device
    let device = EmulatedDevice::build(port, device_type);

    // Start Description and API Servers
    device.serve()
}

/// This function will return a `Device`. It will continuously try to connect by ip until the emulated servers are ready.
/// Unexpected errors will panic.
pub async fn connect_device() -> Device {
    let mut dev = None;
    spawn_fail_timer().await;

    // Try to connect until emulated device servers are ready
    while dev.is_none() {
        match Device::from_ip("127.0.0.1").await {
            Ok(d) => dev = Some(d),
            Err(Error::Reqwest(e)) if e.is_connect() => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => panic!("{}", e),
        }
    }
    dev.unwrap()
}

/// Panics after 5 seconds
pub async fn spawn_fail_timer() {
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(5)).await;
        panic!("Test took too long");
    });
}
