#[macro_use]
mod macros;
mod device_state;
mod info;
mod pairing;

use device_state::*;
use info::*;
use pairing::*;

use smartcast::{Device, Error};

use http::Response;
use rand::{Rng, distributions::{Distribution, Standard}};
use serde_json::Value;
use warp::{self, Filter};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

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

pub enum PortOption {
    Port9000,
    Port7345,
}

impl Into<u16> for PortOption {
    fn into(self) -> u16 {
        match self {
            Self::Port7345 => 7345,
            Self::Port9000 => 9000
        }
    }
}

pub enum DeviceType {
    TV,
    Soundbar,
    Random,
}

impl DeviceType {
    fn settings_root(self) -> String {
        match self {
            Self::TV => "tv_settings".into(),
            Self::Soundbar => "audio_settings".into(),
            Self::Random => Self::settings_root(rand::random()),
        }
    }
}

impl Distribution<DeviceType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DeviceType {
        match rng.gen_range(0..2) {
            0 => DeviceType::TV,
            1 => DeviceType::Soundbar,
            _ => panic!("Bad Range"),
        }
    }
}

#[derive(PartialEq)]
enum State {
    Ready,
    Pairing {
        challenge: u32,
        pair_token: u32,
        client_name: String,
        client_id: String,
    },
}

pub struct EmulatedDevice {
    name: String,
    model: String,
    settings_root: String,
    port: u16,
    uuid: String,
    state: RwLock<State>,
    powered_on: RwLock<bool>,
    input_list: HashMap<String, Input>,
    current_input: RwLock<String>,
    cert: String,
    pkey: String,
}

impl EmulatedDevice {
    fn build(port: PortOption, device_type: DeviceType) -> Self {

        let name = "Emulated Device".to_string();
        let model = rand_data::string(6);
        let settings_root = device_type.settings_root();
        let port = port.into();
        let uuid = rand_data::uuid();

        let input_list = Input::generate();
        let current_input = input_list.values().next().unwrap().name.clone();

        let cert = rcgen::generate_simple_self_signed(vec![
            "127.0.0.1".to_string(),
            "localhost".to_string(),
        ])
        .unwrap();
        let pkey = cert.serialize_private_key_pem();
        let cert = cert.serialize_pem().unwrap();


        Self {
            name,
            model,
            settings_root,
            port,
            uuid,
            state: RwLock::new(State::Ready),
            powered_on: RwLock::new(false),
            input_list,
            current_input: RwLock::new(current_input),
            cert,
            pkey,
        }
    }
}

