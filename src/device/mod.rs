use super::constant;
use super::discover::{ssdp, uaudp_followup};
use super::error::{Error, Result};

mod apps;
mod command;
mod input;
mod response;
mod setting;

pub use self::apps::App;
pub use self::command::{Button, ButtonEvent, DeviceType};
pub use self::input::Input;
pub use self::setting::{ObjectType, SubSetting, UrlBase};

use self::apps::fetch_apps;
use self::command::{Command, RequestType};
use self::response::Response;

use reqwest::Client;
use std::time::Duration;

/// A Vizio Device
// TODO: Document
#[derive(Debug, Clone)]
pub struct Device {
    name: String,
    manufacturer: String,
    model: String,
    dev_type: DeviceType,
    ip_addr: String,
    port: u16,
    uuid: String,
    auth_token: Option<String>,
    client: Client,
}

impl Device {
    pub(crate) async fn new<S: Into<String>>(
        name: S,
        manufacturer: S,
        model: S,
        ip_addr: S,
        uuid: S,
    ) -> Result<Device> {
        // Workaround for testing issues on loopback
        let ip_addr = match ip_addr.into().as_str() {
            "127.0.0.1" => "localhost",
            other => other,
        }
        .to_string();

        let mut device = Device {
            name: name.into(),
            manufacturer: manufacturer.into(),
            model: model.into(),
            dev_type: DeviceType::TV,
            ip_addr,
            port: 0,
            uuid: uuid.into(),
            auth_token: None,
            client: Self::new_client(),
        };

        // Check port options
        device.find_port().await?;

        Ok(device)
    }

