use super::{CommandDetail, Device, Response, Result};

use serde::{de, Deserialize};
use serde_json::Value;

use std::result::Result as StdResult;

pub trait SettingValue<T> {
    fn value(&self) -> Result<T>;
}

#[derive(Debug, Clone)]
pub enum EndpointBase {
    Static,
    Dynamic,
}

impl EndpointBase {
    pub fn as_str(&self) -> String {
        String::from("/menu_native")
            + match self {
                Self::Static => "/static",
                Self::Dynamic => "/dynamic",
            }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    /// Slider which has a max/min value. See [`SliderInfo`] for more details.
    Slider,
    /// List of items which should be displayed. Use [elements()]() to get list data.
    List,
    /// Mutable value
    Value,
    /// Menu containing more [`SubSetting`]s
    Menu,
    /// List of items which should be displayed. Use [elements()]() to get list data.
    XList,
    #[doc(hidden)]
    Other(String),
}

/// Deserializer for [`ObjectType`]
impl<'de> Deserialize<'de> for ObjectType {
    fn deserialize<D>(deserializer: D) -> StdResult<ObjectType, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "T_VALUE_ABS_V1" => ObjectType::Slider,
            "T_LIST_V1" => ObjectType::List,
            "T_VALUE_V1" => ObjectType::Value,
            "T_MENU_V1" => ObjectType::Menu,
            "T_LIST_X_V1" => ObjectType::XList,
            other => ObjectType::Other(other.into()),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// TODO: Document
pub struct SliderInfo {
    #[serde(rename = "DECMARKER")]
    pub dec_marker: String,
    #[serde(rename = "INCMARKER")]
    pub inc_marker: String,
    pub increment: i32,
    #[serde(rename = "MAXIMUM")]
    pub max: i32,
    #[serde(rename = "MINIMUM")]
    pub min: i32,
    pub center: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// TODO: Document
pub struct SubSetting {
    #[serde(rename = "CNAME")]
    endpoint: String,
    hashval: u32,
    #[serde(deserialize_with = "string_to_bool", default)]
    hidden: bool,
    name: String,
    #[serde(deserialize_with = "string_to_bool", default)]
    readonly: bool,
    #[serde(rename = "TYPE")]
    object_type: ObjectType,
    value: Option<Value>, // Not a serde_json Value; the field named value
    #[serde(skip)]
    device: Option<Device>, // Option wrap is a workaround - TODO
}

impl SubSetting {
    // TODO: rename maybe / Document
    pub async fn expand(&self) -> Result<Vec<SubSetting>> {
        let res = self.dynamic_response().await?;
        let mut settings = res.settings()?;

        // Add device reference and update endpoint
        for s in settings.iter_mut() {
            s.add_parent_data(self);
        }
        Ok(settings)
    }

    // TODO: Document
    pub fn hidden(&self) -> bool {
        self.hidden
    }

    // TODO: Document
    pub fn object_type(&self) -> ObjectType {
        self.object_type.clone()
    }

    // TODO: Document
    pub async fn slider_info(&self) -> Result<Option<SliderInfo>> {
        if self.object_type == ObjectType::Slider {
            Ok(self.static_response().await?.slider_info().ok())
        } else {
            Ok(None)
        }
    }

    // TODO: Document
    pub async fn elements(&self) -> Result<Option<Vec<String>>> {
        if self.object_type == ObjectType::List || self.object_type == ObjectType::XList {
            Ok(self.static_response().await?.elements().ok())
        } else {
            Ok(None)
        }
    }

    pub(crate) fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    /// Get Setting value at the dynamic endpoint
    async fn dynamic_response(&self) -> Result<Response> {
        let device = self.device.clone().unwrap();
        Ok(device
            .send_command(CommandDetail::ReadSettings(
                EndpointBase::Dynamic,
                self.endpoint(),
            ))
            .await?)
    }

    /// Get setting value at the static endpoint
    async fn static_response(&self) -> Result<Response> {
        let device = self.device.clone().unwrap();
        Ok(device
            .send_command(CommandDetail::ReadSettings(
                EndpointBase::Static,
                self.endpoint(),
            ))
            .await?)
    }

    /// Get the top level settings menu
    async fn root(device: Device) -> Result<Vec<SubSetting>> {
        let trunk = SubSetting {
            endpoint: format!("/{}", device.settings_root()),
            hashval: 0,
            hidden: false,
            name: "".into(),
            readonly: false,
            object_type: ObjectType::Menu,
            value: None,
            device: Some(device.clone()),
        };
        trunk.expand().await
    }

    fn add_parent_data(&mut self, parent: &SubSetting) {
        self.device = parent.device.clone();
        self.endpoint = format!("{}/{}", parent.endpoint, self.endpoint);
    }
}

pub async fn root(device: Device) -> Result<Vec<SubSetting>> {
    SubSetting::root(device).await
}

fn string_to_bool<'de, D>(deserializer: D) -> StdResult<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    string
        .to_lowercase()
        .parse::<bool>()
        .map_err(|_| de::Error::invalid_type(de::Unexpected::Str(&string), &"a boolean"))
}
