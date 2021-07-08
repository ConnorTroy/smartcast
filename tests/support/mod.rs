#[macro_use]
mod macros;
mod pairing;
mod device_state;

use pairing::*;
use device_state::*;

use smartcast::{Device, Error};

use rand::{distributions::Alphanumeric, Rng};
use warp::{self, Filter};
use http::Response;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::support::device_state::power_state;

enum Result {
    Success,
    InvalidParameter,
    ChallengeIncorrect,
    Blocked,
}

impl ToString for Result {
    fn to_string(&self) -> String {
        match self {
            Self::Success           => "SUCCESS",
            Self::InvalidParameter  => "INVALID_PARAMETER",
            Self::Blocked           => "BLOCKED",
            Self::ChallengeIncorrect=> "CHALLENGE_INCORRECT",
        }.to_string()
    }
}

pub enum PortOption {
    Port8000,
    Port7345,
}

pub struct VizioDevice {
    name: String,
    model: String,
    port: u16,
    uuid: String,
    state: RwLock<State>,
    powered_on: RwLock<bool>,
    current_input: RwLock<String>,
    input_list: HashMap<String, Input>,
    pkey: String,
    cert: String,
}

#[derive(PartialEq)]
enum State {
    Ready,
    Pairing{challenge: u32, pair_token: u32, client_name: String, client_id: String},
}

impl VizioDevice {
    fn build(port: PortOption) -> Self {
        let rand_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(32)
        .collect();

        let name = "Emulated Vizio Device".to_string();
        let model = "P65Q9-H1".to_string();
        let port = match port {
            PortOption::Port7345 => 7345,
            PortOption::Port8000 => 8000,
        };
        let uuid = format!("{}-{}-{}-{}-{}",
            &rand_string[0..8],
            &rand_string[8..12],
            &rand_string[12..16],
            &rand_string[16..20],
            &rand_string[20..32]
        );

        let cert = rcgen::generate_simple_self_signed(
            vec![
                "127.0.0.1".to_string(),
                "localhost".to_string(),
            ]
        ).unwrap();
        let pkey = cert.serialize_private_key_pem();
        let cert = cert.serialize_pem().unwrap();

        let input_list = Input::generate();
        let current_input = input_list.values().nth(0).unwrap().name.clone();

        Self {
            name,
            model,
            port,
            uuid,
            state: RwLock::new(State::Ready),
            powered_on: RwLock::new(false),
            current_input: RwLock::new(current_input),
            input_list,
            pkey,
            cert,
        }
    }
}

pub async fn emulate(port: PortOption) {
    let device = Arc::new( VizioDevice::build(port) );

    // Device Description Server
    let descriptions =
        warp::path("ssdp")
        .and(warp::path("device-desc.xml"))
        .and(warp::path::end())
        .and(warp::get())
        .map({
            let device = device.clone();
            move || {
                let desc_xml = device_desc!(
                    device.name, device.model, device.uuid
                );
                Response::builder()
                    .header("Application-URL", "http//127.0.0.1:8008/apps/")
                    .header("Content-Length", desc_xml.len())
                    .header("Content-Type", "application/xml")
                    .body(desc_xml).unwrap()
            }
        });

    tokio::spawn( warp::serve(descriptions).run(([127, 0, 0, 1], 8008)) );

    // Device API Server

    let expected_put =
        warp::get()
        .map(|| {
            println!("EXPECTED_PUT");
            warp::reply::json::<Value>(
                &serde_json::from_str(
                    &format!("{{{}}}", status!("EXPECTED_PUT"))
                ).unwrap()
            )
        });

    let expected_get =
        warp::put()
        .map(|| {
            println!("EXPECTED_GET");
            warp::reply::json::<Value>(
                &serde_json::from_str(
                    &format!("{{{}}}", status!("EXPECTED_GET"))
                ).unwrap()
            )
        });

    let uri_not_found =
        warp::any()
        .map(|| {
            println!("URI_NOT_FOUND");
            warp::reply::json::<Value>(
                &serde_json::from_str(
                    &format!("{{{}}}", status!("uri_not_found"))
                ).unwrap()
            )
        });

    let pairing =
        warp::path("pairing")
        .and(
            warp::put()
            .and(warp::path::param())
            .and(warp::body::json())
            .map({
                let device = device.clone();
                move |ep: String, val: Value| {
                    match ep.as_str() {
                        "start" => pair_start(val, device.clone()),
                        "pair"  => pair_finish(val, device.clone()),
                        "cancel"=> pair_cancel(val, device.clone()),
                        _       => warp::reply::json::<Value>(
                                        &serde_json::from_str(
                                            &format!("{{{}}}", status!("uri_not_found"))
                                        ).unwrap()
                                    ),
                    }
                }
            })
            .or(expected_put)
    );

    let power_state =
        warp::path!("state" / "device" / "power_mode")
        .and(
            warp::get()
            .map({
                let device = device.clone();
                move || {
                    power_state(device.clone())
                }
            })
            .or( expected_get )
        );

    let inputs =
        warp::path("menu_native")
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
                    move || {
                        list_inputs(device.clone())
                    }
                })
                .or( expected_get )
            ).or(
                warp::path("current_input")
                .and(warp::path::end())
                .and(warp::get())
                .map({
                    let device = device.clone();
                    move || {
                        current_input(device.clone())
                    }
                })
                .or(
                    warp::path("current_input")
                    .and(warp::path::end())
                    .and(warp::put())
                    .and(warp::body::json())
                    .map({
                        let device = device.clone();
                        move |val: Value| {
                            change_input(val, device.clone())
                        }
                    })
                )

        ));

    let api =
        pairing
        .or(power_state)
        .or(inputs)
        .or(uri_not_found);
    tokio::spawn( warp::serve(api)
        .tls()
            .key(device.pkey.clone())
            .cert(device.cert.clone())
        .run(([127, 0, 0, 1], device.port))
    );
}

pub async fn connect_device() -> Device {
    let mut dev = None;

    // Try to connect until emulated device servers are ready
    while dev.is_none() {
        match Device::from_ip("127.0.0.1").await {
            Ok(d) => dev = Some(d),
            Err(Error::Reqwest(e)) if e.is_connect() => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            },
            Err(e) => panic!("{}", e),
        }
    };
    dev.unwrap()
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
        hash.insert( "CAST".into(),
            Input {
                cname: "cast".into(),
                hashval: rng.gen(),
                name: "CAST".into(),
                friendly: "SMARTCAST".into(),
                readonly: true,
            }
        );

        // HDMI
        for i in 0..4 {
            hash.insert( format!("HDMI-{}", i),
                Input {
                    cname: format!("hdmi{}", i),
                    hashval: rng.gen(),
                    name: format!("HDMI-{}", i),
                    friendly: format!("Device {}", rng.gen::<u16>()),
                    readonly: false,
                }
            );
        }

        // COMPOSITE
        hash.insert( "COMP".into(),
            Input {
                cname: "comp".into(),
                hashval: rng.gen(),
                name: "COMP".into(),
                friendly: "COMP".into(),
                readonly: false,
            }
        );

        // TUNER
        hash.insert( "TV".into(),
            Input {
                cname: "tuner".into(),
                hashval: rng.gen(),
                name: "TV".into(),
                friendly: "TV".into(),
                readonly: false,
            }
        );
        hash
    }
}
