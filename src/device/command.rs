use super::{SubSetting, UrlBase};

use serde::ser::{Serialize, Serializer, SerializeStruct};


// #[allow(unused)]
pub enum Command {
    StartPairing{client_name: String, client_id: String},
    FinishPairing{client_id: String, pairing_token: u32, challenge: u32, response_value: String},
    CancelPairing{client_id: String, pairing_token: u32, challenge: u32},

    GetPowerState,
    RemoteButtonPress(Vec<ButtonEvent>),

    GetCurrentInput,
    GetInputList,
    ChangeInput{name: String, hashval: u32},
    LaunchApp,
    ReadSettings(SubSetting),
    ReadStaticSettings(SubSetting),
    // WriteSettings, // To-do (Brick warning)
}

pub enum RequestType {
    Get,
    Put,
}

impl Command {
    /// Get the endpoint of the command
    pub fn endpoint(&self) -> String {
        match self {
            Self::StartPairing{..}                  => "/pairing/start".into(),
            Self::FinishPairing{..}                 => "/pairing/pair".into(),
            Self::CancelPairing{..}                 => "/pairing/cancel".into(),
            Self::GetPowerState                     => "/state/device/power_mode".into(),
            Self::RemoteButtonPress{..}             => "/key_command/".into(),
            Self::GetCurrentInput                   => "/menu_native/dynamic/tv_settings/devices/current_input".into(),
            Self::GetInputList                      => "/menu_native/dynamic/tv_settings/devices/name_input".into(),
            Self::ChangeInput{..}                   => "/menu_native/dynamic/tv_settings/devices/current_input".into(),
            Self::LaunchApp                         => "/app/launch".into(),
            Self::ReadSettings(subsetting)          => subsetting.endpoint(UrlBase::Dynamic),
            Self::ReadStaticSettings(subsetting)    => subsetting.endpoint(UrlBase::Static),
            // Self::WriteSettings             => "/menu_native/dynamic/tv_settings/SETTINGS_CNAME/ITEMS_CNAME",
        }
    }

    /// Get the request type of the command
    pub fn request_type(&self) -> RequestType {
        match self {
            Self::StartPairing{..}
            | Self::FinishPairing{..}
            | Self::CancelPairing{..}
            | Self::RemoteButtonPress{..}
            | Self::ChangeInput{..}
            | Self::LaunchApp       => RequestType::Put,
            // Self::WriteSettings     => RequestType::Put,
            Self::GetPowerState
            | Self::GetCurrentInput
            | Self::GetInputList
            | Self::ReadSettings(_)
            | Self::ReadStaticSettings(_) => RequestType::Get,
        }
    }
}

impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match self {
            Self::StartPairing{client_name, client_id} => {
                let mut command = serializer.serialize_struct("", 2)?;
                command.serialize_field("DEVICE_NAME",  client_name)?;
                command.serialize_field("DEVICE_ID",    client_id)?;
                command.end()
            },
            Self::CancelPairing{client_id, pairing_token, challenge} => {
                let mut command = serializer.serialize_struct("", 4)?;
                command.serialize_field("DEVICE_ID",            client_id)?;
                command.serialize_field("CHALLENGE_TYPE",       challenge)?;
                command.serialize_field("RESPONSE_VALUE",       "1111")?;
                command.serialize_field("PAIRING_REQ_TOKEN",    pairing_token)?;
                command.end()
            },
            Self::FinishPairing{client_id, pairing_token, challenge, response_value} => {
                let mut command = serializer.serialize_struct("", 4)?;
                command.serialize_field("DEVICE_ID",            client_id)?;
                command.serialize_field("CHALLENGE_TYPE",       challenge)?;
                command.serialize_field("RESPONSE_VALUE",       response_value)?;
                command.serialize_field("PAIRING_REQ_TOKEN",    pairing_token)?;
                command.end()
            },
            Self::RemoteButtonPress(button_event_vec) => {
                let mut command = serializer.serialize_struct("", 1)?;
                command.serialize_field("KEYLIST", button_event_vec)?;
                command.end()
            },
            Self::ChangeInput{name, hashval} => {
                let mut command = serializer.serialize_struct("", 3)?;
                command.serialize_field("REQUEST",  "MODIFY")?;
                command.serialize_field("VALUE",    name)?;
                command.serialize_field("HASHVAL",  hashval)?;
                command.end()
            },
            // TO-DO:
            // Self::LaunchApp => {
            //     let mut command = serializer.serialize_struct("", )?;
            //     command.serialize_field("", )?;
            //     command.serialize_field("", )?;
            //     command.end()
            // },
            // Self::WriteSettings => {
            //     let mut command = serializer.serialize_struct("", )?;
            //     command.serialize_field("", )?;
            //     command.serialize_field("", )?;
            //     command.end()
            // },
            _ => serializer.serialize_struct("", 0)?.end()
        }
    }
}

