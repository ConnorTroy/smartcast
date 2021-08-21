use super::{DeviceInfo, Input, SliderInfo, SubSetting};
use crate::error::{ApiError, Result};

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

    // pub fn value(self) -> Result<Value> {
    //     Ok(self.value)
    // }
    // Write Settings
}

pub fn process(response: String) -> Result<Response> {
    let response: Value = match serde_json::from_str(&response) {
        Ok(res) => res,
        Err(_) => return Err(ApiError::Unknown(response).into()),
    };

    let result: String = response["STATUS"]["RESULT"].to_string().to_lowercase();

    // Remove quotes
    let result: &str = &result[1..result.len() - 1];

    // Error Handling
    Err(match result {
        "success" => return Ok(Response { value: response }),
        "invalid_parameter" => ApiError::InvalidParameter,
        "uri_not_found" => ApiError::UriNotFound,
        "max_challenges_exceeded" => ApiError::MaxChallengesExceeded,
        "pairing_denied" => ApiError::PairingDenied,
        "value_out_of_range" => ApiError::ValueOutOfRange,
        "challenge_incorrect" => ApiError::ChallengeIncorrect,
        "blocked" => ApiError::Blocked,
        "failure" => ApiError::Failure,
        "aborted" => ApiError::Aborted,
        "busy" => ApiError::Busy,
        "requires_pairing" => ApiError::RequiresPairing,
        "requires_system_pin" => ApiError::RequiresSystemPin,
        "requires_new_system_pin" => ApiError::RequiresNewSystemPin,
        "net_wifi_needs_valid_ssid" => ApiError::NetWifiNeedsValidSSID,
        "net_wifi_already_connected" => ApiError::NetWifiAlreadyConnected,
        "net_wifi_missing_password" => ApiError::NetWifiMissingPassword,
        "net_wifi_not_existed" => ApiError::NetWifiNotExisted,
        "net_wifi_auth_rejected" => ApiError::NetWifiAuthRejected,
        "net_wifi_connect_timeout" => ApiError::NetWifiConnectTimeout,
        "net_wifi_connect_aborted" => ApiError::NetWifiConnectAborted,
        "net_wifi_connection_error" => ApiError::NetWifiConnection,
        "net_ip_manual_config_error" => ApiError::NetIPManualConfig,
        "net_ip_dhcp_failed" => ApiError::NetIPDHCPFailed,
        "net_unknown_error" => ApiError::NetUnknown,
        _ => {
            return Err(format!(
                "Status Result: {} Detail: {}",
                response["STATUS"]["RESULT"].to_string(),
                response["STATUS"]["DETAIL"].to_string()
            )
            .into());
        }
    }
    .into())
}
