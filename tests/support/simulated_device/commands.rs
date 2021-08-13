use super::{settings::Setting, Input, Result, SimulatedDevice, State};

use rand::Rng;
use serde_json::{Value, json};

/// Start pairing command
pub fn pair_start(mut val: Value, device: SimulatedDevice) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "PAIR START");
    let client_id = serde_json::from_value::<String>(val["DEVICE_ID"].take());
    let client_name = serde_json::from_value::<String>(val["DEVICE_NAME"].take());

    let mut res: String = match (client_id, client_name, device.inner.state.write()) {
        (Ok(client_id), Ok(client_name), Ok(mut state)) if *state == State::Ready => {
            let mut rng = rand::thread_rng();
            let challenge: u32 = 1;
            let pair_token: u32 = rng.gen();
            *state = State::Pairing {
                challenge,
                pair_token,
                client_id,
                client_name,
            };
            format!(
                r#"
                "ITEM": {{
                    "PAIRING_REQ_TOKEN": {},
                    "CHALLENGE_TYPE": {}
                }},
                {}
            "#,
                pair_token,
                challenge,
                status!(Result::Success)
            )
        }
        (_, _, Ok(state)) if *state != State::Ready => status!(Result::Blocked),
        (_, _, Err(_)) => status!(Result::Blocked),
        _ => status!(Result::InvalidParameter),
    };

    res.insert(0, '{');
    res.push('}');
    let res: Value = serde_json::from_str(&res).unwrap();

    warp::reply::json(&res)
}

/// Finish pairing command
pub fn pair_finish(mut val: Value, device: SimulatedDevice) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "PAIR FINISH");
    let client_id = serde_json::from_value::<String>(val["DEVICE_ID"].take());
    let challenge = serde_json::from_value::<u32>(val["CHALLENGE_TYPE"].take());
    let pin = serde_json::from_value::<String>(val["RESPONSE_VALUE"].take());
    let pair_token = serde_json::from_value::<u32>(val["PAIRING_REQ_TOKEN"].take());

    let mut res: String = match (
        client_id,
        challenge,
        pin,
        pair_token,
        device.inner.state.write(),
    ) {
        (Ok(client_id), Ok(challenge), Ok(_), Ok(pair_token), Ok(mut state)) => match &*state {
            State::Pairing {
                challenge: exp_challenge,
                pair_token: exp_pair,
                client_name: _,
                client_id: exp_id,
            } => {
                if challenge != *exp_challenge {
                    status!(Result::ChallengeIncorrect)
                } else if client_id != *exp_id || pair_token != *exp_pair {
                    status!(Result::InvalidParameter)
                } else {
                    *state = State::Ready;
                    format!(
                        r#"
                            "ITEM": {{
                                "AUTH_TOKEN": "{}"
                            }},
                            {}
                        "#,
                        0,
                        status!(Result::Success)
                    )
                }
            }
            _ => status!(Result::Blocked),
        },
        (_, _, _, _, Err(_)) => status!(Result::Blocked),
        _ => status!(Result::InvalidParameter),
    };

    res.insert(0, '{');
    res.push('}');
    let res: Value = serde_json::from_str(&res).unwrap();

    warp::reply::json(&res)
}

/// Cancel pairing command
pub fn pair_cancel(mut val: Value, device: SimulatedDevice) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "PAIR CANCEL");
    let client_id = serde_json::from_value::<String>(val["DEVICE_ID"].take());
    let challenge = serde_json::from_value::<u32>(val["CHALLENGE_TYPE"].take());
    let pin = serde_json::from_value::<String>(val["RESPONSE_VALUE"].take());
    let pair_token = serde_json::from_value::<u32>(val["PAIRING_REQ_TOKEN"].take());

    let mut res: String = match (
        client_id,
        challenge,
        pin,
        pair_token,
        device.inner.state.write(),
    ) {
        (Ok(client_id), Ok(challenge), Ok(pin), Ok(pair_token), Ok(mut state))
            if *state != State::Ready =>
        {
            match &*state {
                State::Pairing {
                    challenge: exp_challenge,
                    pair_token: exp_pair,
                    client_name: _,
                    client_id: exp_id,
                } => {
                    if challenge != *exp_challenge
                        || client_id != *exp_id
                        || pin != "1111"
                        || pair_token != *exp_pair
                    {
                        status!(Result::InvalidParameter)
                    } else {
                        *state = State::Ready;
                        format!(
                            r#"
                            "ITEM": {{}},
                            {}
                        "#,
                            status!(Result::Success)
                        )
                    }
                }
                _ => status!(Result::Blocked),
            }
        }
        (_, _, _, _, Ok(state)) if *state == State::Ready => status!(Result::Blocked),
        (_, _, _, _, Err(_)) => status!(Result::Blocked),
        _ => status!(Result::InvalidParameter),
    };

    res.insert(0, '{');
    res.push('}');
    let res: Value = serde_json::from_str(&res).unwrap();

    warp::reply::json(&res)
}

/// Get power state command
pub fn power_state(device: SimulatedDevice) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "POWER STATE");
    let res = format!(
        r#"
    {{
        "ITEMS": [{{
            "TYPE": "T_VALUE_V1",
            "CNAME": "power_mode",
            "NAME": "Power Mode",
            "VALUE": {}
        }}],
        "PARAMETERS": {{
            "HASHONLY": "FALSE",
            "FLAT": "TRUE",
            "HELPTEXT": "FALSE"
    }},
    {}}}"#,
        *device.inner.powered_on.read().unwrap() as u32,
        status!(Result::Success)
    );

    let res = serde_json::from_str(&res).unwrap();

    warp::reply::json::<Value>(&res)
}

