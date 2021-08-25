use super::discover::{ssdp, uaudp_followup, DEFAULT_SSDP_MAXTIME, SSDP_IP};
use super::error::{Error, Result};

mod apps;
mod command;
mod info;
mod remote;
mod response;
mod settings;

pub use self::apps::App;
pub use self::info::{DeviceInfo, Input};
pub use self::remote::Button;
pub use self::settings::{SettingType, SliderInfo, SubSetting};

use self::apps::{AppList, Payload};
use self::command::{Command, CommandDetail};
use self::remote::KeyEvent;
use self::response::Response;
use self::settings::EndpointBase;

use reqwest::Client;
use tokio::sync::RwLock;

use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

#[allow(dead_code)]
pub const PORT_OPTIONS: [u16; 2] = [7345, 9000];
pub const DEFAULT_TIMEOUT: u64 = 3;

/// A SmartCast Device
///
/// More specifically, a client for connecting to a SmartCast device. Search for devices on your
/// local network using [`discover_devices()`](crate::discover_devices). You can also connect directly
/// using [`Device::from_ip()`](Device::from_ip) or [`Device::from_uuid()`](Device::from_uuid).
///
/// Note that `Device` is [Arc] wrapped for flexibility so cloning is thread safe.
#[derive(Clone)]
pub struct Device {
    inner: Arc<DeviceRef>,
}

impl Device {
    pub(super) async fn new<S: Into<String>>(
        name: S,
        manufacturer: S,
        model: S,
        ip_addr: S,
        uuid: S,
    ) -> Result<Self> {
        log::trace!("Attempting to connect to API");

        // Workaround for testing issues on loopback
        let ip_addr = match ip_addr.into().as_str() {
            "127.0.0.1" => "localhost",
            other => other,
        }
        .to_string();

        // Build Client
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
            .danger_accept_invalid_certs(true)
            .pool_idle_timeout(Some(Duration::from_secs(5)))
            .build()?;

        // Build Device
        let device = Self {
            inner: Arc::new(DeviceRef {
                name: name.into(),
                manufacturer: manufacturer.into(),
                model: model.into(),
                settings_root: RwLock::new(String::new()),
                ip_addr,
                port: RwLock::new(0),
                uuid: uuid.into(),
                auth_token: RwLock::new(None),
                app_list: RwLock::new(AppList::new(client.clone())),
                client,
            }),
        };

        device.initialize().await
    }

    async fn initialize(self) -> Result<Self> {
        log::trace!("Initializing");
        // Check port options
        self.find_port().await?;

        // Get settings root
        self.set_settings_root().await?;

        Ok(self)
    }

    #[cfg(not(test))]
    async fn find_port(&self) -> Result<()> {
        let mut iter = PORT_OPTIONS.iter().peekable();

        loop {
            if let Some(port) = iter.next() {
                log::trace!("Attempt connection to port {}", port);

                {
                    // Code block to drop lock
                    let mut current_port = self.inner.port.write().await;
                    *current_port = *port;
                }

                let res = self.device_info().await;
                match res {
                    Err(Error::Reqwest(e)) if e.is_connect() && iter.peek().is_some() => {}
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(e),
                }
            } else {
                log::error!("Port iterator has been expended");
                panic!("Reached end of port iterator");
            }
        }
    }

    #[cfg(not(test))]
    async fn set_settings_root(&self) -> Result<()> {
        let device_info = self.device_info().await?;
        log::trace!("Set settings root URI");

        let mut settings_root = self.inner.settings_root.write().await;
        *settings_root = device_info.settings_root;

        Ok(())
    }

    /// Connect to a SmartCast device from the device's IP Address
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::Device;
    ///
    /// let ip_addr = "192.168.0.14";
    /// let dev: Device = Device::from_ip(ip_addr).await?;
    /// println!("{}", dev.name());
    /// // > "Living Room TV"

    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_ip<S: Into<String>>(ip_addr: S) -> Result<Self> {
        let ip_addr: String = ip_addr.into();
        log::info!("Attempt API connection to IP '{}'", ip_addr);

        match uaudp_followup(&format!("http://{}:8008/ssdp/device-desc.xml", ip_addr)).await? {
            Some(device) => Ok(device),
            None => {
                log::error!("Device not found at '{}'", ip_addr);
                Err(Error::device_not_found_ip(ip_addr))
            }
        }
    }