pub async fn emulate(port: PortOption, device_type: DeviceType) -> tokio::task::JoinHandle<()> {
    let device = Arc::new(EmulatedDevice::build(port, device_type));

    // --- Device Description Server
    let descriptions = warp::path("ssdp")
        .and(warp::path("device-desc.xml"))
        .and(warp::path::end())
        .and(warp::get())
        .map({
            let device = device.clone();
            move || {
                let desc_xml = device_desc!(device.name, device.model, device.uuid);
                Response::builder()
                    .header("Application-URL", "http//127.0.0.1:8008/apps/")
                    .header("Content-Length", desc_xml.len())
                    .header("Content-Type", "application/xml")
                    .body(desc_xml)
                    .unwrap()
            }
        });

    tokio::spawn(warp::serve(descriptions).run(([127, 0, 0, 1], 8008)));

    // --- Device API Server
    let expected_put = warp::get().map(|| {
        warp::reply::json::<Value>(
            &serde_json::from_str(&format!("{{{}}}", status!("EXPECTED_PUT"))).unwrap(),
        )
    });

    let expected_get = warp::put().map(|| {
        warp::reply::json::<Value>(
            &serde_json::from_str(&format!("{{{}}}", status!("EXPECTED_GET"))).unwrap(),
        )
    });

    let uri_not_found = warp::any().map(|| {
        warp::reply::json::<Value>(
            &serde_json::from_str(&format!("{{{}}}", status!("uri_not_found"))).unwrap(),
        )
    });

    // Pairing Commands
    let pairing = warp::path("pairing").and(
        warp::put()
            .and(warp::path::param())
            .and(warp::body::json())
            .map({
                let device = device.clone();
                move |ep: String, val: Value| match ep.as_str() {
                    "start" => pair_start(val, device.clone()),
                    "pair" => pair_finish(val, device.clone()),
                    "cancel" => pair_cancel(val, device.clone()),
                    _ => warp::reply::json::<Value>(
                        &serde_json::from_str(&format!("{{{}}}", status!("uri_not_found")))
                            .unwrap(),
                    ),
                }
            })
            .or(expected_put),
    );

    // Power State
    let power_state = warp::path!("state" / "device" / "power_mode").and(
        warp::get()
            .map({
                let device = device.clone();
                move || power_state(device.clone())
            })
            .or(expected_get),
    );

    // Input Commands
    let inputs = warp::path("menu_native")
        .and(warp::path("dynamic"))
        .and(warp::path("tv_settings"))
        .and(warp::path("devices"))
        .and(
            warp::path("name_input")
                .and(warp::path::end())
                .and(
                    warp::get()
                        .map({
                            let device = device.clone();
                            move || list_inputs(device.clone())
                        })
                        .or(expected_get),
                )
                .or(warp::path("current_input")
                    .and(warp::path::end())
                    .and(warp::get())
                    .map({
                        let device = device.clone();
                        move || current_input(device.clone())
                    })
                    .or(warp::path("current_input")
                        .and(warp::path::end())
                        .and(warp::put())
                        .and(warp::body::json())
                        .map({
                            let device = device.clone();
                            move |val: Value| change_input(val, device.clone())
                        }))),
        );

    let info = warp::path!("state" / "device" / "deviceinfo").and(
        warp::get()
            .map({
                let device = device.clone();
                move || device_info(device.clone())
            })
            .or(expected_get),
    );

    let api = pairing
        .or(power_state)
        .or(inputs)
        .or(info)
        .or(uri_not_found);
    tokio::spawn(
        warp::serve(api)
            .tls()
            .key(device.pkey.clone())
            .cert(device.cert.clone())
            .run(([127, 0, 0, 1], device.port)),
    )
}

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

pub async fn spawn_fail_timer() {
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_secs(5)).await;
        panic!("Test took too long");
    });
}

struct Input {
    cname: String,
    hashval: u32,
    name: String,
    friendly: String,
    readonly: bool,
}

impl Input {
    fn generate() -> HashMap<String, Input> {
        let mut rng = rand::thread_rng();
        let mut hash = HashMap::new();

        // SMARTCAST Input
        hash.insert(
            "CAST".into(),
            Input {
                cname: "cast".into(),
                hashval: rng.gen(),
                name: "CAST".into(),
                friendly: "SMARTCAST".into(),
                readonly: true,
            },
        );

        // HDMI
        for i in 0..4 {
            hash.insert(
                format!("HDMI-{}", i),
                Input {
                    cname: format!("hdmi{}", i),
                    hashval: rng.gen(),
                    name: format!("HDMI-{}", i),
                    friendly: format!("Device {}", rng.gen::<u16>()),
                    readonly: false,
                },
            );
        }

        // COMPOSITE
        hash.insert(
            "COMP".into(),
            Input {
                cname: "comp".into(),
                hashval: rng.gen(),
                name: "COMP".into(),
                friendly: "COMP".into(),
                readonly: false,
            },
        );

        // TUNER
        hash.insert(
            "TV".into(),
            Input {
                cname: "tuner".into(),
                hashval: rng.gen(),
                name: "TV".into(),
                friendly: "TV".into(),
                readonly: false,
            },
        );
        hash
    }
}

mod rand_data {
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
        format!("{}-{}-{}-{}-{}",
            &rand_string[0..8],
            &rand_string[8..12],
            &rand_string[12..16],
            &rand_string[16..20],
            &rand_string[20..32]
        )
    }
}
