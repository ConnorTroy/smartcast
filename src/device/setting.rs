use super::Result;
use super::Error;

// use serde::de;
// use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};
use serde_json::Value;
use serde::{de, Deserialize};

use std::result::Result as StdResult;
// use std::fmt;

const STATIC_BASE: &str = "/menu_native/static/tv_settings/";
const DYNAMIC_BASE: &str = "/menu_native/dynamic/tv_settings/";

pub trait SettingValue<T> {
    fn value(&self) -> Result<T>;
}

pub enum UrlBase {
    Dynamic,
    Static,
    None,
}

impl UrlBase {
    fn base(&self) -> String {
        match self {
            Self::Static => STATIC_BASE,
            Self::Dynamic => DYNAMIC_BASE,
            Self::None => "",
        }.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Slider,
    List,
    Value,
    Menu,
    XList,
    Other(String),
}

/// Deserializer for [`ObjectType`]
impl<'de> Deserialize<'de> for ObjectType {
    fn deserialize<D>(deserializer: D) -> StdResult<ObjectType, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(
            match String::deserialize(deserializer)?.as_str() {
                "T_VALUE_ABS_V1" => ObjectType::Slider,
                "T_LIST_V1" => ObjectType::List,
                "T_VALUE_V1" => ObjectType::Value,
                "T_MENU_V1" => ObjectType::Menu,
                "T_LIST_X_V1" => ObjectType::XList,
                other => ObjectType::Other(other.into()),
            }
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
    object_type: ObjectType,
    value: Option<Value>,
    slider_info: Option<SliderInfo>,
    elements: Option<Vec<String>>,
}

impl SubSetting {

    pub fn top_level() -> Self {
        Self::default()
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }

    pub fn object_type(&self) -> ObjectType {
        self.object_type.clone()
    }

    pub fn slider_info(&self) -> Option<SliderInfo> {
        self.slider_info.clone()
    }

    pub fn elements(&self) -> Option<Vec<String>> {
        self.elements.clone()
    }

    pub(crate) fn endpoint(&self, endpoint_base: UrlBase) -> String {
        endpoint_base.base() + &self.endpoint
    }

    pub(crate) fn add_parent_endpoint(&mut self, parent_endpoint: String) {
        if parent_endpoint.as_str() != "" {
            self.endpoint.insert_str(0, &(parent_endpoint + "/"));
        }
    }

    pub(crate) fn add_slider_info(&mut self, slider_info: SliderInfo) {
        self.slider_info = Some(slider_info);
    }

