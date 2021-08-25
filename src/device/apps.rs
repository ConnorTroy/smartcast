use super::Result;

use reqwest::Client;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug};

pub const APP_PAYLOAD_URL: &str =
    "http://hometest.buddytv.netdna-cdn.com/appservice/app_availability_prod.json";
pub const APP_NAME_URL: &str =
    "http://hometest.buddytv.netdna-cdn.com/appservice/vizio_apps_prod.json";

#[derive(Clone)]
/// Various information about an App
pub struct App {
    name: String,
    description: String,
    image_url: String,
    id: String,
    payload: Option<Payload>,
}

impl App {
    /// Get the name of the App
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get a description of the App
    pub fn description(&self) -> String {
        self.description.clone()
    }

    /// Get a url for the App's icon image
    pub fn image_url(&self) -> String {
        self.image_url.clone()
    }
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("App");
        d.field("name", &self.name);
        d.field("description", &self.description);
        d.field("image_url", &self.image_url);
        d.finish()
    }
}

impl<'de> Deserialize<'de> for App {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct OuterObject {
            id: String,
            name: String,
            #[serde(rename(deserialize = "mobileAppInfo"))]
            mobile_app_info: InnerObject,
        }

        #[derive(Deserialize)]
        struct InnerObject {
            description: String,
            #[serde(rename(deserialize = "app_icon_image_url"))]
            image_url: String,
        }

        let helper = OuterObject::deserialize(deserializer)?;
        Ok(App {
            id: helper.id,
            name: helper.name,
            description: helper.mobile_app_info.description,
            image_url: helper.mobile_app_info.image_url,
            payload: None,
        })
    }
}

#[derive(Debug)]
/// Struct used to facilitate populating app info
pub(super) struct AppList {
    payloads: HashMap<String, Payload>,
    apps: HashMap<String, App>,
    client: Client,
}

impl AppList {
    pub fn new(client: Client) -> Self {
        Self {
            payloads: HashMap::new(),
            apps: HashMap::new(),
            client,
        }
    }

    /// Get app by payload
    pub async fn get_app(&mut self, payload: Payload) -> Result<Option<App>> {
        if self.payloads.is_empty() {
            self.update().await?;
        }

        Ok(self
            .apps
            .values()
            .find_map(|app| match &app.payload {
                Some(pl) if pl == &payload => Some(app),
                _ => None,
            })
            .cloned())
    }

    /// Update payloads and app descriptions
    pub async fn update(&mut self) -> Result<()> {
        self.fetch_payloads().await?;
        self.fetch_app_info().await?;
        Ok(())
    }

    /// Get payloads from online source
    async fn fetch_payloads(&mut self) -> Result<()> {
        let payloads: Vec<Value> = self
            .client
            .get(APP_PAYLOAD_URL)
            .send()
            .await?
            .json()
            .await?;
        for p in payloads.iter() {
            let info = p["chipsets"]["*"][0].clone();

            let id: String = serde_json::from_value(p["id"].clone())?;
            let payload: Payload = if let Some(payload_str) = info["app_type_payload"].as_str() {
                serde_json::from_str(payload_str)?
            } else {
                serde_json::from_value(info["app_type_payload"].clone())?
            };

            self.payloads.insert(id, payload);
        }

        Ok(())
    }

    /// Get app info from online source
    async fn fetch_app_info(&mut self) -> Result<()> {
        if self.payloads.is_empty() {
            self.fetch_payloads().await?;
        }

        let mut apps: Vec<App> = self.client.get(APP_NAME_URL).send().await?.json().await?;
        self.apps = apps.iter_mut().fold(HashMap::new(), |mut map, mut app| {
            app.payload = self.payloads.get(&app.id).cloned();
            map.insert(app.id.clone(), app.clone());
            map
        });

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) struct Payload {
    name_space: u32,
    app_id: String,
    #[serde(deserialize_with = "null_string")]
    message: String,
}

fn null_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    Ok(String::deserialize(deserializer).unwrap_or_default())
}