/// Button interactions used in [`button_event()`](./struct.Device.html/#method.button_event)
///
/// Must include a [`Button`] to specify what you want to interact with
pub enum ButtonEvent {
    /// Hold the button down
    KeyDown(Button),
    /// Release the button after a hold
    KeyUp(Button),
    /// Click the button once
    KeyPress(Button),
}

impl ButtonEvent {
    fn button(&self) -> Button {
        match self {
            Self::KeyDown(button)
            | Self::KeyUp(button)
            | Self::KeyPress(button) => *button
        }
    }
}

impl ToString for ButtonEvent {
    fn to_string(&self) -> String {
        match self {
            Self::KeyDown(_)    => "KEYDOWN",
            Self::KeyUp(_)      => "KEYUP",
            Self::KeyPress(_)   => "KEYPRESS",
        }.to_string()
    }
}

impl From<ButtonEvent> for Vec<ButtonEvent> {
    fn from(event: ButtonEvent) -> Vec<ButtonEvent> {
        vec![event]
    }
}

impl Serialize for ButtonEvent{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut key_action = serializer.serialize_struct("", 3)?;
        let button = self.button();
        key_action.serialize_field("CODESET", &button.code_set())?;
        key_action.serialize_field("CODE",    &button.code())?;
        key_action.serialize_field("ACTION",  &self.to_string())?;
        key_action.end()
    }
}

/// "Buttons" you can interact with using [`ButtonEvent`]
#[allow(unused)]
#[derive(Clone, Copy)]
pub enum Button {
    /// Seek Forward
    SeekFwd,
    /// Seek Back
    SeekBack,
    /// Pause
    Pause,
    /// Play
    Play,
    /// Directional pad down
    Down,
    /// Directional pad left
    Left,
    /// Directional pad up
    Up,
    /// Directional pad right
    Right,
    /// Ok button
    Ok,
    /// Back
    Back,
    /// Smartcast button
    SmartCast,
    /// Toggle Closed Captioning
    CCToggle,
    /// Info
    Info,
    /// Menu
    Menu,
    /// Home
    Home,
    /// Volume down
    VolumeDown,
    /// Volume up
    VolumeUp,
    /// Disable mute
    MuteOff,
    /// Enable mute
    MuteOn,
    /// Toggle mute
    MuteToggle,
    /// Picture mode
    PicMode,
    /// Picture size
    PicSize,
    /// Next input
    InputNext,
    /// Channel down
    ChannelDown,
    /// Channel up
    ChannelUp,
    /// Previous channel
    ChannelPrev,
    /// Exit
    Exit,
    /// Power off
    PowerOff,
    /// Power on
    PowerOn,
    /// Toggle power
    PowerToggle,
}

impl Button {

    fn code_set(&self) -> u8 {
        match self {
            Self::SeekFwd
            | Self::SeekBack
            | Self::Pause
            | Self::Play        => 2,

            Self::Down
            | Self::Left
            | Self::Up
            | Self::Right
            | Self::Ok          => 3,

            Self::Back
            | Self::SmartCast
            | Self::CCToggle
            | Self::Info
            | Self::Menu
            | Self::Home        => 4,

            Self::VolumeDown
            | Self::VolumeUp
            | Self::MuteOff
            | Self::MuteOn
            | Self::MuteToggle   => 5,

            Self::PicMode
            | Self::PicSize     => 6,

            Self::InputNext     => 7,

            Self::ChannelDown
            | Self::ChannelUp
            | Self::ChannelPrev  => 8,

            Self::Exit          => 9,

            Self::PowerOff
            | Self::PowerOn
            | Self::PowerToggle  => 11,
        }
    }

    fn code(&self) -> u8 {
        match self {
            Self::SeekFwd => 0,
            Self::SeekBack => 1,
            Self::Pause => 2,
            Self::Play => 3,

            Self::Down => 0, //
            Self::Left => 1, //
            // TO-DO: maybe figure out how this has changed
            Self::Up => 8, // or 3
            Self::Right => 7, // or 5
            Self::Ok => 2, //

            Self::Back => 0, //
            Self::SmartCast => 3,//
            Self::CCToggle => 4,
            Self::Info => 6, //
            Self::Menu => 8, //
            Self::Home => 15, //

            Self::VolumeDown => 0, //
            Self::VolumeUp => 1, //
            Self::MuteOff => 2, //
            Self::MuteOn => 3, //
            Self::MuteToggle => 4, //

            Self::PicMode => 0, //
            Self::PicSize => 2, //

            Self::InputNext => 1, //

            Self::ChannelDown => 0,
            Self::ChannelUp => 1,
            Self::ChannelPrev => 2,

            Self::Exit => 0, //

            Self::PowerOff => 0, //
            Self::PowerOn => 1, //
            Self::PowerToggle => 2, //
        }
    }
}