    fn new_client() -> Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(constant::DEFAULT_TIMEOUT))
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Unable to build Reqwest Client")
    }

    #[cfg(not(test))]
    async fn find_port(&mut self) -> Result<Response> {
        let mut iter = constant::PORT_OPTIONS.iter().peekable();

        let res = loop {
            if let Some(port) = iter.next() {
                self.port = *port;
                let res = self.send_command(Command::GetDeviceInfo).await;
                if res.is_ok() || iter.peek().is_none() {
                    break res;
                }
            } else {
                panic!("Reached end of port iterator");
            }
        };
        res
    }

    #[cfg(test)]
    async fn find_port(&mut self) -> Result<()> {
        Ok(())
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
        match uaudp_followup(&format!(
            "http://{}:8008/ssdp/device-desc.xml",
            ip_addr.into()
        ))
        .await?
        {
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
        let mut device_vec = ssdp(
            constant::SSDP_IP,
            &format!("uuid:{}", uuid.into()),
            constant::DEFAULT_SSDP_MAXTIME,
        )
        .await?;
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

    /// If previously paired, you may manually set the client's auth token for the device.
    pub async fn set_auth_token<S: Into<String>>(&mut self, token: S) -> Result<()> {
        let old_token = self.auth_token.clone();
        self.auth_token = Some(token.into());

        // Send a command which requires pairing to test token
        match self
            .send_command(Command::ReadSettings(SubSetting::default()))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                self.auth_token = old_token;
                Err(e)
            }
        }
    }

    /// Begin the pairing process
    ///
    /// Upon calling this method, the device will enter pairing mode.
    /// It may not be necessary to pair your device if it is a soundbar.
    ///
    /// This method returns a `Pairing Token` and a `Challenge Type` which
    /// will need to be passed into [`finish_pair()`](./struct.Device.html/#method.finish_pair)
    /// along with the pin displayed on the device screen.
    pub async fn begin_pair<S: Into<String>>(
        &self,
        client_name: S,
        client_id: S,
    ) -> Result<(u32, u32)> {
        let res = self
            .send_command(Command::StartPairing {
                client_name: client_name.into(),
                client_id: client_id.into(),
            })
            .await?;
        Ok((res.pairing_token()?, res.challenge()?))
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
    pub async fn finish_pair<S: Into<String>>(
        &mut self,
        client_id: S,
        pairing_token: u32,
        challenge: u32,
        pin: S,
    ) -> Result<String> {
        // Strip non digits
        let pin: String = pin.into().chars().filter(|c| c.is_digit(10)).collect();

        let res = self
            .send_command(Command::FinishPairing {
                client_id: client_id.into(),
                pairing_token,
                challenge,
                response_value: pin,
            })
            .await?;
        Ok(res.auth_token()?)
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
    /// dev.cancel_pair(client_id, pairing_token, challenge).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_pair<S: Into<String>>(
        &self,
        client_id: S,
        pairing_token: u32,
        challenge: u32,
    ) -> Result<()> {
        self.send_command(Command::CancelPairing {
            client_id: client_id.into(),
            pairing_token,
            challenge,
        })
        .await?;
        Ok(())
    }

    /// Check whether the device is powered on
    pub async fn is_powered_on(&self) -> Result<bool> {
        let res = self.send_command(Command::GetPowerState).await?;
        Ok(res.power_state()?)
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
        self.send_command(Command::RemoteButtonPress(button_vec))
            .await?;
        Ok(())
    }

    /// Get the current device input
    pub async fn current_input(&self) -> Result<Input> {
        let res = self.send_command(Command::GetCurrentInput).await?;
        Ok(res.current_input()?)
    }

    /// Get list of available inputs
    pub async fn list_inputs(&self) -> Result<Vec<Input>> {
        let res = self.send_command(Command::GetInputList).await?;
        Ok(res.input_list()?)
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
        self.send_command(Command::ChangeInput {
            name: name.into(),
            hashval: self.current_input().await?.hashval(),
        })
        .await?;
        Ok(())
    }

    /// TO-DO: Document
    pub async fn device_info(&self) -> Result<()> {
        let res = self.send_command(Command::GetDeviceInfo).await?;
        println!("{:#?}", res.device_info());
        Ok(())
    }

    /// TO-DO: Document
    pub async fn current_app(&self) -> Result<()> {
        let res = self.send_command(Command::GetCurrentApp).await?;
        println!("{:#?}", res);
        Ok(())
    }

    /// TO-DO: Document
    pub async fn list_apps(&self) -> Result<Vec<App>> {
        Ok(fetch_apps(&self.client).await?)
    }

    /// TO-DO: Document
    pub async fn launch_app(&self, app: &App) -> Result<()> {
        let res = self.send_command(Command::LaunchApp(app.payload())).await?;
        println!("{:#?}", res);
        Ok(())
    }

    /// TO-DO Document
    pub async fn read_settings(&self, subsetting: &SubSetting) -> Result<Vec<SubSetting>> {
        // TO-DO Feature to diable block
        if subsetting.hidden() {
            return Err(Error::Blocked);
        } else if subsetting.object_type() != ObjectType::Menu {
            return Ok(vec![subsetting.clone()]);
        }

        let res: Response = self
            .send_command(Command::ReadSettings(subsetting.clone()))
            .await?;
        let mut settings = res.settings()?;

        let parent_endpoint = subsetting.endpoint(UrlBase::None, &self.dev_type);
        for setting in settings.iter_mut() {
            setting.add_parent_endpoint(parent_endpoint.clone());

            if setting.hidden() {
                continue;
            }

            match setting.object_type() {
                ObjectType::Slider => {
                    if setting.slider_info().is_none() {
                        let res = self
                            .send_command(Command::ReadStaticSettings(setting.clone()))
                            .await?;
                        setting.add_slider_info(res.slider_info()?);
                    }
                }
                ObjectType::List | ObjectType::XList => {
                    if setting.elements().is_none() {
                        let res = self
                            .send_command(Command::ReadStaticSettings(setting.clone()))
                            .await?;
                        setting.add_elements(res.elements()?);
                    }
                }
                _ => {}
            }
        }

        Ok(settings)
    }

    async fn send_command(&self, command: Command) -> Result<Response> {
        let url: String = format!(
            "https://{}:{}{}",
            self.ip_addr,
            self.port,
            command.endpoint(&self.dev_type)
        );

        println!("{:#?}", serde_json::to_string(&command).unwrap());

        // Start building request
        let mut req = match command.request_type() {
            RequestType::Get => self.client.get(url),
            RequestType::Put => {
                self.client
                    .put(url)
                    // Add body for PUT commands
                    .body(serde_json::to_string(&command).unwrap())
            }
        };

        // Add content type header
        req = req.header("Content-Type", "application/json");

        // If paired, add auth token header
        match &self.auth_token {
            Some(token) => {
                req = req.header("Auth", token.to_string());
            }
            None => {}
        }

        // Send command
        let res = req.send().await?;

        let response = res.text().await.unwrap();
        println!("{:#?}", response);
        let json: serde_json::Value = serde_json::from_str(&response).unwrap();

        // Process response
        response::process(json)
    }
}

// #[derive(Debug)]
// struct DeviceInfo {
//     name: String,
//     manufacturer: String,
//     model: String,
//     ip_addr: String,
//     port: u16,
//     auth_token: Option<String>
// }

// impl DeviceInfo {
//     fn new(name: String, manufacturer: String, model: String, ip_addr: String) -> Self {
//         Self {
//             name,
//             manufacturer,
//             model,
//             ip_addr,
//             port: 7345,
//             auth_token: None,
//         }
//     }
// }

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
