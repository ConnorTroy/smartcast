use super::EndpointBase;

use super::{response, ButtonEvent, Device, Response, Result};

use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_json::Value;

use std::result::Result as StdResult;

#[derive(Debug, Copy, Clone)]
pub enum RequestType {
    Get,
    Put,
}

#[allow(unused)] // Temp - TODO: remove
#[derive(Debug)]
pub enum CommandDetail {
    StartPairing {
        client_name: String,
        client_id: String,
    },
    FinishPairing {
        client_id: String,
        pairing_token: u32,
        challenge: u32,
        response_value: String,
    },
    CancelPairing {
        client_id: String,
        pairing_token: u32,
        challenge: u32,
    },
    GetPowerState,
    GetDeviceInfo,
    RemoteButtonPress(Vec<ButtonEvent>),
    GetCurrentInput,
    GetInputList,
    ChangeInput {
        name: String,
        hashval: u32,
    },
    GetCurrentApp,
    LaunchApp(Value),
    ReadSettings(EndpointBase, String),
    // WriteSettings, // TODO (Brick warning)
    Custom(RequestType, String, Option<Value>),
}

impl CommandDetail {
    /// Get the endpoint of the command
    pub fn endpoint(&self, settings_root: String) -> String {
        match self {
            Self::StartPairing { .. } => "/pairing/start".into(),
            Self::FinishPairing { .. } => "/pairing/pair".into(),
            Self::CancelPairing { .. } => "/pairing/cancel".into(),
            Self::GetPowerState => "/state/device/power_mode".into(),
            Self::GetDeviceInfo => "/state/device/deviceinfo".into(),
            Self::RemoteButtonPress { .. } => "/key_command/".into(),
            Self::GetCurrentInput => format!(
                "/menu_native/dynamic/{}/devices/current_input",
                settings_root
            ),
            Self::GetInputList => {
                format!("/menu_native/dynamic/{}/devices/name_input", settings_root)
            }
            Self::ChangeInput { .. } => format!(
                "/menu_native/dynamic/{}/devices/current_input",
                settings_root
            ),
            Self::GetCurrentApp => "/app/current".into(),
            Self::LaunchApp(_) => "/app/launch".into(),
            Self::ReadSettings(base, endpoint) => base.as_str() + endpoint,
            // Self::WriteSettings                 => "/menu_native/dynamic/tv_settings/SETTINGS_CNAME/ITEMS_CNAME",
            Self::Custom(_, endpoint, _) => endpoint.into(),
        }
    }

    /// Get the request type of the command
    pub fn request_type(&self) -> RequestType {
        match self {
            Self::StartPairing { .. }
            | Self::FinishPairing { .. }
            | Self::CancelPairing { .. }
            | Self::RemoteButtonPress { .. }
            | Self::ChangeInput { .. }
            | Self::LaunchApp(_) => RequestType::Put,
            // Self::WriteSettings     => RequestType::Put,
            Self::GetPowerState
            | Self::GetDeviceInfo
            | Self::GetCurrentInput
            | Self::GetInputList
            | Self::GetCurrentApp
            | Self::ReadSettings(_, _) => RequestType::Get,
            Self::Custom(req_type, _, _) => *req_type,
        }
    }
}

pub struct Command {
    detail: CommandDetail,
    endpoint: String,
    device: Device,
}

impl Command {
    pub fn new(device: Device, detail: CommandDetail) -> Self {
        let endpoint = detail.endpoint(device.settings_root());
        Self {
            detail,
            endpoint,
            device,
        }
    }

    pub async fn send(self) -> Result<Response> {
        let device = self.device.clone();
        let client = device.inner.client.clone();

        let url: String = format!(
            "https://{}:{}{}",
            device.ip(),
            device.port(),
            self.detail.endpoint(device.settings_root())
        );

        let res = {
            // Request building
            let mut req = match self.detail.request_type() {
                RequestType::Get => client.get(url),
                RequestType::Put => {
                    client
                        .put(url)
                        // Add content type header
                        .header("Content-Type", "application/json")
                        // Add body for PUT commands
                        .body(serde_json::to_string(&self).unwrap())
                }
            };
            // Add auth token header
            if let Some(token) = &device.auth_token().await {
                req = req.header("Auth", token.to_string())
            }
            log::debug!("req: {:?}", req);

            req
        }
        // Request send
        .send()
        .await?
        // Get response as text because some device errors do not follow the standard format
        .text()
        .await?;

        // Process response
        response::process(res)
    }
}

impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut command = serializer.serialize_struct("", 5)?;
        command.serialize_field("_url", &self.endpoint)?;
        match &self.detail {
            CommandDetail::StartPairing {
                client_name,
                client_id,
            } => {
                command.serialize_field("DEVICE_NAME", client_name)?;
                command.serialize_field("DEVICE_ID", client_id)?;
                command.end()
            }
            CommandDetail::CancelPairing {
                client_id,
                pairing_token,
                challenge,
            } => {
                command.serialize_field("DEVICE_ID", client_id)?;
                command.serialize_field("CHALLENGE_TYPE", challenge)?;
                command.serialize_field("RESPONSE_VALUE", "1111")?;
                command.serialize_field("PAIRING_REQ_TOKEN", pairing_token)?;
                command.end()
            }
            CommandDetail::FinishPairing {
                client_id,
                pairing_token,
                challenge,
                response_value,
            } => {
                command.serialize_field("DEVICE_ID", client_id)?;
                command.serialize_field("CHALLENGE_TYPE", challenge)?;
                command.serialize_field("RESPONSE_VALUE", response_value)?;
                command.serialize_field("PAIRING_REQ_TOKEN", pairing_token)?;
                command.end()
            }
            CommandDetail::RemoteButtonPress(button_event_vec) => {
                command.serialize_field("KEYLIST", button_event_vec)?;
                command.end()
            }
            CommandDetail::ChangeInput { name, hashval } => {
                command.serialize_field("REQUEST", "MODIFY")?;
                command.serialize_field("VALUE", name)?;
                command.serialize_field("HASHVAL", hashval)?;
                command.end()
            }
            CommandDetail::LaunchApp(payload) => {
                command.serialize_field("VALUE", payload)?;
                command.end()
            }
            // TODO:
            // CommandDetail::WriteSettings => {
            //     let mut command = serializer.serialize_struct("", )?;
            //     command.serialize_field("", )?;
            //     command.serialize_field("", )?;
            //     command.end()
            // },
            _ => command.end(),
        }
    }
}
