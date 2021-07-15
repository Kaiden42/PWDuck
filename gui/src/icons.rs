//! Icon definitions used in the gui.

use iced::Font;

/// The font containing the icons generated from the Bootstrap Icons.
///
/// See: <https://icons.getbootstrap.com>
pub const ICON_FONT: Font = Font::External {
    name: "PWDuck Icons",
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
    /// caret-down
    CaretDown,
    /// caret-right
    CaretRight,
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
    /// globe2
    Globe2,
    /// key
    Key,
    /// keyboard
    Keyboard,
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
    /// trash
    Trash,
    /// unlock
    Unlock,
    /// x-square
    XSquare,
}

/// Map the [`Icon`](Icon) to the associated character.
pub const fn icon_to_char(icon: Icon) -> char {
    match icon {
        Icon::ArrowClockwise => '\u{0FA0}',
        Icon::ArrowLeft => '\u{0FAA}',
        Icon::Backspace => '\u{0FB4}',
        Icon::CaretDown => '\u{0FBE}',
        Icon::CaretRight => '\u{0FC8}',
        Icon::DashSquare => '\u{0FD2}',
        Icon::Dice3 => '\u{0FDC}',
        Icon::EyeSlash => '\u{0FE6}',
        Icon::Eye => '\u{0FF0}',
        Icon::FileEarmarkLock => '\u{0FFA}',
        Icon::FileEarmarkPerson => '\u{1004}',
        Icon::FolderPlus => '\u{100E}',
        Icon::Folder => '\u{1018}',
        Icon::Gear => '\u{1022}',
        Icon::Globe2 => '\u{102C}',
        Icon::Key => '\u{1036}',
        Icon::Keyboard => '\u{1040}',
        Icon::List => '\u{104A}',
        Icon::Lock => '\u{1054}',
        Icon::Pencil => '\u{105E}',
        Icon::PersonPlus => '\u{1068}',
        Icon::Person => '\u{1072}',
        Icon::PlusSquare => '\u{107C}',
        Icon::Safe => '\u{1086}',
        Icon::Save => '\u{1090}',
        Icon::ThreeDotsVertical => '\u{109A}',
        Icon::Trash => '\u{10A4}',
        Icon::Unlock => '\u{10AE}',
        Icon::XSquare => '\u{10B8}',
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
