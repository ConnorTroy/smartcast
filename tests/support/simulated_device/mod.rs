mod commands;
mod inputs;
mod settings;

use super::rand_data;

use inputs::Input;
pub use settings::{expected_slider_info, LIST_LEN};

use http::Response;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde_json::Value;
use warp::{filters::BoxedFilter, Filter, Reply};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Result for command response
enum Result {
    Success,
    InvalidParameter,
    ChallengeIncorrect,
    Blocked,
}

impl ToString for Result {
    fn to_string(&self) -> String {
        match self {
            Self::Success => "SUCCESS",
            Self::InvalidParameter => "INVALID_PARAMETER",
            Self::Blocked => "BLOCKED",
            Self::ChallengeIncorrect => "CHALLENGE_INCORRECT",
        }
        .to_string()
    }
}

/// Random will choose port 7345 or 9000 at random
pub enum PortOption {
    Port9000,
    Port7345,
    Random,
}

impl From<PortOption> for u16 {
    fn from(option: PortOption) -> Self {
        match option {
            PortOption::Port7345 => 7345,
            PortOption::Port9000 => 9000,
            PortOption::Random => PortOption::into(rand::random()),
        }
    }
}

impl Distribution<PortOption> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PortOption {
        match rng.gen_range(0..2) {
            0 => PortOption::Port9000,
            1 => PortOption::Port7345,
            _ => panic!("Rand Port - Bad Range"),
        }
    }
}

/// Random will choose TV or SoundBar at random
pub enum DeviceType {
    TV,
    SoundBar,
    Random,
}

impl DeviceType {
    fn settings_root(self) -> String {
        match self {
            Self::TV => "tv_settings".into(),
            Self::SoundBar => "audio_settings".into(),
            Self::Random => Self::settings_root(rand::random()),
        }
    }
}

impl Distribution<DeviceType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DeviceType {
        match rng.gen_range(0..2) {
            0 => DeviceType::TV,
            1 => DeviceType::SoundBar,
            _ => panic!("Rand Device Type - Bad Range"),
        }
    }
}

#[derive(Debug)]
pub enum CodeSet {
    Default,
    Secondary,
    Random,
}

impl CodeSet {
    fn choose(self) -> Self {
        match self {
            Self::Random => rand::random(),
            not_random => not_random,
        }
    }
    fn hashmap(self) -> HashMap<u32, Vec<u32>> {
        if matches!(self, Self::Random) {
            return self.choose().hashmap();
        }

        let mut hash = HashMap::new();
        hash.insert(2, vec![0, 1, 2, 3]);
        match self {
            Self::Default => {
                hash.insert(3, vec![0, 1, 8, 7, 2]);
            }
            Self::Secondary => {
                hash.insert(3, vec![0, 1, 3, 5, 2]);
            }
            _ => panic!("CodeSet not chosen"),
        }
        hash.insert(4, vec![0, 3, 4, 6, 8, 15]);
        hash.insert(5, vec![0, 1, 2, 3, 4]);
        hash.insert(6, vec![0, 2]);
        hash.insert(7, vec![1]);
        hash.insert(8, vec![0, 1, 2]);
        hash.insert(9, vec![0]);
        hash.insert(11, vec![0, 1, 2]);
        hash
    }
}

impl Distribution<CodeSet> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CodeSet {
        match rng.gen_range(0..2) {
            0 => CodeSet::Default,
            1 => CodeSet::Secondary,
            _ => panic!("Rand CodeSet - Bad Range"),
        }
    }
}

/// Device state used for pairing
#[derive(Debug, PartialEq)]
enum State {
    Ready,
    Pairing {
        challenge: u32,
        pair_token: u32,
        client_name: String,
        client_id: String,
    },
}

/// Simulated Device which tests will attempt to connect to
#[derive(Debug, Clone)]
pub struct SimulatedDevice {
    inner: Arc<SimulatedDeviceRef>,
}

impl SimulatedDevice {
    pub fn new(port: PortOption, device_type: DeviceType, code_set: CodeSet) -> Self {
        let name = "Simulated Device".to_string();
        let model = rand_data::string(6);
        let settings_root = device_type.settings_root();
        let port = port.into();
        let uuid = rand_data::uuid();

        let input_list = inputs::generate();
        let current_input = input_list.values().next().unwrap().name.clone();

        let cert = rcgen::generate_simple_self_signed(vec![
            "127.0.0.1".to_string(),
            "localhost".to_string(),
        ])
        .unwrap();
        let pkey = cert.serialize_private_key_pem();
        let cert = cert.serialize_pem().unwrap();

        Self {
            inner: Arc::new(SimulatedDeviceRef {
                name,
                model,
                settings_root,
                port,
                uuid,
                code_set: code_set.hashmap(),
                state: RwLock::new(State::Ready),
                powered_on: RwLock::new(false),
                input_list,
                current_input: RwLock::new(current_input),
                cert,
                pkey,
            }),
        }
    }

    pub fn serve(&self) {
        // Device Description Server
        tokio::spawn(warp::serve(self.description()).run(([127, 0, 0, 1], 8008)));
        log::info!(target: "test::simulated_device::serve", "Starting Description server");

        // Device API Server
        tokio::spawn(
            warp::serve(self.api())
                .tls()
                .key(self.inner.pkey.clone())
                .cert(self.inner.cert.clone())
                .run(([127, 0, 0, 1], self.inner.port)),
        );
        log::info!(target: "test::simulated_device::serve", "Starting API server");
    }