    pub(crate) fn add_elements(&mut self, elements: Vec<String>) {
        self.elements = Some(elements);
    }
}

impl SettingValue<String> for SubSetting {
    fn value(&self) -> Result<String> {
        match &self.value {
            Some(value) => Ok(serde_json::from_value(value.clone())?),
            None => Err(Error::Other("Value is None".into())),
        }
    }
}

impl SettingValue<Value> for SubSetting {
    fn value(&self) -> Result<Value> {
        match &self.value {
            Some(value) => Ok(value.clone()),
            None => Err(Error::Other("Value is None".into())),
        }
    }
}

impl Default for SubSetting {
    fn default() -> Self {
        Self {
            endpoint:       "".into(),
            hashval:        None,
            hidden:         false,
            name:           "TV Settings".into(),
            readonly:       false,
            object_type:    ObjectType::Menu,
            value:          None,
            slider_info:    None,
            elements:       None,
        }
    }
}

fn string_to_bool<'de, D>(deserializer: D) -> StdResult<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    string
        .to_lowercase()
        .parse::<bool>()
        .or(Err(
            de::Error::invalid_type(
                de::Unexpected::Str(&string),
                &"a boolean"
            )
        ))
}

// impl<'de> Deserialize<'de> for SettingSubmenu {
//     fn deserialize<D>(deserializer: D) -> Result<SettingSubmenu, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         enum Field { CName, Elements, Hashval, Hidden, Name, ReadOnly, Type, Value, Extra}

//         impl<'de> Deserialize<'de> for Field {
//             fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
//             where
//                 D: Deserializer<'de>,
//             {
//                 struct FieldVisitor;

//                 impl<'de> Visitor<'de> for FieldVisitor {
//                     type Value = Field;

//                     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                         formatter.write_str("`CNAME` or `ELEMENTS` or `HASHVAL` or `HIDDEN` or `NAME` or `READONLY` or `TYPE` or `VALUE`")
//                     }

//                     fn visit_str<E>(self, value: &str) -> Result<Field, E>
//                     where
//                         E: de::Error,
//                     {
//                         match value {
//                             "CNAME"     => Ok(Field::CName),
//                             "ELEMENTS"  => Ok(Field::Elements),
//                             "HASHVAL"   => Ok(Field::Hashval),
//                             "HIDDEN"    => Ok(Field::Hidden),
//                             "NAME"      => Ok(Field::Name),
//                             "READONLY"  => Ok(Field::ReadOnly),
//                             "TYPE"      => Ok(Field::Type),
//                             "VALUE"     => Ok(Field::Value),
//                             "INDEX"     => Ok(Field::Extra),
//                             "STATUS"    => Ok(Field::Extra),
//                             "ENABLED"   => Ok(Field::Extra),
//                             _ => Err(de::Error::unknown_field(value, FIELDS)),
//                         }
//                     }
//                 }

//                 deserializer.deserialize_identifier(FieldVisitor)
//             }
//         }

//         struct SettingsVisitor;

//         impl<'de> Visitor<'de> for SettingsVisitor {
//             type Value = SettingSubmenu;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_map<V>(self, mut map: V) -> Result<SettingSubmenu, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 let mut cname = None;
//                 let mut elements = None;
//                 let mut hashval = None;
//                 let mut hidden = None;
//                 let mut name = None;
//                 let mut readonly = None;
//                 let mut object_type = None;
//                 let mut value = None;
//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::CName => {
//                             if cname.is_some() {
//                                 return Err(de::Error::duplicate_field("CNAME"));
//                             }
//                             cname = Some(map.next_value()?);
//                         },
//                         Field::Elements => {
//                             if elements.is_some() {
//                                 return Err(de::Error::duplicate_field("ELEMENTS"));
//                             }
//                             elements = Some(map.next_value()?);
//                         },
//                         Field::Hashval => {
//                             if hashval.is_some() {
//                                 return Err(de::Error::duplicate_field("HASHVAL"));
//                             }
//                             hashval = Some(map.next_value()?);
//                         },
//                         Field::Hidden => {
//                             if hidden.is_some() {
//                                 return Err(de::Error::duplicate_field("HIDDEN"));
//                             }
//                             hidden = Some(map.next_value::<bool>()?);
//                         },
//                         Field::Name => {
//                             if name.is_some() {
//                                 return Err(de::Error::duplicate_field("NAME"));
//                             }
//                             name = Some(map.next_value()?);
//                         },
//                         Field::ReadOnly => {
//                             if readonly.is_some() {
//                                 return Err(de::Error::duplicate_field("READONLY"));
//                             }
//                             readonly = Some(map.next_value::<bool>()?);
//                         },
//                         Field::Type => {
//                             if object_type.is_some() {
//                                 return Err(de::Error::duplicate_field("TYPE"));
//                             }
//                             object_type = Some(
//                                 match map.next_value::<String>()?.to_lowercase().as_str() {
//                                     // "t_value_abs_v1" => ObjectType::Slider,
//                                     "t_list_v1" => ObjectType::List,
//                                     "t_value_v1" => ObjectType::Value,
//                                     "t_menu_v1" => ObjectType::Menu,
//                                     "t_list_x_v1" => ObjectType::XList(elements.clone().unwrap()),
//                                     key => ObjectType::Other(key.into()),
//                                 }
//                             );
//                         },
//                         Field::Value => {
//                             if value.is_some() {
//                                 return Err(de::Error::duplicate_field("VALUE"));
//                             }
//                             value = Some(map.next_value()?);
//                         },
//                         _ => {},
//                     }
//                 }
//                 let cname = cname.ok_or_else(|| de::Error::missing_field("CNAME"))?;
//                 let hidden = hidden.unwrap_or(false);
//                 let name = name.ok_or_else(|| de::Error::missing_field("NAME"))?;
//                 let readonly = readonly.unwrap_or(false);
//                 let object_type = object_type.ok_or_else(|| de::Error::missing_field("TYPE"))?;
//                 Ok(SettingSubmenu::build(cname, hashval, hidden, name, readonly, object_type, value))
//             }
//         }

//         const FIELDS: &'static [&'static str] = &["CNAME", "HASHVAL", "HIDDEN", "ELEMENTS", "GROUP", "NAME", "TYPE", "VALUE"];
//         deserializer.deserialize_struct("", FIELDS, SettingsVisitor)
//     }
// }

