use serde::ser::{Serialize, SerializeStruct, Serializer};

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
            Self::KeyDown(button) | Self::KeyUp(button) | Self::KeyPress(button) => *button,
        }
    }
}

impl ToString for ButtonEvent {
    fn to_string(&self) -> String {
        match self {
            Self::KeyDown(_) => "KEYDOWN",
            Self::KeyUp(_) => "KEYUP",
            Self::KeyPress(_) => "KEYPRESS",
        }
        .to_string()
    }
}

impl From<ButtonEvent> for Vec<ButtonEvent> {
    fn from(event: ButtonEvent) -> Vec<ButtonEvent> {
        vec![event]
    }
}

impl Serialize for ButtonEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut key_action = serializer.serialize_struct("", 3)?;
        let button = self.button();
        key_action.serialize_field("CODESET", &button.code_set())?;
        key_action.serialize_field("CODE", &button.code())?;
        key_action.serialize_field("ACTION", &self.to_string())?;
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
            Self::SeekFwd | Self::SeekBack | Self::Pause | Self::Play => 2,

            Self::Down | Self::Left | Self::Up | Self::Right | Self::Ok => 3,

            Self::Back
            | Self::SmartCast
            | Self::CCToggle
            | Self::Info
            | Self::Menu
            | Self::Home => 4,

            Self::VolumeDown | Self::VolumeUp | Self::MuteOff | Self::MuteOn | Self::MuteToggle => {
                5
            }

            Self::PicMode | Self::PicSize => 6,

            Self::InputNext => 7,

            Self::ChannelDown | Self::ChannelUp | Self::ChannelPrev => 8,

            Self::Exit => 9,

            Self::PowerOff | Self::PowerOn | Self::PowerToggle => 11,
        }
    }

    fn code(&self) -> u8 {
        match self {
            Self::SeekFwd => 0,
            Self::SeekBack => 1,
            Self::Pause => 2,
            Self::Play => 3,

            Self::Down => 0,
            Self::Left => 1,
            // TODO: maybe figure out how this has changed
            Self::Up => 8,    // or 3
            Self::Right => 7, // or 5
            Self::Ok => 2,

            Self::Back => 0,
            Self::SmartCast => 3,
            Self::CCToggle => 4,
            Self::Info => 6,
            Self::Menu => 8,
            Self::Home => 15,

            Self::VolumeDown => 0,
            Self::VolumeUp => 1,
            Self::MuteOff => 2,
            Self::MuteOn => 3,
            Self::MuteToggle => 4,

            Self::PicMode => 0,
            Self::PicSize => 2,

            Self::InputNext => 1,

            Self::ChannelDown => 0,
            Self::ChannelUp => 1,
            Self::ChannelPrev => 2,

            Self::Exit => 0,

            Self::PowerOff => 0,
            Self::PowerOn => 1,
            Self::PowerToggle => 2,
        }
    }
}