    fn description(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("ssdp")
            .and(warp::path("device-desc.xml"))
            .and(warp::path::end())
            .and(warp::get())
            .map({
                let desc_xml = device_desc!(self.inner.name, self.inner.model, self.inner.uuid);
                move || {
                    let desc_xml = desc_xml.clone();
                    Response::builder()
                        .header("Application-URL", "http//127.0.0.1:8008/apps/")
                        .header("Content-Length", desc_xml.len())
                        .header("Content-Type", "application/xml")
                        .body(desc_xml)
                        .unwrap()
                }
            })
            .with(warp::log("test::simulated_device::description"))
            .boxed()
    }

    fn api(&self) -> BoxedFilter<(impl Reply,)> {
        self.pairing()
            .or(self.power_state())
            .or(self.inputs())
            .or(self.device_info())
            .or(self.settings())
            .or(self.virtual_remote())
            .or(self.uri_not_found())
            .with(warp::log("test::simulated_device::api"))
            .boxed()
    }

    /// EXPECTED_PUT Status Result
    fn expected_put(&self) -> BoxedFilter<(impl Reply,)> {
        warp::get()
            .map(|| {
                warp::reply::json::<Value>(
                    &serde_json::from_str(&format!("{{{}}}", status!("EXPECTED_PUT"))).unwrap(),
                )
            })
            .boxed()
    }

    /// EXPECTED_GET Status Result
    fn expected_get(&self) -> BoxedFilter<(impl Reply,)> {
        warp::put()
            .map(|| {
                warp::reply::json::<Value>(
                    &serde_json::from_str(&format!("{{{}}}", status!("EXPECTED_GET"))).unwrap(),
                )
            })
            .boxed()
    }

    /// URI_NOT_FOUND Status Result
    fn uri_not_found(&self) -> BoxedFilter<(impl Reply,)> {
        warp::any()
            .map(|| {
                warp::reply::json::<Value>(
                    &serde_json::from_str(&format!("{{{}}}", status!("uri_not_found"))).unwrap(),
                )
            })
            .boxed()
    }

    /// Pairing Commands
    fn pairing(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("pairing")
            .and(
                warp::put()
                    .and(warp::path::param())
                    .and(warp::path::end())
                    .and(warp::body::json())
                    .map({
                        let device = self.clone();
                        move |ep: String, val: Value| match ep.as_str() {
                            "start" => commands::pair_start(val, device.clone()),
                            "pair" => commands::pair_finish(val, device.clone()),
                            "cancel" => commands::pair_cancel(val, device.clone()),
                            _ => warp::reply::json::<Value>(
                                &serde_json::from_str(&format!("{{{}}}", status!("uri_not_found")))
                                    .unwrap(),
                            ),
                        }
                    })
                    .or(self.expected_put()),
            )
            .boxed()
    }

    /// Input Commands
    fn inputs(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path!("menu_native" / "dynamic" / ..)
            .and(warp::path(self.inner.settings_root.clone()))
            .and(warp::path("devices"))
            .and(
                warp::path("name_input")
                    .and(warp::path::end())
                    .and(
                        warp::get()
                            .map({
                                let device = self.clone();
                                move || commands::list_inputs(device.clone())
                            })
                            .or(self.expected_get()),
                    )
                    .or(warp::path("current_input")
                        .and(warp::path::end())
                        .and(warp::get().map({
                            let device = self.clone();
                            move || commands::current_input(device.clone())
                        }))
                        .or(warp::put().and(warp::body::json()).map({
                            let device = self.clone();
                            move |val: Value| commands::change_input(val, device.clone())
                        }))),
            )
            .boxed()
    }

    /// Power State Command
    fn power_state(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path!("state" / "device" / "power_mode")
            .and(
                warp::get()
                    .map({
                        let device = self.clone();
                        move || commands::power_state(device.clone())
                    })
                    .or(self.expected_put()),
            )
            .boxed()
    }

    /// Device Info Command
    fn device_info(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path!("state" / "device" / "deviceinfo")
            .and(
                warp::get()
                    .map({
                        let device = self.clone();
                        move || commands::device_info(device.clone())
                    })
                    .or(self.expected_get()),
            )
            .boxed()
    }

    /// Read/Write Settings Commands
    fn settings(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("menu_native")
            .and(settings::generate(self.inner.settings_root.clone()))
            .boxed()
    }

    fn virtual_remote(&self) -> BoxedFilter<(impl Reply,)> {
        warp::path("key_command")
            .and(
                warp::put()
                    .and(warp::body::json())
                    .map({
                        let device = self.clone();
                        move |val: Value| commands::virtual_remote(val, device.clone())
                    })
                    .or(self.expected_get()),
            )
            .boxed()
    }
}

#[derive(Debug)]
struct SimulatedDeviceRef {
    name: String,
    model: String,
    settings_root: String,
    port: u16,
    uuid: String,
    code_set: HashMap<u32, Vec<u32>>,
    state: RwLock<State>,
    powered_on: RwLock<bool>,
    input_list: HashMap<String, Input>,
    current_input: RwLock<String>,
    cert: String,
    pkey: String,
}
