use super::*;

pub fn device_info(device: Arc<EmulatedDevice>) -> warp::reply::Json {
    //TODO
    println!("INFO");
    let inputs: Vec<String> = device
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
        device.name,
        inputs.join(","),
        device.model,
        "tv_settings", // TODO
        status!(Result::Success),
    );
    let res: Value = serde_json::from_str(&res).unwrap();
    println!("{:#?}", res);
    warp::reply::json(&res)
}