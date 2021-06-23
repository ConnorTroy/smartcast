use super::{Error, Result};

use serde_json::Value;

pub struct Response {

}

pub struct Info {
}

pub struct Status {
    result: String,
    detail: String,
}

pub fn process(res: String) -> Result<Option<Value>> {
    let mut response: Value = serde_json::from_str(&res).unwrap();
    // println!("{:#?}", response);

    let result: String =
        response["STATUS"]["RESULT"]
        .to_string()
        .to_lowercase();

    // Remove quotes
    let result: &str = &result[1 .. result.len() - 1];

    match result {
        "success" => {},
        "max_challenges_exceeded"       => Err(Error::MaxChallengesExceeded)?,
        "pairing_denied"                => Err(Error::PairingDenied)?,
        "value_out_of_range"            => Err(Error::ValueOutOfRange)?,
        "challenge_incorrect"           => Err(Error::ChallengeIncorrect)?,
        "blocked"                       => Err(Error::Blocked)?,
        "failure"                       => Err(Error::Failure)?,
        "aborted"                       => Err(Error::Aborted)?,
        "busy"                          => Err(Error::Busy)?,
        "requires_pairing"              => Err(Error::RequiresPairing)?,
        "requires_system_pin"           => Err(Error::RequiresSystemPin)?,
        "requires_new_system_pin"       => Err(Error::RequiresNewSystemPin)?,
        "net_wifi_needs_valid_ssid"     => Err(Error::NetWifiNeedsValidSSID)?,
        "net_wifi_already_connected"    => Err(Error::NetWifiAlreadyConnected)?,
        "net_wifi_missing_password"     => Err(Error::NetWifiMissingPassword)?,
        "net_wifi_not_existed"          => Err(Error::NetWifiNotExisted)?,
        "net_wifi_auth_rejected"        => Err(Error::NetWifiAuthRejected)?,
        "net_wifi_connect_timeout"      => Err(Error::NetWifiConnectTimeout)?,
        "net_wifi_connect_aborted"      => Err(Error::NetWifiConnectAborted)?,
        "net_wifi_connection_error"     => Err(Error::NetWifiConnection)?,
        "net_ip_manual_config_error"    => Err(Error::NetIPManualConfig)?,
        "net_ip_dhcp_failed"            => Err(Error::NetIPDHCPFailed)?,
        "net_unknown_error"             => Err(Error::NetUnknown)?,
        _ => {
            Err(format!("Uncaught failure, could be an api bug.\nStatus Result: {}\nDetail: {}\n",
                response["STATUS"]["RESULT"].to_string(),
                response["STATUS"]["DETAIL"].to_string()
            ))?;
        },
    }

    let item = response["ITEM"].take();

    println!("{:#?}", item);
    Ok(Some(item))
}
