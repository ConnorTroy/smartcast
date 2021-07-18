mod command;
mod response;
mod setting;

use crate::discover::{ssdp, uaudp_followup};

use self::command::{Command, RequestType};
pub use self::command::{ButtonEvent, Button};
pub use self::response::Input;
pub use self::setting::{SubSetting, ObjectType, UrlBase};

use super::constant;
// use super::discover;
use super::error::{Error, Result};

use reqwest::Client;
use serde_json::Value;

use std::time::Duration;
// use std::rc::Rc;
// use std::sync::{Arc, RwLock};
// use std::cell::RefCell;

/// A Vizio Device
#[derive(Debug, Clone)]
pub struct Device {
    name: String,
    manufacturer: String,
    model: String,
    ip_addr: String,
    port: u16,
    uuid: String,
    auth_token: Option<String>,
    // info: Arc<RwLock<DeviceInfo>>,
    client: Client,
}

impl Device {
    pub(crate) fn new<S: Into<String>>(
        name: S,
        manufacturer: S,
        model: S,
        ip_addr: S,
        uuid: S
    ) -> Device {

        let ip_addr = match ip_addr.into().as_str() {
            "127.0.0.1" => "localhost",
            other => other,
        }.to_string();

        Device {
            name: name.into(),
            manufacturer: manufacturer.into(),
            model: model.into(),
            ip_addr,
            port: 7345,
            uuid: uuid.into(),
            auth_token: None,
            // info: Arc::new(RwLock::new(
            //     DeviceInfo::build(name, manufacturer, model, ip_addr, uuid)
            // )),
            client: Self::new_client(),
        }
    }

