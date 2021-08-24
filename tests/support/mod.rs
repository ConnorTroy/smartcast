#![allow(dead_code)]

#[macro_use]
mod macros;
mod simulated_device;

use simulated_device::SimulatedDevice;
pub use simulated_device::{expected_slider_info, CodeSet, DeviceType, PortOption, LIST_LEN};

use smartcast::{Device, Error};

use tokio::time::{sleep, Sleep};

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// Calls [Simulate] future to start simulated device and get client. Then runs the test passed in. Panics after 5 seconds.
pub struct Test {
    test: Pin<Box<dyn Future<Output = ()>>>,
    timeout: Pin<Box<Sleep>>,
}

impl Test {
    pub async fn simulate<F, Fut: 'static>(
        port_option: PortOption,
        device_type: DeviceType,
        code_set: CodeSet,
        func: F,
    ) where
        F: FnOnce(Device) -> Fut,
        Fut: Future<Output = ()>,
    {
        let start = Instant::now();
        let timeout = Duration::from_secs(5);

        let dev = Simulate::startup(port_option, device_type, code_set, timeout).await;

        Test {
            test: Box::pin(func(dev)),
            timeout: Box::pin(sleep(timeout - Instant::now().duration_since(start))),
        }
        .await
    }
}

impl Future for Test {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.timeout.as_mut().poll(cx).is_ready() {
            panic!("Test took too long");
        }

        self.test.as_mut().poll(cx)
    }
}

/// Future starts up a [`SimulatedDevice`] and provides a [`Device`] client. Panics after specified duration.
struct Simulate {
    startup: Pin<Box<dyn Future<Output = ()>>>,
    connect: Pin<Box<dyn Future<Output = Device>>>,
    ready_to_connect: bool,
    timeout: Pin<Box<Sleep>>,
}

impl Simulate {
    fn startup(
        port_option: PortOption,
        device_type: DeviceType,
        code_set: CodeSet,
        timeout: Duration,
    ) -> Simulate {
        Simulate {
            startup: Box::pin(simulate(port_option, device_type, code_set)),
            connect: Box::pin(connect_device()),
            ready_to_connect: false,
            timeout: Box::pin(sleep(timeout)),
        }
    }
}

impl Future for Simulate {
    type Output = Device;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.timeout.as_mut().poll(cx).is_ready() {
            panic!("Test took too long");
        }

        if !self.ready_to_connect {
            self.ready_to_connect = self.startup.as_mut().poll(cx).is_ready();
        }

        if self.ready_to_connect {
            self.connect.as_mut().poll(cx)
        } else {
            Poll::Pending
        }
    }
}

/// Function to begin emulation of a device
pub async fn simulate(port: PortOption, device_type: DeviceType, command_set: CodeSet) {
    // Start Logger
    if let Err(e) = pretty_env_logger::try_init() {
        log::warn!(target: "test::simulated::simulate", "Logger init() returned '{}'", e);
    }

    // Build simulated device
    let device = SimulatedDevice::new(port, device_type, command_set);

    // Start Description and API Servers
    device.serve();
}

/// This function will return a `Device`. It will continuously try to connect by ip until the simulated servers are ready.
/// Unexpected errors will panic.
pub async fn connect_device() -> Device {
    let mut dev = None;
    // spawn_fail_timer().await;

    // Try to connect until simulated device servers are ready
    while dev.is_none() {
        match Device::from_ip("127.0.0.1").await {
            Ok(d) => dev = Some(d),
            Err(Error::Reqwest(e)) if e.is_connect() => {
                log::warn!(target: "test::simulated::connect_device", "Unable to connect, retrying...");
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => panic!("{}", e),
        }
    }
    dev.unwrap()
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

/// Other helper functions
pub mod helpers {
    pub fn button_vec() -> Vec<smartcast::Button> {
        (0..30)
            .map(|i| match i {
                0 => smartcast::Button::SeekFwd,
                1 => smartcast::Button::SeekBack,
                2 => smartcast::Button::Pause,
                3 => smartcast::Button::Play,
                4 => smartcast::Button::Down,
                5 => smartcast::Button::Left,
                6 => smartcast::Button::Up,
                7 => smartcast::Button::Right,
                8 => smartcast::Button::Ok,
                9 => smartcast::Button::Back,
                10 => smartcast::Button::SmartCast,
                11 => smartcast::Button::CCToggle,
                12 => smartcast::Button::Info,
                13 => smartcast::Button::Menu,
                14 => smartcast::Button::Home,
                15 => smartcast::Button::VolumeDown,
                16 => smartcast::Button::VolumeUp,
                17 => smartcast::Button::MuteOff,
                18 => smartcast::Button::MuteOn,
                19 => smartcast::Button::MuteToggle,
                20 => smartcast::Button::PicMode,
                21 => smartcast::Button::PicSize,
                22 => smartcast::Button::InputNext,
                23 => smartcast::Button::ChannelDown,
                24 => smartcast::Button::ChannelUp,
                25 => smartcast::Button::ChannelPrev,
                26 => smartcast::Button::Exit,
                27 => smartcast::Button::PowerOff,
                28 => smartcast::Button::PowerOn,
                29 => smartcast::Button::PowerToggle,
                _ => panic!(),
            })
            .collect()
    }
}
