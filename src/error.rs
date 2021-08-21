use std::fmt::{Debug, Display};

use serde_json::Value;

/// Result for API calls from [`Device`](super::Device)
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Errors from the SmartCast device
    Api(ApiError),
    /// Errors from ['Device'](super::Device)
    Client(ClientError),
    /// Error from http client
    Reqwest(reqwest::Error),
    /// Error from std::io
    IO(std::io::Error),
    /// Error processing json command
    Json(serde_json::Error),
    #[doc(hidden)]
    Other(String),
}

impl Error {
    pub fn is_api(&self) -> bool {
        matches!(self, Error::Api(_))
    }

    pub fn is_client(&self) -> bool {
        matches!(self, Error::Client(_))
    }

    pub fn is_reqwest(&self) -> bool {
        matches!(self, Error::Reqwest(_))
    }

    pub fn is_serde(&self) -> bool {
        matches!(self, Error::Json(_))
    }

    pub fn is_io(&self) -> bool {
        matches!(self, Error::IO(_))
    }

    pub fn device_not_found_ip(ip_addr: String) -> Error {
        ClientError::DeviceNotFoundIP(ip_addr).into()
    }

    pub fn device_not_found_uuid(uuid: String) -> Error {
        ClientError::DeviceNotFoundUUID(uuid).into()
    }

    pub fn setting_type_bad_match(current_value: Value, new_value: Value) -> Error {
        ClientError::WriteSettingsBadType(current_value, new_value).into()
    }

    pub fn setting_outside_bounds(min: i32, max: i32, new_value: i32) -> Error {
        ClientError::WriteSettingsOutsideBounds(min, max, new_value).into()
    }
}

impl From<ApiError> for Error {
    fn from(e: ApiError) -> Self {
        Error::Api(e)
    }
}

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        Error::Client(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::Reqwest(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::Other(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(e) => write!(f, "{}", e),
            Self::Client(e) => write!(f, "{}", e),
            Self::Reqwest(e) => write!(f, "{}", e),
            Self::IO(e) => write!(f, "{}", e),
            Self::Json(e) => write!(f, "{}", e),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}

/// Errors from the SmartCast device
#[derive(Debug)]
pub enum ApiError {
    /// Invalid Parameter - probably means this api needs to be modified to work with your firmware
    InvalidParameter,
    /// URI not found - probably means this api needs to be modified to work with your firmware
    UriNotFound,
    /// Pairing: Too many failed pair attempts
    MaxChallengesExceeded,
    /// Pairing: Incorrect pin
    PairingDenied,
    /// Pairing: Pin out of range
    ValueOutOfRange,
    /// Pairing: Incorrect challenge
    ChallengeIncorrect,
    /// Pairing: is already in progress
    Blocked,
    /// Unknown command failure
    Failure,
    /// Unknown abort
    Aborted,
    /// Device is busy
    Busy,
    /// Device requires pairing
    RequiresPairing,
    /// Device requires system pin
    RequiresSystemPin,
    /// Device requires new system pin
    RequiresNewSystemPin,
    /// Wifi needs SSID
    NetWifiNeedsValidSSID,
    /// Wifi already connected
    NetWifiAlreadyConnected,
    /// Wifi needs password
    NetWifiMissingPassword,
    /// Wifi network does not exist
    NetWifiNotExisted,
    /// Wifi authentication rejected
    NetWifiAuthRejected,
    /// Wifi connection timeout
    NetWifiConnectTimeout,
    /// Wifi connection aborted
    NetWifiConnectAborted,
    /// Wifi connection error
    NetWifiConnection,
    /// IP config error
    NetIPManualConfig,
    /// DHCP failure
    NetIPDHCPFailed,
    /// Unknown Network Error
    NetUnknown,
    #[doc(hidden)]
    Unknown(String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidParameter => write!(f, "Invalid Parameter"),
            Self::UriNotFound => write!(f, "URI not found"),
            Self::MaxChallengesExceeded => write!(f, "Too many failed pair attempts"),
            Self::PairingDenied => write!(f, "Incorrect pin"),
            Self::ValueOutOfRange => write!(f, "Pin out of range"),
            Self::ChallengeIncorrect => write!(f, "Incorrect challenge"),
            Self::Blocked => write!(f, "Command was blocked"),
            Self::Failure => write!(f, "Unknown command failure"),
            Self::Aborted => write!(f, "Unknown abort"),
            Self::Busy => write!(f, "Device is busy"),
            Self::RequiresPairing => write!(f, "Device requires pairing"),
            Self::RequiresSystemPin => write!(f, "Device requires system pin"),
            Self::RequiresNewSystemPin => write!(f, "Device requires new system pin"),
            Self::NetWifiNeedsValidSSID => write!(f, "Wifi needs SSID"),
            Self::NetWifiAlreadyConnected => write!(f, "Wifi already connected"),
            Self::NetWifiMissingPassword => write!(f, "Wifi needs password"),
            Self::NetWifiNotExisted => write!(f, "Wifi network does not exist"),
            Self::NetWifiAuthRejected => write!(f, "Wifi authentication rejected"),
            Self::NetWifiConnectTimeout => write!(f, "Wifi connection timeout"),
            Self::NetWifiConnectAborted => write!(f, "Wifi connection aborted"),
            Self::NetWifiConnection => write!(f, "Wifi connection error"),
            Self::NetIPManualConfig => write!(f, "IP config error"),
            Self::NetIPDHCPFailed => write!(f, "DHCP failure"),
            Self::NetUnknown => write!(f, "Unknown network Error"),
            Self::Unknown(e) => write!(f, "Unknown error: '{}'", e),
        }
    }
}

impl From<String> for ApiError {
    fn from(e: String) -> ApiError {
        ApiError::Unknown(e)
    }
}

#[derive(Debug)]
pub enum ClientError {
    /// Could not find device by IP
    DeviceNotFoundIP(String),
    /// Could not find device by UUID
    DeviceNotFoundUUID(String),
    /// New settings value type does not match current
    WriteSettingsBadType(Value, Value),
    /// New settings value is outside the bounds of the slider
    WriteSettingsOutsideBounds(i32, i32, i32),
    /// Attempted to write a read only setting
    WriteSettingsReadOnly,
    #[doc(hidden)]
    Message(String),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::DeviceNotFoundIP(ip) => {
                write!(f, "Could not connect to SmartCast device with ip: '{}'", ip)
            }

            Self::DeviceNotFoundUUID(uuid) => write!(
                f,
                "Could not connect to SmartCast device with uuid: '{}'",
                uuid
            ),

            Self::WriteSettingsBadType(current, new) => write!(
                f,
                "New value type [{:?}] does not match current [{:?}]",
                new, current
            ),

            Self::WriteSettingsOutsideBounds(min, max, new_val) => write!(
                f,
                "New value is outside bounds: [{} <= x <= {}] New value: {}",
                min, max, new_val
            ),

            Self::WriteSettingsReadOnly => {
                write!(f, "Attempted to write a menu or read only setting")
            }

            Self::Message(msg) => write!(f, "{}", msg),
        }
    }
}
