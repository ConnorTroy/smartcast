use serde::{de, Deserialize, Deserializer};

#[derive(Debug)]
/// Various infomation about the device returned by [`device_info()`](super::Device::device_info)
pub struct DeviceInfo {
    /// Device name as known by Google Cast
    pub cast_name: String,
    /// Physical inputs on the device
    pub inputs: Vec<String>,
    /// Device's model name
    pub model_name: String,
    /// Device's serial number
    pub serial_number: String,
    /// Device's firmware version
    pub fw_version: String,
    /// URI of root settings
    pub(super) settings_root: String,
    /// Device's chipset version
    pub(super) chipset: u32,
}

impl<'de> Deserialize<'de> for DeviceInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        struct Value {
            cast_name: String,
            inputs: Vec<String>,
            model_name: String,
            settings_root: String,
            system_info: SystemInfo,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        struct SystemInfo {
            chipset: u32,
            serial_number: String,
            #[serde(rename = "VERSION")]
            fw_version: String,
        }

        let helper = Value::deserialize(deserializer)?;

        Ok(DeviceInfo {
            cast_name: helper.cast_name,
            inputs: helper.inputs,
            model_name: helper.model_name,
            settings_root: helper.settings_root,
            chipset: helper.system_info.chipset,
            serial_number: helper.system_info.serial_number,
            fw_version: helper.system_info.fw_version,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Input on the device
pub struct Input {
    name: String,
    #[serde(rename(deserialize = "VALUE"))]
    #[serde(deserialize_with = "parse_input_friendly")]
    friendly_name: String,
    hashval: u32,
}

impl Input {
    /// Input's name
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Input's "friendly" name
    pub fn friendly_name(&self) -> String {
        self.friendly_name.clone()
    }

    pub(super) fn hashval(&self) -> u32 {
        self.hashval
    }
}

fn parse_input_friendly<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let mut value = serde_json::Value::deserialize(deserializer)?;
    serde_json::from_value::<String>(value.clone()).or_else(|_| {
        serde_json::from_value::<String>(value["NAME"].take())
            .map_err(|_| de::Error::missing_field("NAME"))
    })
}
