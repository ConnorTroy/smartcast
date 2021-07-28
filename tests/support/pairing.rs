use super::*;

pub fn pair_start(mut val: Value, device: Arc<VizioDevice>) -> warp::reply::Json {
    let client_id = serde_json::from_value::<String>(val["DEVICE_ID"].take());
    let client_name = serde_json::from_value::<String>(val["DEVICE_NAME"].take());

    let mut res: String = match (client_id, client_name, device.state.write()) {
        (Ok(client_id), Ok(client_name), Ok(mut state))
        if *state == State::Ready => {
            let mut rng = rand::thread_rng();
            let challenge: u32 = 1;
            let pair_token: u32 = rng.gen();
            *state = State::Pairing{challenge, pair_token, client_id, client_name};
            format!(r#"
                "ITEM": {{
                    "PAIRING_REQ_TOKEN": {},
                    "CHALLENGE_TYPE": {}
                }},
                {}
            "#, pair_token, challenge, status!(Result::Success))
        },
        (_, _, Ok(state)) if *state != State::Ready
                        => status!(Result::Blocked),
        (_, _, Err(_))  => status!(Result::Blocked),
        _               => status!(Result::InvalidParameter),
    };

    res.insert(0, '{');
    res.push('}');
    let res: Value = serde_json::from_str( &res ).unwrap();

    warp::reply::json(&res)
}

pub fn pair_finish(mut val: Value, device: Arc<VizioDevice>) -> warp::reply::Json {
    let client_id = serde_json::from_value::<String>(val["DEVICE_ID"].take());
    let challenge = serde_json::from_value::<u32>(val["CHALLENGE_TYPE"].take());
    let pin = serde_json::from_value::<String>(val["RESPONSE_VALUE"].take());
    let pair_token = serde_json::from_value::<u32>(val["PAIRING_REQ_TOKEN"].take());

    let mut res: String = match (client_id, challenge, pin, pair_token, device.state.write()) {
        (Ok(client_id), Ok(challenge), Ok(_), Ok(pair_token), Ok(mut state)) => {
            match &*state {
                State::Pairing {
                    challenge: exp_challenge,
                    pair_token: exp_pair,
                    client_name: _ ,
                    client_id: exp_id} => {
                    if challenge != *exp_challenge {
                        status!(Result::ChallengeIncorrect)
                    } else if client_id != *exp_id || pair_token != *exp_pair {
                        status!(Result::InvalidParameter)
                    } else {
                        *state = State::Ready;
                        format!(r#"
                            "ITEM": {{
                                "AUTH_TOKEN": "{}"
                            }},
                            {}
                        "#, 0, status!(Result::Success))
                    }
                },
                _ => status!(Result::Blocked),
            }
        },
        (_, _, _, _, Err(_)) => status!(Result::Blocked),
        _ => status!(Result::InvalidParameter),
    };

    res.insert(0, '{');
    res.push('}');
    let res: Value = serde_json::from_str( &res ).unwrap();

    warp::reply::json(&res)
}

pub fn pair_cancel(mut val: Value, device: Arc<VizioDevice>) -> warp::reply::Json {
    let client_id = serde_json::from_value::<String>(val["DEVICE_ID"].take());
    let challenge = serde_json::from_value::<u32>(val["CHALLENGE_TYPE"].take());
    let pin = serde_json::from_value::<String>(val["RESPONSE_VALUE"].take());
    let pair_token = serde_json::from_value::<u32>(val["PAIRING_REQ_TOKEN"].take());

    let mut res: String = match (client_id, challenge, pin, pair_token, device.state.write()) {
        (Ok(client_id), Ok(challenge), Ok(pin), Ok(pair_token), Ok(mut state))
        if *state != State::Ready => {
            match &*state {
                State::Pairing {
                    challenge: exp_challenge,
                    pair_token: exp_pair,
                    client_name: _ ,
                    client_id: exp_id} => {
                    if challenge != *exp_challenge
                    || client_id != *exp_id
                    || pin != "1111"
                    || pair_token != *exp_pair {
                        status!(Result::InvalidParameter)
                    } else {
                        *state = State::Ready;
                        format!(r#"
                            "ITEM": {{}},
                            {}
                        "#, status!(Result::Success))
                    }
                },
                _ => status!(Result::Blocked),
            }
        },
        (_, _, _, _, Ok(state)) if *state == State::Ready
                        => status!(Result::Blocked),
        (_, _, _, _, Err(_))  => status!(Result::Blocked),
        _               => status!(Result::InvalidParameter),
    };

    res.insert(0, '{');
    res.push('}');
    let res: Value = serde_json::from_str( &res ).unwrap();

    warp::reply::json(&res)
}
