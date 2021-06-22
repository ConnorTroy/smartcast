mod command;

use super::constant;
use super::discover;
use super::error::{Error, Result};

use reqwest::Client;

use std::error;
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
    client: Option<Client>,
}

impl Device {
    pub(crate) fn new(
        friendly_name: String,
        manufacturer: String,
        model_name: String,
        ip_addr: String,
        uuid: String
    ) -> Self {
        Self {
            friendly_name,
            manufacturer,
            model_name,
            ip_addr,
            port: 7345,
            uuid,
            auth_token: None,
            client: None,
        }
    }

    pub(crate) fn build_client(&mut self) -> Result<()> {
        if self.client.is_none() {
            self.client = Some(
                reqwest::Client::builder()
                .timeout(Duration::from_secs(constant::DEFAULT_TIMEOUT))
                .https_only(true)
                .build()?
            );
        }
        Ok(())
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

    /// Get client's auth token for device
    pub fn auth_token(&self) -> Option<String> {
        self.auth_token.clone()
    }

    /// Begin the pairing process.
    ///
    /// Upon calling this method, the device will enter pairing mode.
    /// If the device is a tv, it will display a pin to be entered using [`finish_pair()`](./struct.Device.html/#method.finish_pair).
    pub async fn begin_pair(&mut self, client_id: String, client_name: String) -> Result<()> {
        // TO-DO

        Ok(())
    }

    /// Finish the pairing process.
    ///
    /// Upon calling this method with the pin displayed by the device, the
    /// pairing process will end and the client will be paired.
    ///
    /// # Example
    ///
    /// ```
    /// let client_id = "myapp-rs".to_string();
    /// let client_name = "My App Name".to_string();
    ///
    /// device.begin_pair(client_id, client_name).await?;
    ///
    /// let mut pin = String::new();
    /// stdin().read_line(&mut pin)?;
    ///
    /// device.finish_pair(pin).await?;
    /// ```
    pub async fn finish_pair(&mut self, pin: String) -> Result<()> {
        // TO-DO

        Ok(())
    }

    /// Cancel the pairing process.
    ///
    /// Upon calling this method, the pairing process will be canceled and the
    /// device will leave pairing mode.
    pub async fn cancel_pair(&mut self) -> Result<()> {
        // TO-DO
        
        Ok(())
    }

    // /// Set client's auth token for device (if already paired). Returns an error
    // /// if connection fails.
    // pub fn set_auth_token(&mut self, token: String) -> Result<()> {
    //     self.auth_token = Some(token);
    //     // TO-DO: Verify token
    //     Ok(())
    // }
}
