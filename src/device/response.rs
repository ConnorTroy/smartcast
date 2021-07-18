use super::{Error, Result};

use serde_json::Value;

pub fn process(mut response: Value) -> Result<Option<Value>> {
    // TO-DO: handle bad request xml

    let result: String =
        response["STATUS"]["RESULT"]
        .to_string()
        .to_lowercase();

    // Remove quotes
    let result: &str = &result[1 .. result.len() - 1];

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

    // TO-DO: do this better.
    let item: Option<Value> = match (&response["ITEM"], &response["ITEMS"]) {
        (Value::Null, Value::Null) => None,
        (Value::Object(_), Value::Null) => Some(response["ITEM"].take()),
        (Value::Null, Value::Array(_)) => Some(response["ITEMS"].take()),
        _ => panic!("Unexpected response json")
    };

    Ok(item)
}

#[derive(Debug, Clone)]
pub struct Input {
    name: String,
    friendly_name: String,
    hashval: u32,
}

impl Input {
    fn new(name: String, friendly_name: String, hashval: u32) -> Self {
        Self {
            name,
            friendly_name,
            hashval,
        }
    }

    pub(crate) fn from_value(input_value: &mut Value) -> Self {
        // "NAME"
        let name: String = serde_json::from_value(input_value["NAME"].take()).unwrap();

        // "VALUE" is the friendly name for current input or object containing friendly for list of inputs
        let friendly_name: String =
            serde_json::from_value::<String>(input_value["VALUE"].clone())
            .unwrap_or_else(|_|
                serde_json::from_value::<String>(input_value["VALUE"]["NAME"].take()).unwrap()
            );

        // "HASHVAL"
        let hashval: u32 = serde_json::from_value(input_value["HASHVAL"].take()).unwrap();

        Self::new(name, friendly_name, hashval)
    }

    pub(crate) fn from_array(json_value: &mut Value) -> Vec<Self> {
        let mut input_vec: Vec<Self> = Vec::new();

        for input_value in json_value.as_array_mut().unwrap() {
            let input = Self::from_value(input_value);
            input_vec.push(input);
        }

        input_vec
    }

    pub(crate) fn hashval(&self) -> u32 {
        self.hashval
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn friendly_name(&self) -> String {
        self.friendly_name.clone()
    }
}