    /// Connect to a SmartCast device from the device's UUID
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::Device;
    ///
    /// let uuid = "cb72c9c8-2d45-65b6-424a-13fa25a650db";
    /// let dev: Device = Device::from_uuid(uuid).await?;
    /// println!("{}", dev.name());
    /// // > "Living Room TV"

    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_uuid<S: Into<String>>(uuid: S) -> Result<Self> {
        let uuid: String = uuid.into();
        log::info!("Attempt API connection to device with UUID '{}'", uuid);

        let mut device_vec = ssdp(SSDP_IP, &format!("uuid:{}", uuid), DEFAULT_SSDP_MAXTIME).await?;
        if !device_vec.is_empty() {
            Ok(device_vec.swap_remove(0))
        } else {
            log::error!("Device not found with UUID '{}'", uuid);
            Err(Error::device_not_found_uuid(uuid))
        }
    }

    /// Get device's 'friendly' name
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get device's model name
    pub fn model_name(&self) -> String {
        self.inner.model.clone()
    }

    /// Get device's local IP
    pub fn ip(&self) -> String {
        self.inner.ip_addr.clone()
    }

    /// Get device's API port
    pub fn port(&self) -> u16 {
        if let Ok(port) = self.inner.port.try_read() {
            *port
        } else {
            // Port shouldn't ever be written outside initialization
            // so use try_read() to avoid awaiting and panic if it
            // is locked
            panic!("Unable to unlock port for read");
        }
    }

    /// Get device's UUID
    pub fn uuid(&self) -> String {
        self.inner.uuid.clone()
    }

    /// If set, get the client's auth token for the device
    pub async fn auth_token(&self) -> Option<String> {
        self.inner.auth_token.read().await.clone()
    }

