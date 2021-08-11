use super::{CommandDetail, Device, Response, Result};

use serde::{de, Deserialize};
use serde_json::Value;

use std::fmt;
use std::result::Result as StdResult;

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
/// Object types to which [`SubSetting`] corresponds
pub enum SettingsType {
    /// Slider which has a max/min value. See [`SliderInfo`] for more details.
    Slider,
    /// List of possible values which should be displayed. Use [`elements()`](SubSetting::elements) to get list data.
    List,
    /// Mutable value
    Value,
    /// Menu containing more [`SubSetting`]s
    Menu,
    /// List of possible values which should be displayed. Use [`elements()`](SubSetting::elements) to get list data.
    XList,
    #[doc(hidden)]
    Other(String),
}

/// Deserializer for [`SettingsType`]
impl<'de> Deserialize<'de> for SettingsType {
    fn deserialize<D>(deserializer: D) -> StdResult<SettingsType, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "T_VALUE_ABS_V1" => SettingsType::Slider,
            "T_LIST_V1" => SettingsType::List,
            "T_VALUE_V1" => SettingsType::Value,
            "T_MENU_V1" => SettingsType::Menu,
            "T_LIST_X_V1" => SettingsType::XList,
            other => SettingsType::Other(other.into()),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Information about a settings slider
pub struct SliderInfo {
    #[serde(rename = "DECMARKER")]
    /// Text at the low end of the slider
    pub dec_marker: String,
    #[serde(rename = "INCMARKER")]
    /// Text at the high end of the slider
    pub inc_marker: String,
    /// Amount value can change at a time
    pub increment: i32,
    #[serde(rename = "MAXIMUM")]
    /// Slider max value
    pub max: i32,
    #[serde(rename = "MINIMUM")]
    /// Slider min value
    pub min: i32,
    /// Value at center of the slider
    pub center: i32,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Settings for a Device
///
/// Because every device has a different settings layout, we need to propagate through them at runtime.
/// You can get the root of a [`Device`]s settings using the [`settings()`](Device::settings) method.
/// Propagate through the settings using [`expand()`](SubSetting::expand).
///
/// A `SubSetting` [`SettingsType`] can correspond any one of the following:
/// * `Menu` - an object which contains settings or more menus
/// * `Value` - a setting with a set value
/// * `Slider` - a setting with possible values on a scale
/// * `List` or `Xlist` - a setting with a list of possible values
///
/// # Example
///
/// ```
/// use smartcast::{Device, SubSetting};
///
/// # async fn example() -> Result<(), smartcast::Error> {
/// let mut dev = Device::from_ip("192.168.0.14").await?;
/// dev.set_auth_token("Z2zscc1udl");
///
/// let settings: Vec<SubSetting> = dev.settings().await?;
/// println!("{:#?}", settings);
/// // > [
/// // > SubSetting {
/// // >     name: "Picture Mode",
/// // >     value: Some(
/// // >         String(
/// // >             "Calibrated",
/// // >         ),
/// // >     ),
/// // >     hidden: false,
/// // >     read_only: false,
/// // >     object_type: XList,
/// // > },
/// // > ...
/// // > ]
///
/// let pic_settings: Vec<SubSetting> = settings[0].expand().await?;
/// println!("{:#?}", pic_settings);
/// // > [
/// // > SubSetting {
/// // >     name: "Picture Mode",
/// // >     value: Some(
/// // >         String(
/// // >             "Calibrated",
/// // >         ),
/// // >     ),
/// // >     hidden: false,
/// // >     read_only: false,
/// // >     object_type: XList,
/// // > },
/// // > SubSetting {
/// // >     name: "Ambient Light Sensor",
/// // >     value: Some(
/// // >         String(
/// // >             "Off",
/// // >         ),
/// // >     ),
/// // >     hidden: false,
/// // >     read_only: false,
/// // >     object_type: List,
/// // > },
/// // > ...
/// // > ]
/// # Ok(())
/// # }
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
    object_type: SettingsType,
    value: Option<Value>, // Not a serde_json Value; the field named value
    #[serde(skip)]
    device: Option<Device>,
}

impl fmt::Debug for SubSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_struct("SubSetting");
        d.field("name", &self.name);
        d.field("value", &self.value);
        d.field("hidden", &self.hidden);
        d.field("read_only", &self.readonly);
        d.field("object_type", &self.object_type);
        d.finish()
    }
}

impl SubSetting {
    /// If the settings object is a `Menu`, get its [`SubSetting`]s
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::{Device, SubSetting};
    /// #
    /// # async fn example() -> Result<(), smartcast::Error> {
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    /// dev.set_auth_token("Z2zscc1udl");
    /// let settings: Vec<SubSetting> = dev.settings().await?;
    /// println!("{:#?}", settings);
    /// // > [
    /// // > SubSetting {
    /// // >     name: "Picture",
    /// // >     value: None,
    /// // >     hidden: false,
    /// // >     read_only: false,
    /// // >     object_type: Menu,
    /// // > },
    /// // > ...
    /// // > ]
    /// let pic_settings: Vec<SubSetting> = settings[0].expand().await?;
    /// println!("{:#?}", pic_settings);
    /// // > [
    /// // > SubSetting {
    /// // >     name: "Picture Mode",
    /// // >     value: Some(
    /// // >         String(
    /// // >             "Calibrated",
    /// // >         ),
    /// // >     ),
    /// // >     hidden: false,
    /// // >     read_only: false,
    /// // >     object_type: XList,
    /// // > },
    /// // > SubSetting {
    /// // >     name: "Ambient Light Sensor",
    /// // >     value: Some(
    /// // >         String(
    /// // >             "Off",
    /// // >         ),
    /// // >     ),
    /// // >     hidden: false,
    /// // >     read_only: false,
    /// // >     object_type: List,
    /// // > },
    /// // > ...
    /// // > ]
    /// # Ok(())
    /// # }
    /// ```
    pub async fn expand(&self) -> Result<Vec<SubSetting>> {
        let res = self.dynamic_response().await?;
        let mut settings = res.settings()?;

        // Add device reference and update endpoint
        for s in settings.iter_mut() {
            s.add_parent_data(self);
        }
        Ok(settings)
    }

