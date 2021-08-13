#[macro_use]
mod macros;
mod simulated_device;

use simulated_device::SimulatedDevice;
pub use simulated_device::{expected_slider_info, DeviceType, PortOption, LIST_LEN};

use smartcast::{Device, Error};

use std::time::Duration;

/// Function to begin emulation of a device
pub async fn simulate(port: PortOption, device_type: DeviceType) {
    println!();
    // Start Logger
    if let Err(e) = pretty_env_logger::try_init() {
        log::warn!(target: "test::simulated::simulate", "Logger init() returned '{}'", e);
    }

    // Build simulated device
    let device = SimulatedDevice::new(port, device_type);

    // Start Description and API Servers
    device.serve();
}

/// This function will return a `Device`. It will continuously try to connect by ip until the simulated servers are ready.
/// Unexpected errors will panic.
pub async fn connect_device() -> Device {
    let mut dev = None;
    spawn_fail_timer().await;

    // Try to connect until simulated device servers are ready
    while dev.is_none() {
        match Device::from_ip("127.0.0.1").await {
            Ok(d) => dev = Some(d),
            Err(Error::Reqwest(e)) if e.is_connect() => {
                log::warn!(target: "test::simulated::connect_device", "Unable to connect: '{:?}', retrying...", e);
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
        log::error!(target: "test::simulated::timer", "Test took too long");
        panic!("Test took too long");
    });
}

/// Random data helpers
pub mod rand_data {
    use rand::{distributions::Alphanumeric, Rng};

    pub fn string(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(len)
            .collect()
    }

    pub fn uuid() -> String {
        let rand_string = string(32);
        format!(
            "{}-{}-{}-{}-{}",
            &rand_string[0..8],
            &rand_string[8..12],
            &rand_string[12..16],
            &rand_string[16..20],
            &rand_string[20..32]
        )
    }
}
