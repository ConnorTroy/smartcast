use super::{DeviceInfo, Input, SliderInfo, SubSetting};
use crate::error::{ApiError, Error, Result};

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug)]
pub(super) struct Response {
    pub value: Value,
}

impl Response {
    pub fn items<T>(&mut self) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value(
            self.value
                .get("ITEMS")
                .unwrap_or_else(|| panic!("'ITEMS' not found in response"))
                .clone(),
        )
        .map_err(|e| e.into())
    }

    pub fn first_item<T>(&mut self, key: Option<&str>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value({
            let item = match self.value.get("ITEM") {
                Some(item) => item.clone(),
                None => self.items::<Value>()?[0].take(),
            };

            if let Some(key) = key {
                item.get(key)
                    .ok_or_else(|| Error::Client("Key Not Found".into()))?
                    .clone()
            } else {
                item
            }
        })
        .map_err(|e| e.into())
    }

    pub fn pairing(mut self) -> Result<(u32, u32)> {
        Ok((
            self.first_item(Some("PAIRING_REQ_TOKEN"))?,
            self.first_item(Some("CHALLENGE_TYPE"))?,
        ))
    }

    pub fn auth_token(mut self) -> Result<String> {
        self.first_item(Some("AUTH_TOKEN"))
    }

    pub fn power_state(mut self) -> Result<bool> {
        Ok(self.first_item::<i32>(Some("VALUE"))? == 1)
    }

    pub fn device_info(mut self) -> Result<DeviceInfo> {
        self.first_item(Some("VALUE"))
    }

    pub fn current_input(mut self) -> Result<Input> {
        self.first_item(None)
    }

    pub fn input_list(mut self) -> Result<Vec<Input>> {
        self.items()
    }

    pub fn settings(mut self) -> Result<Vec<SubSetting>> {
        self.items()
    }

    pub fn slider_info(mut self) -> Option<SliderInfo> {
        self.first_item(None).ok()
    }

    pub fn elements(mut self) -> Result<Vec<String>> {
        self.first_item(Some("ELEMENTS"))
    }
}

impl From<Response> for Value {
    fn from(response: Response) -> Self {
        response.value
    }
}

impl From<Response> for Result<DeviceInfo> {
    fn from(response: Response) -> Self {
        response.device_info()
    }
}

impl From<Response> for Result<Input> {
    fn from(response: Response) -> Self {
        response.current_input()
    }
}

impl From<Response> for Result<Vec<Input>> {
    fn from(response: Response) -> Self {
        response.input_list()
    }
}

impl From<Response> for Result<Vec<SubSetting>> {
    fn from(response: Response) -> Self {
        response.settings()
    }
}

impl From<Response> for Option<SliderInfo> {
    fn from(response: Response) -> Self {
        response.slider_info()
    }
}

pub(super) fn process(response: String) -> Result<Response> {
    let response: Value = match serde_json::from_str(&response) {
        Ok(res) => res,
        Err(_) => return Err(ApiError::from(response).into()),
    };

    // Error Handling
    let result: String = response["STATUS"]["RESULT"]
        .to_string()
        .to_lowercase()
        .replace("\"", "");

    Err(match result.as_str() {
        // Command was successful so return the response
        "success" => return Ok(Response { value: response }),

        // Anything else is an error
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
        _ => format!(
            "Status Result: {} Detail: {}",
            response["STATUS"]["RESULT"].to_string(),
            response["STATUS"]["DETAIL"].to_string()
        )
        .into(),
    }
    .into())
}
