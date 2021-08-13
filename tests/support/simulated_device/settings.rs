// TODO: Remove Ignores
#![allow(dead_code)]
#![allow(unused)]
use super::{commands, rand_data, Result};

use smartcast::SliderInfo;

use rand::Rng;
use serde::{ser::SerializeStruct, Serialize};
use serde_json::{json, Value};
use warp::{filters::BoxedFilter, Filter, Reply};

use std::collections::HashMap;

pub const LIST_LEN: usize = 5;

#[derive(Debug, Clone)]
pub enum SettingType {
    Slider,
    Value,
    Menu(Vec<Setting>),
    List,
    XList,
}

impl SettingType {
    fn cname(&self) -> String {
        match self {
            Self::Slider => "slider",
            Self::Value => "value",
            Self::Menu(_) => "menu",
            Self::List => "list",
            Self::XList => "x_list",
        }
        .into()
    }
}

impl ToString for SettingType {
    fn to_string(&self) -> String {
        match self {
            Self::Slider => "T_VALUE_ABS_V1",
            Self::Value => "T_VALUE_V1",
            Self::Menu(_) => "T_MENU_V1",
            Self::List => "T_LIST_V1",
            Self::XList => "T_LIST_X_V1",
        }
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct Setting {
    pub name: String,
    pub cname: String,
    pub setting_type: SettingType,
    pub value: Value,
    pub hidden: bool,
    pub hashval: u32,
    pub elements: Vec<String>,
}

impl Setting {
    fn new(setting_type: SettingType) -> Self {
        let mut rng = rand::thread_rng();
        let mut elements = Vec::new();
        if matches!(setting_type, SettingType::List) || matches!(setting_type, SettingType::XList) {
            for i in 0..LIST_LEN {
                elements.push(rand_data::string(10));
            }
        }

        Self {
            name: rand_data::string(6),
            cname: setting_type.cname(),
            setting_type,
            value: json!(serde_json::Value::Null),
            hidden: false,
            hashval: rng.gen(),
            elements,
        }
    }

    fn dynamic_in_menu(&self) -> String {
        match self.setting_type {
            SettingType::Menu(_) => format!(
                r#"
                {{
                    "CNAME": "{}",
                    "HASHVAL": {},
                    "NAME": "{}",
                    "TYPE": "{}",
                }}
                "#,
                self.cname,
                self.hashval,
                self.name,
                self.setting_type.to_string(),
            ),
            SettingType::XList => format!(
                r#"
                {{
                    "CNAME": "{}",
                    "ELEMENTS": ["{}"],
                    "HASHVAL": {},
                    "NAME": "{}",
                    "TYPE": "{}",
                    "VALUE": "{}"
                }}
                "#,
                self.cname,
                self.elements.join("\", \""),
                self.hashval,
                self.name,
                self.setting_type.to_string(),
                self.value,
            ),
            _ => format!(
                r#"
                {{
                    "CNAME": "{}",
                    "HASHVAL": {},
                    "NAME": "{}",
                    "TYPE": "{}",
                    "VALUE": "{}"
                }}
                "#,
                self.cname,
                self.hashval,
                self.name,
                self.setting_type.to_string(),
                self.value,
            ),
        }
    }

    fn dynamic_as_string(&self) -> String {
        let mut rng = rand::thread_rng();
        let hashlist: Vec<u32> = vec![rng.gen(), rng.gen()];

        match self.setting_type.clone() {
            SettingType::Menu(submenus) => {
                let items: String = submenus
                    .iter()
                    .map(|x| x.dynamic_in_menu())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!(
                    r#"
                    {{
                        "CNAME": "{}",
                        "GROUP": "G_SOMEGROUP",
                        "HASHLIST": {:?},
                        "ITEMS": [{}],
                        "NAME": "{}",
                        "PARAMETERS": {{
                            "FLAT": "TRUE",
                            "HASHONLY": "FALSE",
                            "HELPTEXT": "FALSE"
                        }},
                        {},
                        "TYPE": "{}"
                    }}
                    "#,
                    self.cname,
                    hashlist,
                    items,
                    self.name,
                    status!(Result::Success),
                    self.setting_type.to_string(),
                )
            }
            SettingType::XList => {
                format!(
                    r#"
                    {{
                        "HASHLIST": {:?},
                        "ITEMS": [
                        {{
                            "CNAME": "{}",
                            "ELEMENTS": {:?},
                            "HASHVAL": {},
                            "NAME": "{}",
                            "TYPE": "{}",
                            "VALUE": {}
                        }}
                        ],
                        "PARAMETERS": {{
                            "FLAT": "TRUE",
                            "HASHONLY": "FALSE",
                            "HELPTEXT": "FALSE"
                        }},
                        {}
                    }}
                    "#,
                    hashlist,
                    self.cname,
                    self.elements,
                    self.hashval,
                    self.name,
                    self.setting_type.to_string(),
                    self.value,
                    status!(Result::Success),
                )
            }
            _ => {
                format!(
                    r#"
                    {{
                        "HASHLIST": {:?},
                        "ITEMS": [
                        {{
                            "CNAME": "{}",
                            "HASHVAL": {},
                            "NAME": "{}",
                            "TYPE": "{}",
                            "VALUE": {}
                        }}
                        ],
                        "PARAMETERS": {{
                            "FLAT": "TRUE",
                            "HASHONLY": "FALSE",
                            "HELPTEXT": "FALSE"
                        }},
                        {}
                    }}
                    "#,
                    hashlist,
                    self.cname,
                    self.hashval,
                    self.name,
                    self.setting_type.to_string(),
                    self.value,
                    status!(Result::Success),
                )
            }
        }
    }

    fn static_as_string(&self) -> String {
        match self.setting_type {
            SettingType::List
            | SettingType::XList => {
                format!(
                    r#"
                    {{
                        "HASHVAL": {},
                        "ITEMS": [
                        {{
                            "CNAME": "{}",
                            "ELEMENTS": {:?},
                            "NAME": "{}",
                            "TYPE": "{}"
                        }}
                        ],
                        "PARAMETERS": {{
                            "FLAT": "TRUE",
                            "HASHONLY": "FALSE",
                            "HELPTEXT": "FALSE"
                        }},
                        {}
                    }}
                    "#,
                    self.hashval,
                    self.cname,
                    self.elements,
                    self.name,
                    self.setting_type.to_string(),
                    status!(Result::Success),
                )
            },
            SettingType::Slider => {
                format!(
                    r#"
                    {{
                        "HASHVAL": {},
                        "ITEMS": [
                        {{
                            "CENTER": 0,
                            "CNAME": "{}",
                            "DECMARKER": "low_end",
                            "INCMARKER": "high_end",
                            "INCREMENT": 1,
                            "MAXIMUM": 100,
                            "MINIMUM": -100,
                            "NAME": "{}",
                            "TYPE": "T_VALUE_ABS_V1"
                        }}
                        ],
                        "PARAMETERS": {{
                            "FLAT": "TRUE",
                            "HASHONLY": "FALSE",
                            "HELPTEXT": "FALSE"
                        }},
                        {}
                    }}
                    "#,
                    self.hashval,
                    self.cname,
                    self.name,
                    status!(Result::Success),
                )
            }
            _ => {
                log::error!(target: "test::simulated_device::settings", "Unexpected Static GET");
                panic!("Unexpected Static GET");
            }
        }
    }

    pub fn dynamic_value(&self) -> Value {
        let strvalue = self.dynamic_as_string();
        serde_json::from_str(&strvalue).unwrap()
    }

    pub fn static_value(&self) -> Value {
        let strvalue = self.static_as_string();
        serde_json::from_str(&strvalue).unwrap()
    }

    pub fn dynamic_filter_read(&self) -> BoxedFilter<(impl Reply,)> {
        let cname = warp::path(self.cname.clone());
        let end = warp::path::end().and(warp::get()).map({
            let setting = self.clone();
            move || commands::read_setting_dynamic(setting.clone())
        });

        if matches!(self.setting_type, SettingType::Menu(_)) {
            end.boxed()
        } else {
            cname.and(end).boxed()
        }
    }

    pub fn static_filter(&self) -> BoxedFilter<(impl Reply,)> {
        let cname = warp::path(self.cname.clone());
        let end = warp::path::end().and(warp::get()).map({
            let setting = self.clone();
            move || commands::read_setting_static(setting.clone())
        });

        if matches!(self.setting_type, SettingType::Menu(_)) {
            end.boxed()
        } else {
            cname.and(end).boxed()
        }
    }
}

pub fn expected_slider_info() -> SliderInfo {
    SliderInfo {
        dec_marker: "low_end".into(),
        inc_marker: "high_end".into(),
        increment: 1,
        max: 100,
        min: -100,
        center: 0,
    }
}

pub fn generate(settings_root: String) -> BoxedFilter<(impl Reply,)> {
    let value_setting = Setting::new(SettingType::Value);
    let slider_setting = Setting::new(SettingType::Slider);
    let list_setting = Setting::new(SettingType::List);
    let x_list_setting = Setting::new(SettingType::XList);
    let menu_setting = Setting::new(SettingType::Menu(vec![
        value_setting.clone(),
        slider_setting.clone(),
        list_setting.clone(),
        x_list_setting.clone(),
    ]));

    warp::path("dynamic")
        .and(warp::path(settings_root.clone()))
        .and(
            menu_setting
                .dynamic_filter_read()
                .or(value_setting.dynamic_filter_read())
                .or(slider_setting.dynamic_filter_read())
                .or(list_setting.dynamic_filter_read())
                .or(x_list_setting.dynamic_filter_read()),
        )
        .or(
            warp::path("static")
            .and(warp::path(settings_root))
            .and(
            menu_setting
                .static_filter()
                .or(value_setting.static_filter())
                .or(slider_setting.static_filter())
                .or(list_setting.static_filter())
                .or(x_list_setting.static_filter()),
        ))
        .boxed()
}