    fn new_client() -> Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(constant::DEFAULT_TIMEOUT))
            .https_only(true)
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Unable to build Reqwest Client")
    }

    /// Connect to a SmartCast device from the device's IP Address
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::Device;
    /// #
    /// # async fn connect_ip() -> Result<Device, smartcast::Error> {
    /// let ip_addr = "192.168.0.14";
    /// let dev: Device = Device::from_ip(ip_addr).await?;
    /// println!("{}", dev.name());
    /// // > "Living Room TV"
    /// #
    /// # Ok(dev)
    /// # }
    /// ```
    pub async fn from_ip<S: Into<String>>(ip_addr: S) -> Result<Self> {
        match uaudp_followup(
            &format!("http://{}:8008/ssdp/device-desc.xml", ip_addr.into())
        ).await? {
            Some(device) => Ok(device),
            None => Err(Error::Other("Device not found".into())),
        }
    }

    /// Connect to a SmartCast device from the device's UUID
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::Device;
    /// #
    /// # async fn connect_uuid() -> Result<Device, smartcast::Error> {
    /// let uuid = "cb72c9c8-2d45-65b6-424a-13fa25a650db";
    /// let dev: Device = Device::from_uuid(uuid).await?;
    /// println!("{}", dev.name());
    /// // > "Living Room TV"
    /// #
    /// # Ok(dev)
    /// # }
    /// ```
    pub async fn from_uuid<S: Into<String>>(uuid: S) -> Result<Self> {
        let mut device_vec = ssdp(constant::SSDP_IP, &format!("uuid:{}", uuid.into()), constant::DEFAULT_SSDP_MAXTIME ).await?;
        if !device_vec.is_empty() {
            Ok(device_vec.swap_remove(0))
        } else {
            Err(Error::Other("Device not found".into()))
        }
    }

    /// Get device's 'friendly' name
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get device's model name
    pub fn model_name(&self) -> String {
        self.model.clone()
    }

    #[cfg(test)]
    pub fn manufacturer(&self) -> String {
        self.manufacturer.clone()
    }

    /// Get device's local IP
    pub fn ip(&self) -> String {
        self.ip_addr.clone()
    }

    /// Get device's port
    pub fn port(&self) -> u16 {
        // TO-DO: Verify port (dependant on device firmware)
        self.port
    }

    /// Get device's UUID
    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    /// If set, get the client's auth token for the device
    pub fn auth_token(&self) -> Option<String> {
        self.auth_token.clone()
    }

    /// If already paired, you may manually set the client's auth token for the device.
    pub fn set_auth_token<S: Into<String>>(&mut self, token: S) -> Result<()> {
        self.auth_token = Some(token.into());
        // TO-DO: Verify token
        Ok(())
    }

    /// Begin the pairing process
    ///
    /// Upon calling this method, the device will enter pairing mode.
    /// It may not be necessary to pair your device if it is a soundbar.
    ///
    /// This method returns a `Pairing Token` and a `Challenge Type` which
    /// will need to be passed into [`finish_pair()`](./struct.Device.html/#method.finish_pair)
    /// along with the pin displayed on the device screen.
    pub async fn begin_pair<S: Into<String>>(&self, client_name: S, client_id: S) -> Result<(u32, u32)> {
        let mut res = self.send_command(
            Command::StartPairing {
                client_name: client_name.into(),
                client_id: client_id.into(),
            }
        ).await?.unwrap();

        let pairing_token: u32 = serde_json::from_value(res["PAIRING_REQ_TOKEN"].take()).unwrap();
        let challenge: u32 = serde_json::from_value(res["CHALLENGE_TYPE"].take()).unwrap();

        Ok((pairing_token, challenge))
    }

    /// Finish the pairing process
    ///
    /// Upon calling this method with `Client Name`, `Pairing Token` and `Challenge Type` returned from
    /// [`begin_pair()`](./struct.Device.html/#method.begin_pair), and the pin displayed
    /// by the device, the pairing process will end and the client will be paired.
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::Device;
    /// # use std::io::stdin;
    /// #
    /// # async fn pair() -> Result<String, smartcast::Error> {
    /// let mut dev = Device::from_ip("192.168.0.14").await.unwrap();
    ///
    /// let client_name = "My App Name";
    /// let client_id = "myapp-rs";
    ///
    /// // Begin Pairing
    /// let (pairing_token, challenge) = dev.begin_pair(client_name, client_id).await?;
    ///
    /// // Input pin displayed on screen
    /// let mut pin = String::new();
    /// stdin().read_line(&mut pin).unwrap();
    ///
    /// // Finish Pairing
    /// let auth_token = dev.finish_pair(client_id, pairing_token, challenge, &pin).await?;
    /// println!("{}", auth_token);
    /// // > "Z2zscc1udl"
    /// # Ok(auth_token)
    /// # }
    /// ```
    pub async fn finish_pair<S: Into<String>>(&mut self, client_id: S, pairing_token: u32, challenge: u32, pin: S) -> Result<String> {
        // Strip non digits
        let pin: String = pin.into().chars().filter(|c| c.is_digit(10)).collect();

        let mut res = self.send_command(
            Command::FinishPairing {
                client_id: client_id.into(),
                pairing_token,
                challenge,
                response_value: pin,
            }
        ).await?.unwrap();

        let auth_token: String = serde_json::from_value(res["AUTH_TOKEN"].take()).unwrap();

        self.auth_token = Some(auth_token.clone());
        Ok(auth_token)
    }

    /// Cancel the pairing process
    ///
    /// Upon calling this method, the pairing process will be canceled and the
    /// device will leave pairing mode.
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::Device;
    /// #
    /// # async fn pair_cancel() -> Result<(), smartcast::Error> {
    /// let mut dev = Device::from_ip("192.168.0.14").await.unwrap();
    ///
    /// let client_name = "My App Name";
    /// let client_id = "myapp-rs";
    ///
    /// // Begin Pairing
    /// let (pairing_token, challenge) = dev.begin_pair(client_name, client_id).await?;
    ///
    /// // Cancel Pairing
    /// dev.cancel_pair(client_name, client_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// **The SmartCast API has changed. This method may return an error until more info
    /// is learned about how to access this function on the device. For now a soft reboot
    /// or waiting for a timeout is the only way to cancel pairing mode.**
    pub async fn cancel_pair<S: Into<String>>(&self, client_name: S, client_id: S) -> Result<()> {
        self.send_command(
            Command::CancelPairing {
                client_name: client_name.into(),
                client_id: client_id.into(),
            }
        ).await?;
        Ok(())
    }

    /// Check whether the device is powered on
    pub async fn is_powered_on(&self) -> Result<bool> {
        let mut res = self.send_command(Command::GetPowerState).await?.unwrap();
        let power_state: u32 = serde_json::from_value(res[0]["VALUE"].take()).unwrap();
        Ok(power_state == 1)
    }

    /// Emulates a button press on a remote control
    ///
    /// Pass in a [`ButtonEvent`] or vector of [`ButtonEvent`]s to interact with the device.
    /// In the latter case, commands will be processed in order.
    ///
    /// # Example
    ///
    /// ```
    /// use smartcast::{Device, ButtonEvent, Button};
    ///
    /// # async fn power_on_volume_up() -> Result<Device, smartcast::Error> {
    /// let mut dev = Device::from_ip("192.168.0.14").await.unwrap();
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// // Power on device
    /// if !dev.is_powered_on().await? {
    ///     dev.button_event(ButtonEvent::KeyPress(Button::PowerOn)).await?;
    /// }
    ///
    /// // Increase Volume
    /// dev.button_event(ButtonEvent::KeyPress(Button::VolumeUp)).await?;
    ///
    /// // Increase Volume More
    /// dev.button_event(vec![
    ///     ButtonEvent::KeyPress(Button::VolumeUp),
    ///     ButtonEvent::KeyPress(Button::VolumeUp),
    /// ]).await?;
    /// # Ok(dev)
    /// # }
    /// ```
    pub async fn button_event<V: Into<Vec<ButtonEvent>>>(&self, buttons: V) -> Result<()> {
        let button_vec: Vec<ButtonEvent> = buttons.into();
        self.send_command(Command::RemoteButtonPress(button_vec)).await?;
        Ok(())
    }

    /// Get the current device input
    pub async fn current_input(&self) -> Result<Input> {
        let mut res = self.send_command(Command::GetCurrentInput).await?.unwrap();
        Ok(Input::from_value(&mut res[0]))
    }

    /// Get list of available inputs
    pub async fn list_inputs(&self) -> Result<Vec<Input>> {
        let mut res = self.send_command(Command::GetInputList).await?.unwrap();
        Ok(Input::from_array(&mut res))
    }

    /// Changes the input of the device
    ///
    /// # Example
    ///
    /// ```
    /// # use smartcast::Device;
    /// #
    /// # async fn change_input() -> Result<(), smartcast::Error> {
    /// let mut dev = Device::from_ip("192.168.0.14").await.unwrap();
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// println!("{}", dev.current_input().await?.friendly_name());
    /// // > "Nintendo Switch"
    ///
    /// dev.change_input("HDMI-2").await?;
    /// println!("{}", dev.current_input().await?.friendly_name());
    /// // > "Playstation 4"
    /// # Ok(())
    /// # }
    /// ```
    /// Note: the input's default name must be passed in, not the input's custom name -- e.g.
    /// "HDMI-2" instead of "Playstation 4"
    pub async fn change_input<S: Into<String>>(&self, name: S) -> Result<()> {
        self.send_command(
            Command::ChangeInput{
                name: name.into(),
                hashval: self.current_input().await?.hashval(),
            }
        ).await?;
        Ok(())
    }

    /// TO-DO Document
    pub async fn read_settings(&self, subsetting: SubSetting) -> Result<Vec<SubSetting>> {

        if subsetting.hidden() {
            return Err(Error::Blocked);
        } else if subsetting.object_type() != ObjectType::Menu {
            return Ok(vec![subsetting]);
        }

        let res = self.send_command(Command::ReadSettings(subsetting.clone())).await?.unwrap();

        let mut settings_res: Vec<SubSetting> = serde_json::from_value(res)?;

        let parent_endpoint = subsetting.endpoint(UrlBase::None);
        for setting in settings_res.iter_mut() {
            setting.add_parent_endpoint(parent_endpoint.clone());

            if setting.hidden() {
                continue;
            }

            match setting.object_type() {
                ObjectType::Slider => {
                    if setting.slider_info().is_none() {
                        let static_setting = self.send_command(Command::ReadStaticSettings(setting.clone())).await?.unwrap();
                        let slider_info: setting::SliderInfo = serde_json::from_value(static_setting[0].clone())?;
                        setting.add_slider_info(slider_info);
                    }
                },
                ObjectType::List
                | ObjectType::XList => {
                    if setting.elements().is_none() {
                        let static_setting = self.send_command(Command::ReadStaticSettings(setting.clone())).await?.unwrap();
                        let elements: Vec<String> = serde_json::from_value(static_setting[0]["ELEMENTS"].clone())?;
                        setting.add_elements(elements);
                    }
                }
                _ => {},
            }
        }

        Ok(settings_res)
    }

    async fn send_command(&self, command: Command) -> Result<Option<Value>> {
        let url: String = format!("https://{}:{}{}", self.ip_addr, self.port, command.endpoint());

        // Start building request
        let mut req = match command.request_type() {
            RequestType::Get => {
                self.client.get(url)
            },
            RequestType::Put => {
                self.client.put(url)
                // Add body for PUT commands
                .body(
                    serde_json::to_string(&command).unwrap()
                )
            },
        };

        // Add content type header
        req = req.header("Content-Type", "application/json");

        // If paired, add auth token header
        match &self.auth_token {
            Some(token) => {
                req = req.header("Auth", token.to_string());
            },
            None => {},
        }

        // Send command
        let res = req.send().await?;

        // Process response
        response::process(res.json().await?)
    }
}

#[derive(Debug)]
struct DeviceInfo {
    name: String,
    manufacturer: String,
    model: String,
    ip_addr: String,
    port: u16,
    uuid: String,
    auth_token: Option<String>
}

impl DeviceInfo {
    fn build(name: String,manufacturer: String,model: String,ip_addr: String,uuid: String) -> Self {
        Self {
            name,
            manufacturer,
            model,
            ip_addr,
            // TO-DO support port 8000
            port: 7345,
            uuid,
            auth_token: None,
        }
    }
}

#[cfg(test)]
impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
        && self.manufacturer == other.manufacturer
        && self.model == other.model
        && self.ip_addr == other.ip_addr
        && self.port == other.port
        && self.uuid == other.uuid
        && self.auth_token == other.auth_token
    }
}
