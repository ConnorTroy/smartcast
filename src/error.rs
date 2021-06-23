/// Result for API calls from [`Device`](super::Device)
pub type Result<T> = std::result::Result<T, Error>;

/// Error for API calls from [`Device`](super::Device)
#[derive(Debug)]
pub enum Error {
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
    /// Error from http client
    Reqwest(reqwest::Error),
    /// Error from std::io
    StdIO(std::io::Error),
    #[doc(hidden)]
    /// Error processing json command
    // Json(serde_json::Error),
    Other(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::Reqwest(e)
    }
}

// impl From<serde_json::Error> for Error {
//     fn from(e: serde_json::Error) -> Error {
//         Error::Json(e)
//     }
// }

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::StdIO(e)
    }
}

impl From<std::string::String> for Error {
    fn from(e: std::string::String) -> Error {
        Error::Other(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::MaxChallengesExceeded => {
                write!(f, "Too many failed pair attempts")
            },
            Error::PairingDenied => {
                write!(f, "Incorrect pin")
            },
            Error::ValueOutOfRange => {
                write!(f, "Pin out of range")
            },
            Error::ChallengeIncorrect => {
                write!(f, "Incorrect challenge")
            },
            Error::Blocked => {
                write!(f, "Pairing is already in progress")
            },
            Error::Failure => {
                write!(f, "Unknown command failure")
            },
            Error::Aborted => {
                write!(f, "Unknown abort")
            },
            Error::Busy => {
                write!(f, "Device is busy")
            },
            Error::RequiresPairing => {
                write!(f, "Device requires pairing")
            },
            Error::RequiresSystemPin => {
                write!(f, "Device requires system pin")
            },
            Error::RequiresNewSystemPin => {
                write!(f, "Device requires new system pin")
            },
            Error::NetWifiNeedsValidSSID => {
                write!(f, "Wifi needs SSID")
            },
            Error::NetWifiAlreadyConnected => {
                write!(f, "Wifi already connected")
            },
            Error::NetWifiMissingPassword => {
                write!(f, "Wifi needs password")
            },
            Error::NetWifiNotExisted => {
                write!(f, "Wifi network does not exist")
            },
            Error::NetWifiAuthRejected => {
                write!(f, "Wifi authentication rejected")
            },
            Error::NetWifiConnectTimeout => {
                write!(f, "Wifi connection timeout")
            },
            Error::NetWifiConnectAborted => {
                write!(f, "Wifi connection aborted")
            },
            Error::NetWifiConnection => {
                write!(f, "Wifi connection error")
            },
            Error::NetIPManualConfig => {
                write!(f, "IP config error")
            },
            Error::NetIPDHCPFailed => {
                write!(f, "DHCP failure")
            },
            Error::NetUnknown => {
                write!(f, "Unknown Network Error")
            },
            Error::Reqwest(_)// | Error::Json(_)
            | Error::StdIO(_) | Error::Other(_)=> {
                write!(f, "{}", self)
            }
        }
    }
}
