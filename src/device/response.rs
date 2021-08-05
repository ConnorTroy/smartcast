use super::{Error, Input, Result, SubSetting, setting::SliderInfo};

use serde_json::Value;

#[derive(Debug)]
pub struct Response {
    pub value: Value,
}

impl Response {
    pub fn pairing_token(&self) -> Result<u32> {
        Ok(serde_json::from_value(self.value["ITEM"]["PAIRING_REQ_TOKEN"].clone())?)
    }
    pub fn challenge(&self) -> Result<u32> {
        Ok(serde_json::from_value(self.value["ITEM"]["CHALLENGE_TYPE"].clone())?)
    }
    pub fn auth_token(&self) -> Result<String> {
        Ok(serde_json::from_value(self.value["ITEM"]["AUTH_TOKEN"].clone())?)
    }
    pub fn power_state(&self) -> Result<bool> {
        Ok(serde_json::from_value::<u32>(self.value["ITEMS"][0]["VALUE"].clone())? == 1)
    }
    pub fn device_info(&self) -> Result<Value> {
        Ok(self.value.clone()) // TODO: Device Info Struct
    }
    pub fn esn(&self) -> Result<String> {
        Ok(serde_json::from_value(self.value["ITEMS"][0]["VALUE"].clone())?)
    }
    pub fn serial(&self) -> Result<String> {
        Ok(serde_json::from_value(self.value["ITEMS"][0]["VALUE"].clone())?)
    }
    pub fn fw_version(&self) -> Result<String> {
        Ok(serde_json::from_value(self.value["ITEMS"][0]["VALUE"].clone())?)
    }
    pub fn current_input(&self) -> Result<Input> {
        Ok(serde_json::from_value(self.value["ITEMS"][0].clone())?)
    }
    pub fn input_list(&self) -> Result<Vec<Input>> {
        Ok(serde_json::from_value(self.value["ITEMS"].clone())?)
    }
    // Get Current App
    pub fn settings(&self) -> Result<Vec<SubSetting>> {
        Ok(serde_json::from_value(self.value["ITEMS"].clone())?)
    }
    pub fn slider_info(&self) -> Result<SliderInfo> {
        Ok(serde_json::from_value(self.value["ITEMS"][0].clone())?)
    }
    pub fn elements(&self) -> Result<Vec<String>> {
        Ok(serde_json::from_value(self.value["ITEMS"][0]["ELEMENTS"].clone())?)
    }
    // Write Settings
}

pub fn process(response: Value) -> Result<Response> {
    // TO-DO: handle bad request xml

    let result: String =
        response["STATUS"]["RESULT"]
        .to_string()
        .to_lowercase();

    // Remove quotes
    let result: &str = &result[1 .. result.len() - 1];

    // Error Handling
    match result {
        "success" => {},
        "invalid_parameter"             => return Err(Error::InvalidParameter),
        "uri_not_found"                 => return Err(Error::UriNotFound),
        "max_challenges_exceeded"       => return Err(Error::MaxChallengesExceeded),
        "pairing_denied"                => return Err(Error::PairingDenied),
        "value_out_of_range"            => return Err(Error::ValueOutOfRange),
        "challenge_incorrect"           => return Err(Error::ChallengeIncorrect),
        "blocked"                       => return Err(Error::Blocked),
        "failure"                       => return Err(Error::Failure),
        "aborted"                       => return Err(Error::Aborted),
        "busy"                          => return Err(Error::Busy),
        "requires_pairing"              => return Err(Error::RequiresPairing),
        "requires_system_pin"           => return Err(Error::RequiresSystemPin),
        "requires_new_system_pin"       => return Err(Error::RequiresNewSystemPin),
        "net_wifi_needs_valid_ssid"     => return Err(Error::NetWifiNeedsValidSSID),
        "net_wifi_already_connected"    => return Err(Error::NetWifiAlreadyConnected),
        "net_wifi_missing_password"     => return Err(Error::NetWifiMissingPassword),
        "net_wifi_not_existed"          => return Err(Error::NetWifiNotExisted),
        "net_wifi_auth_rejected"        => return Err(Error::NetWifiAuthRejected),
        "net_wifi_connect_timeout"      => return Err(Error::NetWifiConnectTimeout),
        "net_wifi_connect_aborted"      => return Err(Error::NetWifiConnectAborted),
        "net_wifi_connection_error"     => return Err(Error::NetWifiConnection),
        "net_ip_manual_config_error"    => return Err(Error::NetIPManualConfig),
        "net_ip_dhcp_failed"            => return Err(Error::NetIPDHCPFailed),
        "net_unknown_error"             => return Err(Error::NetUnknown),
        _ => {
            return Err(format!("Uncaught failure, could be an api bug.\nStatus Result: {}\nDetail: {}\n",
                response["STATUS"]["RESULT"].to_string(),
                response["STATUS"]["DETAIL"].to_string()
            ).into());
        },
    }

    // println!("{:#?}", response);

    Ok(Response{value: response})
}
