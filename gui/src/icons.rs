//! Icon definitions used in the gui.

use iced::Font;

/// The font containing the icons generated from the Bootstrap Icons.
///
/// See: <https://icons.getbootstrap.com>
pub const ICON_FONT: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../font/pwduck-icons.ttf"),
};

#[derive(Clone, Copy, Debug)]
/// An enumeration of all available icons in the [`ICON_FONT`](ICON_FONT).
///
/// See: <https://icons.getbootstrap.com>
pub enum Icon {
    /// arrow-clockwise
    ArrowClockwise,
    /// arrow-left
    ArrowLeft,
    /// backspace
    Backspace,
    /// dash-square
    DashSquare,
    /// dice-3
    Dice3,
    /// eye-slash
    EyeSlash,
    /// eye
    Eye,
    /// file-earmark-lock
    FileEarmarkLock,
    /// file-earmark-person
    FileEarmarkPerson,
    /// folder-plus
    FolderPlus,
    /// folder
    Folder,
    /// gear
    Gear,
    /// key
    Key,
    /// list
    List,
    /// lock
    Lock,
    /// pencil
    Pencil,
    /// person-plus
    PersonPlus,
    /// person
    Person,
    /// plus-square
    PlusSquare,
    /// safe
    Safe,
    /// save
    Save,
    /// three-dots-vertical
    ThreeDotsVertical,
    /// unlock
    Unlock,
    /// x-square
    XSquare,
}

/// Map the [`Icon`](Icon) to the associated character.
pub const fn icon_to_char(icon: Icon) -> char {
    match icon {
        Icon::ArrowClockwise => '\u{FA0}',
        Icon::ArrowLeft => '\u{FAA}',
        Icon::Backspace => '\u{FB4}',
        Icon::DashSquare => '\u{FBE}',
        Icon::Dice3 => '\u{FC8}',
        Icon::EyeSlash => '\u{FD2}',
        Icon::Eye => '\u{FDC}',
        Icon::FileEarmarkLock => '\u{FE6}',
        Icon::FileEarmarkPerson => '\u{FF0}',
        Icon::FolderPlus => '\u{FFA}',
        Icon::Folder => '\u{1004}',
        Icon::Gear => '\u{100E}',
        Icon::Key => '\u{1018}',
        Icon::List => '\u{1022}',
        Icon::Lock => '\u{102C}',
        Icon::Pencil => '\u{1036}',
        Icon::PersonPlus => '\u{1040}',
        Icon::Person => '\u{104A}',
        Icon::PlusSquare => '\u{1054}',
        Icon::Safe => '\u{105E}',
        Icon::Save => '\u{1068}',
        Icon::ThreeDotsVertical => '\u{1072}',
        Icon::Unlock => '\u{107C}',
        Icon::XSquare => '\u{1086}',
    }
}

impl From<Icon> for char {
    fn from(icon: Icon) -> Self {
        icon_to_char(icon)
    }
}

impl From<Icon> for String {
    fn from(icon: Icon) -> Self {
        icon_to_char(icon).into()
    }
}

impl std::fmt::Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", icon_to_char(*self))
    }
}
