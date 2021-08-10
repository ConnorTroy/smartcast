use super::*;

pub fn power_state(device: Arc<EmulatedDevice>) -> warp::reply::Json {
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
        *device.powered_on.read().unwrap() as u32,
        status!(Result::Success)
    );

    let res = serde_json::from_str(&res).unwrap();

    warp::reply::json::<Value>(&res)
}

pub fn current_input(device: Arc<EmulatedDevice>) -> warp::reply::Json {
    let input: &Input = device
        .input_list
        .get(&*device.current_input.read().unwrap())
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

pub fn list_inputs(device: Arc<EmulatedDevice>) -> warp::reply::Json {
    let mut rng = rand::thread_rng();

    let mut items: Vec<String> = Vec::new();
    for input in device.input_list.values() {
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

pub fn change_input(mut val: Value, device: Arc<EmulatedDevice>) -> warp::reply::Json {
    let request = serde_json::from_value::<String>(val["REQUEST"].take()).unwrap();
    let name = serde_json::from_value::<String>(val["VALUE"].take());
    let hashval = serde_json::from_value::<u32>(val["HASHVAL"].take());

    let mut res = match (
        request.as_str(),
        name,
        hashval,
        device.current_input.write(),
    ) {
        ("MODIFY", Ok(name), Ok(hashval), Ok(mut current_input)) => {
            if device.input_list.get(&name).is_none() {
                status!(Result::InvalidParameter)
            } else if device.input_list.get(&*current_input).unwrap().hashval != hashval {
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
