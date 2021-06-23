use super::{Error, Result, Device};

use reqwest::{Client, Response};
use serde_json::{json, Value};


pub enum CommandField {
    DeviceName(String),
    DeviceId(String),
    ChallengeType(u32),
    ResponseValue(String),
    PairingReqToken(u32),
    AuthToken(String),
    Event(RemoteButton),
    Action(Action),
}

pub enum RemoteButton {
    VolumeDown,
    VolumeUp,
    MuteOff,
    MuteOn,
    MuteToggle,
    CycleInput,
    ChannelDown,
    ChannelUp,
    PreviousCh,
    PowerOff,
    PowerOn,
    PowerToggle,
}

pub enum Action {
    KeyDown,
    KeyUp,
    KeyPress,
}

pub enum ResponseField {
    Name(String),
    CName(String),
    Type(String),
    Value(String),
    Enabled(bool),
    HashVal(u32),
    // HashList([])
    Group()

}

pub enum Command {
    StartPairing{client_name: String, client_id: String},
    FinishPairing{client_id: String, pairing_token: u32, challenge: u32, response_value: String},
    CancelPairing{client_name: String, client_id: String},

    GetPowerState,
    RemoteButtonPress{button: RemoteButton, action: Action},

    GetCurrentInput,
    GetInputList,
    ChangeInput,
    LaunchApp,
    ReadSettings,
    // WriteSettings, // Todo (Brick warning)
}
#[derive(Debug)]
pub enum RequestType {
    Get,
    Put,
}

impl Command {

    /// Get the endpoint of the command
    pub fn endpoint(&self) -> String {
        match self {
            Self::StartPairing{..}          => "/pairing/start",
            Self::FinishPairing{..}         => "/pairing/pair",
            Self::CancelPairing{..}         => "/pairing/cancel",
            Self::GetPowerState             => "/state/device/power_mode",
            Self::RemoteButtonPress{..}     => "/key_command/",
            Self::GetCurrentInput           => "/menu_native/dynamic/tv_settings/devices/current_input",
            Self::GetInputList              => "/menu_native/dynamic/tv_settings/devices/name_input",
            Self::ChangeInput               => "/menu_native/dynamic/tv_settings/devices/current_input",
            Self::LaunchApp                 => "/app/launch",
            Self::ReadSettings              => "/menu_native/dynamic/tv_settings/SETTINGS_CNAME",
            // Self::WriteSettings             => "/menu_native/dynamic/tv_settings/SETTINGS_CNAME/ITEMS_CNAME",
        }.to_string()
    }

    /// Get the request type of the command
    pub fn request_type(&self) -> RequestType {
        match self {
            Self::StartPairing{..}          => RequestType::Put,
            Self::FinishPairing{..}         => RequestType::Put,
            Self::CancelPairing{..}         => RequestType::Put,
            Self::GetPowerState             => RequestType::Get,
            Self::RemoteButtonPress{..}     => RequestType::Put,
            Self::GetCurrentInput           => RequestType::Get,
            Self::GetInputList              => RequestType::Get,
            Self::ChangeInput               => RequestType::Put,
            Self::LaunchApp                 => RequestType::Put,
            Self::ReadSettings              => RequestType::Get,
            // Self::WriteSettings             => RequestType::Put,
        }
    }

    pub fn body(&self) -> Option<Value> {
        match self {
            Self::StartPairing{client_name, client_id}
            | Self::CancelPairing{client_name, client_id} => {
                Some(json!({
                    "DEVICE_NAME": client_name,
                    "DEVICE_ID": client_id,
                }))
            },
            Self::FinishPairing{client_id, pairing_token, challenge, response_value} => {
                Some(json!({
                    "DEVICE_ID": client_id,
                    "CHALLENGE_TYPE": challenge,
                    "RESPONSE_VALUE": response_value,
                    "PAIRING_REQ_TOKEN": pairing_token,
                }))
            },
            // Self::GetPowerState             => None,
            Self::RemoteButtonPress{..}    => None,
            // Self::GetCurrentInput           => None,
            // Self::GetInputList              => None,
            // Self::ChangeInput               => None,
            // Self::LaunchApp                 => None,
            // Self::ReadSettings              => None,
            // Self::WriteSettings             => None,
            _ => None
        }
    }
}
