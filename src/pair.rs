use super::{Error, Result, Device};

/// Pairing
#[derive(Debug, Clone)]
pub(crate) struct Pairing {
    pairing_req_token: u32,
    challenge_type: u32,
}

impl Device {

    /// Begin the pairing process.
    ///
    /// Upon calling this method, the device will enter pairing mode.
    /// If the device is a tv, it will display a pin to be entered using [`finish_pair()`](./struct.Device.html/#method.finish_pair).
    pub async fn begin_pair(&mut self, client_id: String, client_name: String) -> Result<()> {

        if self.pairing.is_none() {

        }

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

        if self.pairing.is_some() {

        }
        Ok(())
    }

    /// Cancel the pairing process.
    ///
    /// Upon calling this method, the pairing process will be canceled and the
    /// device will leave pairing mode.
    pub async fn cancel_pair(&mut self) -> Result<()> {

        if self.pairing.is_some() {

        }
        Ok(())
    }
}
