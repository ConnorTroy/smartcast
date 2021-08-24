use std::fmt::Debug;

/// Button interactions used in `key(up|down|press)()` in [super::Device]
///
/// Must include a [`Button`] to specify what you want to interact with
#[derive(Debug, Clone, Copy)]
pub(super) enum KeyEvent {
    /// Hold the button down
    Down,
    /// Release the button after a hold
    Up,
    /// Click the button once
    Press,
}

impl ToString for KeyEvent {
    fn to_string(&self) -> String {
        match self {
            Self::Down => "KEYDOWN",
            Self::Up => "KEYUP",
            Self::Press => "KEYPRESS",
        }
        .to_string()
    }
}

impl From<KeyEvent> for Vec<KeyEvent> {
    fn from(event: KeyEvent) -> Vec<KeyEvent> {
        vec![event]
    }
}

/// Remote control "buttons" you can interact with using [`Device::key_press()`](super::Device::key_press),
/// [`Device::key_down()`](super::Device::key_down), or [`Device::key_up()`](super::Device::key_up)
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
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
    #[doc(hidden)]
    LeftAlt,
    /// Directional pad up
    Up,
    #[doc(hidden)]
    UpAlt,
    /// Directional pad right
    Right,
    #[doc(hidden)]
    RightAlt,
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
    pub(super) fn codeset(&self) -> u8 {
        match self {
            Self::SeekFwd | Self::SeekBack | Self::Pause | Self::Play => 2,

            Self::Down
            | Self::Left
            | Self::LeftAlt
            | Self::Up
            | Self::UpAlt
            | Self::Right
            | Self::RightAlt
            | Self::Ok => 3,

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

    pub(super) fn code(&self) -> u8 {
        match self {
            // Code set 2
            Self::SeekFwd => 0,
            Self::SeekBack => 1,
            Self::Pause => 2,
            Self::Play => 3,

            // Code set 3
            Self::Down => 0,
            Self::Left => 1,
            Self::LeftAlt => 4,
            Self::Up => 8,
            Self::UpAlt => 3,
            Self::Right => 7,
            Self::RightAlt => 5,
            Self::Ok => 2,

            // Code set 4
            Self::Back => 0,
            Self::SmartCast => 3,
            Self::CCToggle => 4,
            Self::Info => 6,
            Self::Menu => 8,
            Self::Home => 15,

            // Code set 5
            Self::VolumeDown => 0,
            Self::VolumeUp => 1,
            Self::MuteOff => 2,
            Self::MuteOn => 3,
            Self::MuteToggle => 4,

            // Code set 6
            Self::PicMode => 0,
            Self::PicSize => 2,

            // Code set 7
            Self::InputNext => 1,

            // Code set 8
            Self::ChannelDown => 0,
            Self::ChannelUp => 1,
            Self::ChannelPrev => 2,

            // Code set 9
            Self::Exit => 0,

            // Code set 11
            Self::PowerOff => 0,
            Self::PowerOn => 1,
            Self::PowerToggle => 2,
        }
    }

    pub(super) fn alt(&self) -> Option<Self> {
        match self {
            Self::Left => Some(Self::LeftAlt),
            Self::Up => Some(Self::UpAlt),
            Self::Right => Some(Self::RightAlt),
            _ => None,
        }
    }
}