    /// Name of the setting
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Whether the setting should be displayed
    pub fn hidden(&self) -> bool {
        self.hidden
    }

    /// Whether the setting is read-only
    pub fn read_only(&self) -> bool {
        self.readonly
    }

    /// Type of the settings object. See [`SettingsType`].
    pub fn object_type(&self) -> SettingsType {
        self.object_type.clone()
    }

    /// The current value of the setting
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::{Device, SubSetting};
    /// #
    /// # async fn example() -> Result<(), smartcast::Error> {
    /// # let mut dev = Device::from_ip("192.168.0.14").await?;
    /// let settings: Vec<SubSetting> = dev.settings().await?;
    /// let pic_settings: Vec<SubSetting> = settings[0].expand().await?;
    /// println!("{:#?}", pic_settings);
    /// // > [
    /// // > SubSetting {
    /// // >     name: "Picture Mode",
    /// // >     value: Some(
    /// // >         String(
    /// // >             "Calibrated",
    /// // >         ),
    /// // >     ),
    /// // >     hidden: false,
    /// // >     read_only: false,
    /// // >     object_type: XList,
    /// // > },
    /// // > ...
    /// // > ]
    /// if let Some(value) = pic_settings[0].value::<String>() {
    ///     println!("{}", value);
    /// }
    /// // > Calibrated
    /// # Ok(())
    /// # }
    /// ```
    pub fn value<T: for<'de> serde::Deserialize<'de>>(&self) -> Option<T> {
        if let Some(value) = self.value.clone() {
            serde_json::from_value(value).ok()
        } else {
            None
        }
    }

    /// If the setting object is a `Slider`, get the slider info. See [`SliderInfo`].
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::{Device, SubSetting};
    /// #
    /// # async fn example() -> Result<(), smartcast::Error> {
    /// # let mut dev = Device::from_ip("192.168.0.14").await?;
    /// let settings: Vec<SubSetting> = dev.settings().await?;
    /// let pic_settings: Vec<SubSetting> = settings[0].expand().await?;
    /// println!("{:#?}", pic_settings);
    /// // > [
    /// // > ...
    /// // > SubSetting {
    /// // >     name: "Tint",
    /// // >     value: Some(
    /// // >         Number(
    /// // >             0,
    /// // >         ),
    /// // >     ),
    /// // >     hidden: false,
    /// // >     read_only: false,
    /// // >     object_type: Slider,
    /// // > },
    /// // > ...
    /// // > ]
    /// if let Some(slider_info) = pic_settings[8].slider_info().await? {
    ///     println!("{:#?}", slider_info);
    /// }
    /// // > SliderInfo {
    /// // > dec_marker: "Red",
    /// // > inc_marker: "Green",
    /// // > increment: 1,
    /// // > max: 50,
    /// // > min: -50,
    /// // > center: 0,
    /// // > }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn slider_info(&self) -> Result<Option<SliderInfo>> {
        if self.object_type == SettingsType::Slider {
            match self.static_response().await?.slider_info() {
                Ok(info) => Ok(Some(info)),
                Err(_) => Ok(self.dynamic_response().await?.slider_info().ok()),
            }
        } else {
            Ok(None)
        }
    }

    /// If the setting object is a `List` or `XList`, get its elements. See [`SettingsType`].
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::{Device, SubSetting};
    /// #
    /// # async fn example() -> Result<(), smartcast::Error> {
    /// # let mut dev = Device::from_ip("192.168.0.14").await?;
    /// let settings: Vec<SubSetting> = dev.settings().await?;
    /// let pic_settings: Vec<SubSetting> = settings[0].expand().await?;
    /// println!("{:#?}", pic_settings);
    /// // > [
    /// // > SubSetting {
    /// // >     name: "Picture Mode",
    /// // >     value: Some(
    /// // >         String(
    /// // >             "Calibrated",
    /// // >         ),
    /// // >     ),
    /// // >     hidden: false,
    /// // >     read_only: false,
    /// // >     object_type: XList,
    /// // > },
    /// // > ...
    /// // > ]
    /// if let Some(elements) = pic_settings[0].elements().await? {
    ///     println!("{:#?}", elements);
    /// }
    /// // > [
    /// // > "Vivid",
    /// // > "Bright",
    /// // > "Calibrated",
    /// // > "Calibrated Dark*",
    /// // > "Game",
    /// // > "Sports",
    /// // > ],
    /// # Ok(())
    /// # }
    /// ```
    pub async fn elements(&self) -> Result<Option<Vec<String>>> {
        if self.object_type == SettingsType::List || self.object_type == SettingsType::XList {
            match self.dynamic_response().await?.elements() {
                Ok(elements) => Ok(Some(elements)),
                Err(_) => Ok(self.static_response().await?.elements().ok()),
            }
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
            object_type: SettingsType::Menu,
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