/// Get current input command
pub fn current_input(device: SimulatedDevice) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "CURRENT INPUT");
    let input: &Input = device
        .inner
        .input_list
        .get(&*device.inner.current_input.read().unwrap())
        .unwrap();

    let mut rng = rand::thread_rng();

    warp::reply::json::<Value>(
        &serde_json::from_str(&format!(
            r#"
            {{
                "HASHLIST": [
                    {},
                    {}
                ],
                "ITEMS": [{{
                    "CNAME": "current_input",
                    "ENABLED": "FALSE",
                    "HASHVAL": {},
                    "HIDDEN": "TRUE",
                    "NAME": "Current Input",
                    "TYPE": "T_STRING_V1",
                    "VALUE": "{}"
                }}],
                "PARAMETERS": {{
                    "HASHONLY": "FALSE",
                    "FLAT": "TRUE",
                    "HELPTEXT": "FALSE"
            }},
            {}}}"#,
            rng.gen::<u32>(),
            rng.gen::<u32>(),
            input.hashval,
            input.name,
            status!(Result::Success)
        ))
        .unwrap(),
    )
}

/// Get list of inputs command
pub fn list_inputs(device: SimulatedDevice) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "LIST INPUTS");
    let mut rng = rand::thread_rng();

    let mut items: Vec<String> = Vec::new();
    for input in device.inner.input_list.values() {
        items.push(format!(
            r#"
            {{
                "CNAME": "{}",
                "ENABLED": "FALSE",
                "HASHVAL": {},
                "NAME": "{}",
                "READONLY": "{}",
                "TYPE": "T_DEVICE_V1",
                "VALUE": {{
                    "METADATA": "",
                    "NAME": "{}"
                }}
            }}"#,
            input.cname, input.hashval, input.name, input.readonly, input.friendly
        ));
    }

    let items = items.join(",");

    warp::reply::json::<Value>(
        &serde_json::from_str(&format!(
            r#"
            {{
                "CNAME": "name_input",
                "GROUP": "G_DEVICES",
                "HASHLIST": [
                    {},
                    {},
                    {}
                ],
                "ITEMS": [{}],
                "PARAMETERS": {{
                    "HASHONLY": "FALSE",
                    "FLAT": "TRUE",
                    "HELPTEXT": "FALSE"
            }},
            {}}}
            "#,
            rng.gen::<u32>(),
            rng.gen::<u32>(),
            rng.gen::<u32>(),
            items,
            status!(Result::Success)
        ))
        .unwrap(),
    )
}

/// Change input command
pub fn change_input(mut val: Value, device: SimulatedDevice) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "CHANGE INPUT");
    let request = serde_json::from_value::<String>(val["REQUEST"].take()).unwrap();
    let name = serde_json::from_value::<String>(val["VALUE"].take());
    let hashval = serde_json::from_value::<u32>(val["HASHVAL"].take());

    let mut res = match (
        request.as_str(),
        name,
        hashval,
        device.inner.current_input.write(),
    ) {
        ("MODIFY", Ok(name), Ok(hashval), Ok(mut current_input)) => {
            if device.inner.input_list.get(&name).is_none() {
                status!(Result::InvalidParameter)
            } else if device
                .inner
                .input_list
                .get(&*current_input)
                .unwrap()
                .hashval
                != hashval
            {
                status!("Bad_Hashval")
            } else {
                *current_input = name;
                status!(Result::Success)
            }
        }
        (_, _, _, Err(_)) => status!(Result::Blocked),
        _ => status!(Result::InvalidParameter),
    };

    res.insert(0, '{');
    res.push('}');
    let res: Value = serde_json::from_str(&res).unwrap();

    warp::reply::json(&res)
}

/// Get device info command
pub fn device_info(device: SimulatedDevice) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "DEVICE INFO");
    let inputs: Vec<String> = device
        .inner
        .input_list
        .keys()
        .map(|x| format!("\"{}\"", x))
        .collect();

    let res = format!(
        r#"
            {{
                "ITEMS": [
                    {{
                        "VALUE": {{
                            "CAST_NAME": "{}",
                            "INPUTS": [{}],
                            "MODEL_NAME": "{}",
                            "SETTINGS_ROOT": "{}",
                            "SYSTEM_INFO": {{
                                "CHIPSET": 3,
                                "SERIAL_NUMBER": "1",
                                "VERSION": "1"
                            }}
                        }}
                    }}
                ],
                {}
            }}"#,
        device.inner.name,
        inputs.join(","),
        device.inner.model,
        device.inner.settings_root,
        status!(Result::Success),
    );
    let res: Value = serde_json::from_str(&res).unwrap();
    warp::reply::json(&res)
}

/// Read dynamic settings command
pub fn read_setting_dynamic(setting: Setting) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "READ DYNAMIC SETTINGS");
    warp::reply::json(&setting.dynamic_value())
}

/// Read static settings command
pub fn read_setting_static(setting: Setting) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "READ STATIC SETTINGS");
    warp::reply::json(&setting.static_value())
}

pub fn write_setting(mut val: Value, setting: Setting) -> warp::reply::Json {
    log::info!(target: "test::simulated_device::commands", "WRITE SETTINGS");
    let request = serde_json::from_value::<String>(val["REQUEST"].take());
    let hashval = serde_json::from_value::<u32>(val["HASHVAL"].take());
    let value = val["VALUE"].take();
    warp::reply::json(&json!(""))
}

// TODO:
// Get ESN command
// Get Serial No. command
// Get FW Version command
// Virtual remote commands
// Write settings command
// Get app list command
// Launch app command
