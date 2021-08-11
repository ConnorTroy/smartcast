use super::{DeviceInfo, Error, Input, Result, SliderInfo, SubSetting};

use serde_json::Value;

#[derive(Debug)]
pub struct Response {
    pub value: Value,
}

impl Response {
    pub fn pairing(mut self) -> Result<(u32, u32)> {
        Ok((
            serde_json::from_value(self.value["ITEM"]["PAIRING_REQ_TOKEN"].take())?,
            serde_json::from_value(self.value["ITEM"]["CHALLENGE_TYPE"].take())?,
        ))
    }

    pub fn auth_token(mut self) -> Result<String> {
        Ok(serde_json::from_value(
            self.value["ITEM"]["AUTH_TOKEN"].take(),
        )?)
    }

    pub fn power_state(mut self) -> Result<bool> {
        Ok(serde_json::from_value::<u32>(self.value["ITEMS"][0]["VALUE"].take())? == 1)
    }

    pub fn device_info(mut self) -> Result<DeviceInfo> {
        Ok(serde_json::from_value::<DeviceInfo>(self.value["ITEMS"][0]["VALUE"].take()).unwrap())
    }

    pub fn esn(mut self) -> Result<String> {
        Ok(serde_json::from_value(
            self.value["ITEMS"][0]["VALUE"].take(),
        )?)
    }

    pub fn serial(mut self) -> Result<String> {
        Ok(serde_json::from_value(
            self.value["ITEMS"][0]["VALUE"].take(),
        )?)
    }

    pub fn fw_version(mut self) -> Result<String> {
        Ok(serde_json::from_value(
            self.value["ITEMS"][0]["VALUE"].take(),
        )?)
    }

    pub fn current_input(mut self) -> Result<Input> {
        Ok(serde_json::from_value(self.value["ITEMS"][0].take())?)
    }

    pub fn input_list(mut self) -> Result<Vec<Input>> {
        Ok(serde_json::from_value(self.value["ITEMS"].take())?)
    }

    // TODO: Get Current App
    // pub fn current_app(mut self) -> Result<App> {

    // }

    pub fn settings(mut self) -> Result<Vec<SubSetting>> {
        Ok(serde_json::from_value(self.value["ITEMS"].take())?)
    }

    pub fn slider_info(mut self) -> Result<SliderInfo> {
        Ok(serde_json::from_value(self.value["ITEMS"][0].take())?)
    }

    pub fn elements(mut self) -> Result<Vec<String>> {
        Ok(serde_json::from_value(
            self.value["ITEMS"][0]["ELEMENTS"].take(),
        )?)
    }

    pub fn value(self) -> Result<Value> {
        Ok(self.value)
    }
    // Write Settings
}

pub fn process(response: String) -> Result<Response> {
    let response: Value = match serde_json::from_str(&response) {
        Ok(res) => res,
        Err(_) => return Err(Error::Other(response)),
    };

    let result: String = response["STATUS"]["RESULT"].to_string().to_lowercase();

    // Remove quotes
    let result: &str = &result[1..result.len() - 1];

    // Error Handling
    match result {
        "success" => {}
        "invalid_parameter" => return Err(Error::InvalidParameter),
        "uri_not_found" => return Err(Error::UriNotFound),
        "max_challenges_exceeded" => return Err(Error::MaxChallengesExceeded),
        "pairing_denied" => return Err(Error::PairingDenied),
        "value_out_of_range" => return Err(Error::ValueOutOfRange),
        "challenge_incorrect" => return Err(Error::ChallengeIncorrect),
        "blocked" => return Err(Error::Blocked),
        "failure" => return Err(Error::Failure),
        "aborted" => return Err(Error::Aborted),
        "busy" => return Err(Error::Busy),
        "requires_pairing" => return Err(Error::RequiresPairing),
        "requires_system_pin" => return Err(Error::RequiresSystemPin),
        "requires_new_system_pin" => return Err(Error::RequiresNewSystemPin),
        "net_wifi_needs_valid_ssid" => return Err(Error::NetWifiNeedsValidSSID),
        "net_wifi_already_connected" => return Err(Error::NetWifiAlreadyConnected),
        "net_wifi_missing_password" => return Err(Error::NetWifiMissingPassword),
        "net_wifi_not_existed" => return Err(Error::NetWifiNotExisted),
        "net_wifi_auth_rejected" => return Err(Error::NetWifiAuthRejected),
        "net_wifi_connect_timeout" => return Err(Error::NetWifiConnectTimeout),
        "net_wifi_connect_aborted" => return Err(Error::NetWifiConnectAborted),
        "net_wifi_connection_error" => return Err(Error::NetWifiConnection),
        "net_ip_manual_config_error" => return Err(Error::NetIPManualConfig),
        "net_ip_dhcp_failed" => return Err(Error::NetIPDHCPFailed),
        "net_unknown_error" => return Err(Error::NetUnknown),
        _ => {
            return Err(format!(
                "Uncaught failure, could be an api bug.\nStatus Result: {}\nDetail: {}\n",
                response["STATUS"]["RESULT"].to_string(),
                response["STATUS"]["DETAIL"].to_string()
            )
            .into());
        }
    }

    Ok(Response { value: response })
}
