mod command;
mod response;

use self::command::{Command, RequestType};
pub use self::command::{ButtonEvent, Button};

use super::constant;
// use super::discover;
use super::error::{Error, Result};

use reqwest::Client;
use serde_json::Value;

use std::time::Duration;

/// A Vizio Device
#[derive(Debug, Clone)]
pub struct Device {
    friendly_name: String,
    manufacturer: String,
    model_name: String,
    ip_addr: String,
    port: u16,
    uuid: String,
    auth_token: Option<String>,
    client: Client,
}

impl Device {
    pub(crate) fn new(
        friendly_name: String,
        manufacturer: String,
        model_name: String,
        ip_addr: String,
        uuid: String
    ) -> Self {
        let client =
            reqwest::Client::builder()
            .timeout(Duration::from_secs(constant::DEFAULT_TIMEOUT))
            .https_only(true)
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Unable to build Reqwest Client");
        Self {
            friendly_name,
            manufacturer,
            model_name,
            ip_addr,
            port: 7345,
            uuid,
            auth_token: None,
            client,
        }
    }

    /// Get device's model name
    pub fn model_name(&self) -> String {
        self.model_name.clone()
    }

    /// Get device's 'friendly' name
    pub fn friendly_name(&self) -> String {
        self.friendly_name.clone()
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

    /// If already paired, set client's auth token for the device.
    /// Returns an error if connection fails.
    pub fn set_auth_token(&mut self, token: String) -> Result<()> {
        self.auth_token = Some(token);
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

        println!("{}", res);

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
    /// let client_name = "My App Name";
    /// let client_id = "myapp-rs";
    ///
    /// let (pairing_token, challenge) = dev.begin_pair(client_name, client_id).await?;
    ///
    /// let mut pin = String::new();
    /// stdin().read_line(&mut pin).unwrap();
    ///
    /// let auth_token = dev.finish_pair(client_id, pairing_token, challenge, pin).await?;
    /// println!("{}", auth_token);
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
    /// // Power on device
    /// if !dev.is_powered_on().await? {
    ///     dev.button_event(ButtonEvent::KeyPress(Button::PowerOn)).await?
    /// }
    ///
    /// // Increase Volume
    /// dev.button_event(ButtonEvent::KeyPress(Button::VolumeUp)).await?;
    ///
    /// // Increase Volume More
    /// dev.button_event(vec![
    ///     ButtonEvent::KeyPress(Button::VolumeUp),
    ///     ButtonEvent::KeyPress(Button::VolumeUp),
    /// ]).await?
    /// ```
    pub async fn button_event<V: Into<Vec<ButtonEvent>>>(&self, buttons: V) -> Result<()> {
        let button_vec: Vec<ButtonEvent> = buttons.into();
        self.send_command(Command::RemoteButtonPress(button_vec)).await?;
        Ok(())
    }

    /// Get the current device input
    pub async fn current_input(&self) -> Result<()> {
        // TO-DO
        let res = self.send_command(Command::GetCurrentInput).await?.unwrap();
        println!("{}\n", res.to_string());
        Ok(())
    }

    /// Get list of available inputs
    pub async fn list_inputs(&self) -> Result<()> {
        // TO-DO
        let res = self.send_command(Command::GetInputList).await?.unwrap();
        println!("{:#?}\n", res);
        Ok(())
    }

    async fn send_command(&self, command: Command) -> Result<Option<Value>> {

        // Start building request
        let mut req = match command.request_type() {
            RequestType::Get => {
                self.client.get(
                    format!("https://{}:{}{}", self.ip_addr, self.port, command.endpoint())
                )
            },
            RequestType::Put => {
                println!("Body:{}\n", serde_json::to_string(&command).unwrap());
                self.client.put(
                    format!("https://{}:{}{}", self.ip_addr, self.port, command.endpoint())
                )
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
                req = req.header("Auth", token);
            },
            None => {},
        }

        // Send command
        let res = req.send().await?;

        // Process response
        response::process(res.text().await?)
    }
}
