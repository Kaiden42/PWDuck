//! TODO

use iced::Font;

/// TODO
pub const ICON_FONT: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../font/pwduck-icons.ttf"),
};

#[derive(Clone, Copy, Debug)]
/// TODO
pub enum Icon {
    /// TODO
    ArrowClockwise,
    /// TODO
    ArrowLeft,
    /// TODO
    Backspace,
    /// TODO
    DashSquare,
    /// TODO
    Dice3,
    /// TODO
    EyeSlash,
    /// TODO
    Eye,
    /// TODO
    FileEarmarkLock,
    /// TODO
    FileEarmarkPerson,
    /// TODO
    FolderPlus,
    /// TODO
    Folder,
    /// TODO
    Gear,
    /// TODO
    Key,
    /// TODO
    List,
    /// TODO
    Lock,
    /// TODO
    Pencil,
    /// TODO
    PersonPlus,
    /// TODO
    Person,
    /// TODO
    PlusSquare,
    /// TODO
    Safe,
    /// TODO
    Save,
    /// TODO
    ThreeDotsVertical,
    /// TODO
    Unlock,
    /// TODO
    XSquare,
}

/// TODO
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