    /// If previously paired, you may manually set the client's auth token for the device.
    pub async fn set_auth_token<S: Into<String>>(&self, new_token: S) -> Result<()> {
        let new_token: String = new_token.into();
        log::trace!("Set auth token '{}'", new_token);

        let old_token = self.auth_token().await;

        {
            let mut token = self.inner.auth_token.write().await;
            *token = Some(new_token);
        }

        // Send a command which requires pairing to test token
        match self.current_input().await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::warn!("Auth token was rejected by the device, reverting");
                {
                    let mut token = self.inner.auth_token.write().await;
                    *token = old_token;
                }
                Err(e)
            }
        }
    }

    /// Get various information about the device in the form of [`DeviceInfo`]
    pub async fn device_info(&self) -> Result<DeviceInfo> {
        log::trace!("Get Device Info");
        self.send_command(CommandDetail::GetDeviceInfo)
            .await?
            .into()
    }

    /// Begin the pairing process
    ///
    /// The device will enter pairing mode upon calling this method with a `Client Name` which will be displayed
    /// in the device's "Mobile Devices" page, along with a `Client ID` which will be used to identify the client.
    ///
    /// This method returns `pairing data` consisting of a `Pairing Token`, a `Challenge Type`, and the `Client ID` which
    /// will need to be passed into [`finish_pair()`](Self::finish_pair)
    /// or [`cancel_pair()`](Self::cancel_pair).
    ///
    /// Note: It may not be necessary to pair your device if it is a soundbar.
    pub async fn begin_pair<S: Into<String>>(
        &self,
        client_name: S,
        client_id: S,
    ) -> Result<(u32, u32, String)> {
        let client_name: String = client_name.into();
        let client_id: String = client_id.into();
        log::trace!("Begin Pairing");
        log::debug!("client_name: {}, client_id: {}", client_name, client_id);

        self.send_command(CommandDetail::StartPairing {
            client_name,
            client_id: client_id.clone(),
        })
        .await?
        .pairing()
        .map(|(token, challenge)| (token, challenge, client_id))
    }

    /// Finish the pairing process
    ///
    /// Upon calling this method with the `pairing data` returned from
    /// [`begin_pair()`](Self::begin_pair) and the pin displayed
    /// by the device, the pairing process will end and the client will be paired.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<String, smartcast::Error> {

    /// use smartcast::Device;
    /// use std::io::stdin;
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    ///
    /// let client_name = "My App Name";
    /// let client_id = "myapp-rs";
    ///
    /// // Begin Pairing
    /// let pairing_data = dev.begin_pair(client_name, client_id).await?;
    ///
    /// // Input pin displayed on screen
    /// let mut pin = String::new();
    /// stdin().read_line(&mut pin);
    ///
    /// // Finish Pairing
    /// let auth_token = dev.finish_pair(pairing_data, &pin).await?;
    /// println!("{}", auth_token);
    /// // > "Z2zscc1udl"

    /// # Ok(auth_token)
    /// # }
    /// ```
    pub async fn finish_pair<S: Into<String>>(
        &mut self,
        pairing_data: (u32, u32, String),
        pin: S,
    ) -> Result<String> {
        let (pairing_token, challenge, client_id) = pairing_data;
        // Strip non digits
        let pin: String = pin.into().chars().filter(|c| c.is_digit(10)).collect();
        log::trace!("Finsh Pairing");
        log::debug!(
            "pairing_token: {}, challenge: {}, client_id: {}, pin: {}",
            pairing_token,
            challenge,
            client_id,
            pin
        );

        self.send_command(CommandDetail::FinishPairing {
            client_id,
            pairing_token,
            challenge,
            response_value: pin,
        })
        .await?
        .auth_token()
    }

    /// Cancel the pairing process
    ///
    /// Upon calling this method with the `pairing data` returned from
    /// [`begin_pair()`](Self::begin_pair),
    /// the pairing process will be canceled and the device will leave pairing mode.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::Device;
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    ///
    /// let client_name = "My App Name";
    /// let client_id = "myapp-rs";
    ///
    /// // Begin Pairing
    /// let pairing_data = dev.begin_pair(client_name, client_id).await?;
    ///
    /// // Cancel Pairing
    /// dev.cancel_pair(pairing_data).await?;

    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_pair(&self, pairing_data: (u32, u32, String)) -> Result<()> {
        let (pairing_token, challenge, client_id) = pairing_data;
        log::trace!("Cancel Pairing");
        log::debug!(
            "pairing_token: {}, challenge: {}, client_id: {}",
            pairing_token,
            challenge,
            client_id
        );

        self.send_command(CommandDetail::CancelPairing {
            client_id,
            pairing_token,
            challenge,
        })
        .await
        .map(drop)
    }

    /// Check whether the device is powered on
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::{Device, Button};
    ///
    /// let dev = Device::from_ip("192.168.0.14").await?;
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// // Power on device
    /// if !dev.is_powered_on().await? {
    ///     dev.key_press(Button::PowerOn).await?;
    /// }

    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_powered_on(&self) -> Result<bool> {
        log::trace!("Power status");
        self.send_command(CommandDetail::GetPowerState)
            .await?
            .power_state()
    }

    /// Emulates a simple remote control button press
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::{Device, Button};
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// // Increase Volume
    /// dev.key_press(Button::VolumeUp).await?;

    /// # Ok(())
    /// # }
    /// ```
    pub async fn key_press(&self, button: Button) -> Result<()> {
        log::trace!("Virtual Remote Key Press");
        self.virtual_remote(KeyEvent::Press, button).await.map(drop)
    }

    /// Emulates holding down a remote control button
    ///
    /// If a duration is specified, the remote button will be held down for the duration.
    /// Otherwise it will be held down indefinitely and [`key_up()`](Self::key_up) must be called.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::{Device, Button};
    /// use std::time::Duration;
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// // Increase Volume for 5 seconds
    /// dev.key_down(Button::VolumeUp, Some(Duration::from_secs(5))).await?;

    /// # Ok(())
    /// # }
    /// ```
    pub async fn key_down(&self, button: Button, duration: Option<Duration>) -> Result<()> {
        log::trace!("Virtual Remote Key Down");
        log::debug!("key_down duration: {:?}", duration);

        self.virtual_remote(KeyEvent::Down, button).await?;
        if let Some(duration) = duration {
            // Sleep for duration
            tokio::time::sleep(duration).await;
            self.key_up(button).await?;
        }
        Ok(())
    }

    /// Emulates releasing a remote control button
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::{Device, Button};
    /// use tokio::time::sleep;
    /// use std::time::Duration;
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// // Increase Volume for 5 seconds
    /// dev.key_down(Button::VolumeUp, None).await?;
    /// sleep(Duration::from_secs(5)).await;
    /// dev.key_up(Button::VolumeUp).await?;

    /// # Ok(())
    /// # }
    /// ```
    pub async fn key_up(&self, button: Button) -> Result<()> {
        log::trace!("Virtual Remote Key Up");
        self.virtual_remote(KeyEvent::Up, button).await.map(drop)
    }

    /// Get information about the app currently running on the device
    ///
    /// App info is sourced from a 3rd party. This method will return
    /// `None` if the app data isn't available from that source.
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::Device;
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// if let Some(app) = dev.current_app().await? {
    ///     println!("{:#?}", app);
    ///     // > App {
    ///     // >     name: "Netflix",
    ///     // >     description: "Award-winning series, movies and more",
    ///     // >     image_url: "http://{icon_url}",
    ///     // > },
    /// }

    /// # Ok(())
    /// # }
    /// ```
    pub async fn current_app(&self) -> Result<Option<App>> {
        // Get payload from device
        let current_payload: Payload = self
            .send_command(CommandDetail::GetCurrentApp)
            .await?
            .app_payload()?;

        // Get app by payload
        self.inner
            .app_list
            .write()
            .await
            .get_app(current_payload)
            .await
    }

    /// Get the current device input
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::Device;
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// println!("{}", dev.current_input().await?.friendly_name());
    /// // > "Nintendo Switch"

    /// # Ok(())
    /// # }
    /// ```
    pub async fn current_input(&self) -> Result<Input> {
        log::trace!("Get Current Input");
        self.send_command(CommandDetail::GetCurrentInput)
            .await
            .map(|response| response.into())?
    }

    /// Get list of available inputs
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::{Device, Input};
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
    /// dev.set_auth_token("Z2zscc1udl");
    ///
    /// let inputs: Vec<Input> = dev.list_inputs().await?;
    ///
    /// println!("{}", inputs[0].friendly_name());
    /// // > "Nintendo Switch"

    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_inputs(&self) -> Result<Vec<Input>> {
        log::trace!("List Inputs");
        self.send_command(CommandDetail::GetInputList)
            .await
            .map(|response| response.into())?
    }

    /// Changes the input of the device
    ///
    /// # Example
    ///
    /// ```
    /// # async fn example() -> Result<(), smartcast::Error> {

    /// use smartcast::Device;
    ///
    /// let mut dev = Device::from_ip("192.168.0.14").await?;
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
        let name: String = name.into();
        log::trace!("Change Input");
        log::debug!("change_input name: {}", name);

        self.send_command(CommandDetail::ChangeInput {
            name,
            hashval: self.current_input().await?.hashval(),
        })
        .await?;
        Ok(())
    }

    /// Get the root of the device's [`Settings`](SubSetting).
    pub async fn settings(&self) -> Result<Vec<SubSetting>> {
        log::trace!("Settings Root");
        settings::root(self.clone()).await
    }

    pub(super) fn settings_root(&self) -> String {
        if let Ok(settings_root) = self.inner.settings_root.try_read() {
            settings_root.clone()
        } else {
            // Same as port(), settings_root shouldn't ever be written outside initialization
            // so use try_read() to avoid awaiting and panic if it is locked
            panic!("Unable to settings root for read");
        }
    }

    async fn virtual_remote(&self, event: KeyEvent, button: Button) -> Result<()> {
        log::trace!("Virtual Remote Handler");
        log::debug!("Event: {:?}, Button: {:?}", event, button);

        match (
            self.send_command(CommandDetail::RemoteButtonPress(event, button))
                .await,
            button.alt(),
        ) {
            (Ok(_), _) => Ok(()),
            (Err(e), Some(button_alt)) if e.is_api() => self
                .send_command(CommandDetail::RemoteButtonPress(event, button_alt))
                .await
                .map(drop),
            (Err(other), _) => Err(other),
        }
    }

    fn send_command(&self, detail: CommandDetail) -> impl Future<Output = Result<Response>> {
        log::debug!("send_command detail: '{:?}'", detail);
        Command::new(self.clone(), detail).send()
    }

    #[cfg(test)]
    async fn find_port(&self) -> Result<()> {
        Ok(())
    }

    #[cfg(test)]
    async fn set_settings_root(&self) -> Result<()> {
        Ok(())
    }

    #[cfg(test)]
    pub fn manufacturer(&self) -> String {
        self.inner.manufacturer.clone()
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Device");
        d.field("name", &self.name());
        d.field("manufacturer", &self.inner.manufacturer.clone());
        d.field("model", &self.model_name());
        d.field("settings_root", &self.settings_root());
        d.field("ip_addr", &self.ip());
        d.field("port", &self.port());
        d.field("uuid", &self.uuid());
        d.field(
            "auth_token",
            &match self.inner.auth_token.try_read() {
                Ok(token) => token.clone(),
                Err(_) => Some("***Locked***".into()),
            },
        );
        d.finish()
    }
}

#[derive(Debug)]
pub struct DeviceRef {
    name: String,
    manufacturer: String,
    model: String,
    settings_root: RwLock<String>,
    ip_addr: String,
    port: RwLock<u16>,
    uuid: String,
    auth_token: RwLock<Option<String>>,
    app_list: RwLock<AppList>,
    client: Client,
}

impl DeviceRef {}

#[cfg(test)]
impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.manufacturer() == other.manufacturer()
            && self.model_name() == other.model_name()
            && self.ip() == other.ip()
            && self.port() == other.port()
            && self.uuid() == other.uuid()
            && *self.inner.auth_token.try_read().unwrap()
                == *other.inner.auth_token.try_read().unwrap()
    }
}
