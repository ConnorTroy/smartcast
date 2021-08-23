use super::{CommandDetail, Device, Response};
use crate::error::{ClientError, Error, Result};

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
/// Object types to which [`SubSetting`] corresponds.
pub enum SettingType {
    /// Slider which has a max/min value. See [`SliderInfo`] for more details.
    Slider,
    /// Common setting with some value
    Value,
    /// Menu containing more [`SubSetting`]s
    Menu,
    /// List of possible values which should be displayed. Use [`elements()`](SubSetting::elements) to get list data.
    List,
    /// List of possible values which should be displayed. Use [`elements()`](SubSetting::elements) to get list data.
    XList,
    #[doc(hidden)]
    Other(String),
}

/// Deserializer for [`SettingType`]
impl<'de> Deserialize<'de> for SettingType {
    fn deserialize<D>(deserializer: D) -> StdResult<SettingType, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "T_VALUE_ABS_V1" => SettingType::Slider,
            "T_LIST_V1" => SettingType::List,
            "T_VALUE_V1" => SettingType::Value,
            "T_MENU_V1" => SettingType::Menu,
            "T_LIST_X_V1" => SettingType::XList,
            other => SettingType::Other(other.into()),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Information about a settings slider
pub struct SliderInfo {
    #[serde(rename = "DECMARKER")]
    #[serde(default)]
    /// Text at the low end of the slider
    pub dec_marker: String,
    #[serde(rename = "INCMARKER")]
    #[serde(default)]
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
    pub center: Option<i32>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Settings for a Device
///
/// Because every device has a different settings layout, we need to propagate through them at runtime.
/// You can get the root of a [`Device`]s settings using the [`settings()`](Device::settings) method.
/// Propagate through the settings using [`expand()`](SubSetting::expand).
///
/// A `SubSetting` [`SettingType`] can correspond any one of the following:
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
    hashval: Option<u32>,
    #[serde(deserialize_with = "string_to_bool", default)]
    hidden: bool,
    name: String,
    #[serde(deserialize_with = "string_to_bool", default)]
    readonly: bool,
    #[serde(rename = "TYPE")]
    object_type: SettingType,
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
    /// If the settings object is a `Menu`, get its [`SubSetting`]s.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {
    /// use smartcast::{Device, SubSetting};
    ///
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
        log::trace!("SubSetting Expand");
        if !matches!(self.object_type, SettingType::Menu) {
            return Ok(vec![self.clone()]);
        }

        let mut settings: Vec<SubSetting> = self.dynamic_response().await?.settings()?;

        // Add device reference and update endpoint
        for s in settings.iter_mut() {
            s.add_parent_data(self);

            // Some value types are actually sliders so try to update accordingly
            if s.object_type == SettingType::Value {
                s.object_type = SettingType::Slider;
                if s.slider_info().await?.is_none() {
                    s.object_type = SettingType::Value;
                }
            }
        }
        Ok(settings)
    }

    /// Name of the setting.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns true if the setting should be displayed.
    pub fn hidden(&self) -> bool {
        self.hidden
    }

    /// Returns true if the setting is read only.
    pub fn read_only(&self) -> bool {
        self.readonly
    }

    /// Type of the settings object. See [`SettingType`].
    pub fn setting_type(&self) -> SettingType {
        self.object_type.clone()
    }

    /// Returns true if the value is a boolean. Returns false otherwise.
    pub fn is_boolean(&self) -> bool {
        if let Some(value) = self.value.clone() {
            value.is_boolean()
        } else {
            false
        }
    }

    /// Returns true if the value is a String. Returns false otherwise.
    pub fn is_string(&self) -> bool {
        if let Some(value) = self.value.clone() {
            value.is_string()
        } else {
            false
        }
    }

    /// Returns true if the Value is a 32 bit signed integer.
    pub fn is_number(&self) -> bool {
        if let Some(value) = self.value.clone() {
            value.is_number()
        } else {
            false
        }
    }

    /// Get the current value of the setting.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {
    /// use smartcast::{Device, SubSetting};
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    ///
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
    pub fn value<T>(&self) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(value) = self.value.clone() {
            serde_json::from_value(value).ok()
        } else {
            None
        }
    }

    /// Change the value of the setting.
    ///
    /// Returns an error if:
    /// * The setting is `read-only`.
    /// * The value passed in is not the same type as the value currently in the setting.
    /// * In the case of a `Slider`, the value passed in is higher than the max or lower than the min.
    /// * In the case of a `List` or `XList`, the value passed in is not present in the setting's [`Elements`](Self::elements).
    /// * The [`setting type`](Self::setting_type) is not a `Slider`, `List`, `Xlist`, or `Value`.
    ///
    /// # Example
    /// ```
    /// todo!();
    /// ```
    #[allow(unused)] // Temp - TODO: remove
    pub async fn write<T>(&self, new_value: T) -> Result<()> {
        log::trace!("Write SubSetting");
        todo!();
    }

    /// If the setting object is a `Slider`, get the slider info. See [`SliderInfo`].
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {
    /// use smartcast::{Device, SubSetting};
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    ///
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
    /// // >     dec_marker: "Red",
    /// // >     inc_marker: "Green",
    /// // >     increment: 1,
    /// // >     max: 50,
    /// // >     min: -50,
    /// // >     center: 0,
    /// // > }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn slider_info(&self) -> Result<Option<SliderInfo>> {
        log::trace!("Get Slider Info");
        if self.object_type == SettingType::Slider {
            match self.static_response().await?.slider_info() {
                Ok(info) => Ok(Some(info)),
                Err(_) => Ok(self.dynamic_response().await?.slider_info().ok()),
            }
        } else {
            Ok(None)
        }
    }

    /// If the setting object is a `List` or `XList`, get its elements. See [`SettingType`].
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::{Device, SubSetting};
    ///
    /// let dev = Device::from_ip("192.168.0.14").await?;
    ///
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
    /// println!("{:#?}", pic_settings[0].elements().await?);
    /// // > [
    /// // >     "Vivid",
    /// // >     "Bright",
    /// // >     "Calibrated",
    /// // >     "Calibrated Dark*",
    /// // >     "Game",
    /// // >     "Sports",
    /// // > ],
    /// # Ok(())
    /// # }
    /// ```
    pub async fn elements(&self) -> Result<Vec<String>> {
        log::trace!("Get Elements");
        if self.object_type == SettingType::List || self.object_type == SettingType::XList {
            match self.dynamic_response().await?.elements() {
                Ok(elements) => Ok(elements),
                Err(_) => Ok(self.static_response().await?.elements().unwrap_or_default()),
            }
        } else {
            Ok(Vec::new())
        }
    }

    pub(super) fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    /// Get Setting value at the dynamic endpoint
    async fn dynamic_response(&self) -> Result<Response> {
        log::trace!("Get Dynamic Response");
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
        log::trace!("Get Static Response");
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
        log::trace!("Get Settings Root");
        let root = SubSetting {
            endpoint: format!("/{}", device.settings_root()),
            hashval: None,
            hidden: false,
            name: "".into(),
            readonly: false,
            object_type: SettingType::Menu,
            value: None,
            device: Some(device.clone()),
        };
        root.expand().await
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
