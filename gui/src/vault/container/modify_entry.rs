//! The view of the entry creator / modifier.
use std::sync::MutexGuard;

use getset::{CopyGetters, Getters, MutGetters, Setters};
use iced::{
    button, scrollable, text_input, Button, Column, Command, Element, Length, Row, Scrollable,
    Space, Text,
};
use iced_aw::{modal, Card};
use iced_focus::Focus;
use pwduck_core::{EntryBody, EntryHead, MemKey, PWDuckCoreError, PasswordInfo, Uuid, Vault};

use crate::{
    error::PWDuckGuiError,
    icons::{Icon, ICON_FONT},
    password_score::PasswordScore,
    theme::Theme,
    utils::{
        centered_container_with_column, default_text_input, default_vertical_space,
        estimate_password_strength, icon_button, password_toggle, ButtonData, ButtonKind, SomeIf,
    },
    Platform, DEFAULT_COLUMN_PADDING, DEFAULT_COLUMN_SPACING, DEFAULT_MAX_WIDTH,
    DEFAULT_ROW_SPACING,
};

#[cfg(test)]
use mocktopus::macros::*;

/// The state of the modify entry view.
#[derive(CopyGetters, Getters, MutGetters, Setters, Focus)]
pub struct ModifyEntryView {
    /// The entry was newly created or an existing entry will be modified.
    state: State,

    /// The decrypted head of the entry to modify.
    #[getset(get = "pub", get_mut = "pub")]
    entry_head: EntryHead,
    /// The decrypted body of the entry to modify.
    #[getset(get = "pub", get_mut = "pub")]
    entry_body: EntryBody,

    /// The state of the [`TextInput`](iced::TextInput) of the title.
    #[focus(enable)]
    title_state: text_input::State,
    /// The state of the [`TextInput`](iced::TextInput) of the usermane.
    #[focus(enable)]
    username_state: text_input::State,
    /// The state of the [`Button`](iced::Button) to copy the username.
    username_copy_state: button::State,
    /// The state of the [`TextInput`](iced::TextInput) of the password.
    #[focus(enable)]
    password_state: text_input::State,
    /// The visibility of the password.
    #[getset(get = "pub", set = "pub")]
    password_show: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility of the password.
    password_show_state: button::State,
    /// The state of the [`Button`](iced::Button) to open the password generator.
    password_generate_state: button::State,
    /// The state of the [`Button`](iced::Button) to copy the password.
    password_copy_state: button::State,

    /// The state of the [`TextInput`](iced::TextInput) of the web address.
    #[focus(enable)]
    web_address_state: text_input::State,
    /// The state of the [`Button`](iced::Button) to open the web address in a browser.
    open_in_browser_state: button::State,
    /// The state of the [`TextInput`](iced::TextInput) of the email.
    #[focus(enable)]
    email_state: text_input::State,

    /// The estimated password score.
    password_score: Option<PasswordScore>,

    /// Whether the entry was modified or not.
    is_modified: bool,

    /// The state of the cancel [`Button`](iced::Button).
    cancel_state: button::State,
    /// The state of the submit [`Button`](iced::Button).
    submit_state: button::State,

    /// If the advanced area is shown.
    #[getset(get_copy)]
    show_advanced: bool,
    /// The state of the [`Button`](iced::Button) to toggle the visibility of the advanced area.
    advanced_button_state: button::State,
    /// The state of the advanced area.
    #[focus(enable = "self.show_advanced")]
    advanced_state: AdvancedState,

    /// The state of the [`Scrollable`](iced::Scrollable).
    scroll_state: scrollable::State,
}

/// The message that is send by the `ModifyEntryView`.
#[derive(Clone, Debug)]
pub enum ModifyEntryMessage {
    /// Change the title to the new value.
    TitleInput(String),
    /// Change the username to the new value.
    UsernameInput(String),
    /// Copy the username.
    UsernameCopy,
    /// Change the password to the new value.
    PasswordInput(String),
    /// Toggle the visibility of the password.
    PasswordShow,
    /// Open the password generator.
    PasswordGenerate,
    /// Copy the password.
    PasswordCopy,

    /// Change the web address to the new value.
    WebAddressInput(String),
    /// Open the web address in a browser.
    OpenInBrowser,
    /// The result of the browser opener.
    Opener(Result<(), PWDuckGuiError>),
    /// Change the email to the new value.
    EmailInput(String),

    /// Set the password score tho the new value.
    PasswordScore(Result<PasswordInfo, PWDuckCoreError>),

    /// Cancel the modification of the entry.
    Cancel,
    /// Submit the modification of the entry.
    Submit,

    /// Toggle the visibility of the advanced area.
    ToggleAdvanced,
    /// The messages produced by the advanced area.
    Advanced(AdvancedStateMessage),

    /// The messages produced by the modal.
    Modal(ModifyEntryModalMessage),
}
impl SomeIf for ModifyEntryMessage {}

#[cfg_attr(test, mockable)]
impl ModifyEntryView {
    /// Create a new [`ModifyEntryView`](ModifyEntryView).
    ///
    /// It expects:
    ///  - A new entry was created or an existing will be modified.
    ///  - The head of the entry to modify.
    ///  - The body of teh entry to modify.
    pub fn with(state: State, entry_head: EntryHead, entry_body: EntryBody) -> Self {
        Self {
            state,

            entry_head,
            entry_body,

            title_state: if state == State::Create {
                text_input::State::focused()
            } else {
                text_input::State::new()
            },
            username_state: text_input::State::new(),
            username_copy_state: button::State::new(),
            password_state: text_input::State::new(),
            password_show: false,
            password_show_state: button::State::new(),
            password_generate_state: button::State::new(),
            password_copy_state: button::State::new(),

            web_address_state: text_input::State::new(),
            open_in_browser_state: button::State::new(),
            email_state: text_input::State::new(),

            password_score: Option::None,

            is_modified: false,

            cancel_state: button::State::new(),
            submit_state: button::State::new(),

            show_advanced: false,
            advanced_button_state: button::State::new(),
            advanced_state: AdvancedState::new(),

            scroll_state: scrollable::State::new(),
        }
    }

    /// True, if the container contains unsaved changes.
    #[allow(clippy::missing_const_for_fn)]
    pub fn contains_unsaved_changes(&self) -> bool {
        self.entry_head.is_modified() || self.entry_body.is_modified()
    }

    /// Update the title and replace it with the given value.
    fn update_title(&mut self, title: String) -> Command<ModifyEntryMessage> {
        let _ = self.entry_head_mut().set_title(title);
        self.is_modified = true;
        Command::none()
    }

    /// Update the username and replace it with the given value.
    fn update_username(&mut self, username: String) -> Command<ModifyEntryMessage> {
        let _ = self.entry_body_mut().set_username(username);
        self.is_modified = true;
        Command::none()
    }

    /// Copy the username to clipboard.
    fn copy_username(&self, clipboard: &mut iced::Clipboard) -> Command<ModifyEntryMessage> {
        clipboard.write(self.entry_body().username().to_string());
        Command::none()
    }

    /// Update the password and replace it with the given value.
    fn update_password(&mut self, password: String) -> Command<ModifyEntryMessage> {
        let _ = self.entry_body_mut().set_password(password);
        self.is_modified = true;
        self.estimate_password_strength()
    }

    /// Update the web address and replace it with the given value.
    fn update_web_address(&mut self, web_address: String) -> Command<ModifyEntryMessage> {
        let _ = self.entry_head_mut().set_web_address(web_address);
        self.is_modified = true;
        Command::none()
    }

    /// Open the web address of the entry in the browser.
    fn open_in_browser<P: Platform + 'static>(&self) -> Command<ModifyEntryMessage> {
        Command::perform(
            P::open_in_browser(self.entry_head.web_address().clone()),
            ModifyEntryMessage::Opener,
        )
    }

    /// Update the email and replace it with the given value.
    fn update_email(&mut self, email: String) -> Command<ModifyEntryMessage> {
        let _ = self.entry_body_mut().set_email(email);
        self.is_modified = true;
        Command::none()
    }

    /// Toggle the visibility of the password.
    fn toggle_password_visibility(&mut self) -> Command<ModifyEntryMessage> {
        self.password_show = !self.password_show;
        Command::none()
    }

    /// Copy the password to the clipboard.
    fn copy_password(&self, clipboard: &mut iced::Clipboard) -> Command<ModifyEntryMessage> {
        clipboard.write(self.entry_body().password().to_string());
        Command::none()
    }

    /// Toggle the visibility of the advanced area.
    fn toggle_advanced_visibility(&mut self) -> Command<ModifyEntryMessage> {
        self.show_advanced = !self.show_advanced;
        Command::none()
    }

    /// Estimate the strength of the password.
    fn estimate_password_strength(&self) -> Command<ModifyEntryMessage> {
        Command::perform(
            estimate_password_strength(self.entry_body.password().clone()),
            ModifyEntryMessage::PasswordScore,
        )
    }

    /// Set the estimated password score.
    fn set_password_score(
        &mut self,
        password_info: Result<PasswordInfo, PWDuckCoreError>,
    ) -> Command<ModifyEntryMessage> {
        self.password_score = Some(PasswordScore::new(password_info));
        Command::none()
    }

    /// Submit the modification of the entry.
    fn submit(
        &self,
        vault: &mut Vault,
        mem_key: &MutexGuard<MemKey>,
    ) -> Result<Command<ModifyEntryMessage>, PWDuckGuiError> {
        // TODO async
        let master_key = vault
            .master_key()
            .as_unprotected(mem_key, vault.salt(), vault.nonce())?;

        vault.insert_entry(
            self.entry_head.clone(),
            self.entry_body.clone(),
            &master_key,
        )?;

        Ok(Command::none())
    }

    /// Request the deletion of the entry.
    #[allow(clippy::unused_self)]
    fn request_entry_deletion(
        &mut self,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyEntryMessage> {
        *modal_state = modal::State::new(crate::ModalState::ModifyEntry(
            ModifyEntryModal::delete_request(),
        ));
        modal_state.show(true);
        Command::none()
    }

    /// Update the auto type sequence of the entry to the given sequence.
    fn update_auto_type_sequence(
        &mut self,
        auto_type_sequence: String,
    ) -> Command<ModifyEntryMessage> {
        let _ = self
            .entry_head
            .set_auto_type_sequence(auto_type_sequence.into());
        self.is_modified = true;
        Command::none()
    }

    /// Update the advanced state with the given message.
    fn update_advanced<P: Platform + 'static>(
        &mut self,
        message: AdvancedStateMessage,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyEntryMessage> {
        match message {
            AdvancedStateMessage::DeleteEntryRequest => self.request_entry_deletion(modal_state),
            AdvancedStateMessage::AutoTypeInput(auto_type_sequence) => {
                self.update_auto_type_sequence(auto_type_sequence)
            }
        }
    }

    /// Close the modal
    #[allow(clippy::unused_self)]
    fn close_modal(
        &mut self,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyEntryMessage> {
        *modal_state = modal::State::default();
        Command::none()
    }

    /// Delete the entry from the vault.
    fn delete_entry(&mut self, vault: &mut Vault) -> Command<ModifyEntryMessage> {
        vault.delete_entry(self.entry_head.uuid());
        Command::none()
    }

    /// Update the state of the modal.
    fn update_modal(
        &mut self,
        message: &ModifyEntryModalMessage,
        vault: &mut Vault,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyEntryMessage> {
        match message {
            ModifyEntryModalMessage::Close => self.close_modal(modal_state),
            ModifyEntryModalMessage::SubmitDelete => {
                Command::batch([self.delete_entry(vault), self.close_modal(modal_state)])
            }
        }
    }

    /// Update the state of the [`ModifyEntryView`](ModifyEntryView).
    pub fn update<P: Platform + 'static>(
        &mut self,
        message: ModifyEntryMessage,
        vault: &mut Vault,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
        clipboard: &mut iced::Clipboard,
    ) -> Result<Command<ModifyEntryMessage>, PWDuckGuiError> {
        match message {
            ModifyEntryMessage::TitleInput(title) => Ok(self.update_title(title)),
            ModifyEntryMessage::UsernameInput(username) => Ok(self.update_username(username)),
            ModifyEntryMessage::UsernameCopy => Ok(self.copy_username(clipboard)),
            ModifyEntryMessage::PasswordInput(password) => Ok(self.update_password(password)),
            ModifyEntryMessage::PasswordShow => Ok(self.toggle_password_visibility()),
            ModifyEntryMessage::PasswordCopy => Ok(self.copy_password(clipboard)),
            ModifyEntryMessage::WebAddressInput(web_address) => {
                Ok(self.update_web_address(web_address))
            }
            ModifyEntryMessage::OpenInBrowser => Ok(self.open_in_browser::<P>()),
            ModifyEntryMessage::Opener(result) => {
                result?;
                Ok(Command::none())
            }
            ModifyEntryMessage::EmailInput(email) => Ok(self.update_email(email)),
            ModifyEntryMessage::PasswordScore(password_info) => {
                Ok(self.set_password_score(password_info))
            }
            ModifyEntryMessage::ToggleAdvanced => Ok(self.toggle_advanced_visibility()),
            ModifyEntryMessage::Advanced(message) => {
                Ok(self.update_advanced::<P>(message, modal_state))
            }
            ModifyEntryMessage::Modal(message) => {
                Ok(self.update_modal(&message, vault, modal_state))
            }
            ModifyEntryMessage::Cancel => Ok(Command::none()),
            ModifyEntryMessage::Submit => self.submit(vault, &crate::MEM_KEY.lock()?),
            ModifyEntryMessage::PasswordGenerate => {
                PWDuckGuiError::Unreachable("ModifyEntryMessage".into()).into()
            }
        }
    }

    /// Create the view of the [`ModifyEntryView`](ModifyEntryView).
    #[cfg_attr(coverage, no_coverage)]
    pub fn view<P: Platform + 'static>(
        &mut self,
        _selected_group_uuid: &Uuid,
        theme: &dyn Theme,
    ) -> Element<ModifyEntryMessage> {
        let title = title_text_input(&mut self.title_state, self.entry_head.title(), theme);
        let username = username_row(
            &mut self.username_state,
            self.entry_body.username(),
            &mut self.username_copy_state,
            theme,
        );
        let password = password_row(
            &mut self.password_state,
            self.entry_body.password(),
            self.password_show,
            &mut self.password_show_state,
            &mut self.password_generate_state,
            &mut self.password_copy_state,
            &mut self.password_score,
            theme,
        );
        let web_address = web_address_row::<P>(
            &mut self.web_address_state,
            self.entry_head.web_address(),
            &mut self.open_in_browser_state,
            theme,
        );
        let email = email_text_input(&mut self.email_state, self.entry_body.email(), theme);

        let control_row = control_button_row(
            &mut self.cancel_state,
            &mut self.submit_state,
            self.is_modified && !self.entry_head.title().is_empty(),
            theme,
        );

        let advanced = advanced_area::<P>(
            &mut self.advanced_button_state,
            self.show_advanced,
            &mut self.advanced_state,
            self.state,
            &self.entry_head,
            &self.entry_body,
            theme,
        );

        let scrollable = Scrollable::new(&mut self.scroll_state)
            .padding(DEFAULT_COLUMN_PADDING)
            .spacing(DEFAULT_COLUMN_SPACING)
            .push(Text::new(match self.state {
                State::Create => "Create new entry:",
                State::Modify => "Edit entry:",
            }))
            .push(title)
            .push(default_vertical_space())
            .push(username)
            .push(password)
            .push(default_vertical_space())
            .push(web_address)
            .push(email)
            .push(default_vertical_space())
            .push(control_row)
            .push(default_vertical_space())
            .push(advanced);

        centered_container_with_column(vec![scrollable.into()], theme).into()
    }
}

/// Create the field for the title field.
#[cfg_attr(coverage, no_coverage)]
fn title_text_input<'a>(
    state: &'a mut text_input::State,
    title: &'a str,
    theme: &dyn Theme,
) -> Element<'a, ModifyEntryMessage> {
    default_text_input(
        state,
        "Title of this entry",
        title,
        ModifyEntryMessage::TitleInput,
    )
    .style(theme.text_input())
    .into()
}

/// Create the row for the username field.
#[cfg_attr(coverage, no_coverage)]
fn username_row<'a>(
    text_input_state: &'a mut text_input::State,
    username: &'a str,
    button_state: &'a mut button::State,
    theme: &dyn Theme,
) -> Element<'a, ModifyEntryMessage> {
    let username = default_text_input(
        text_input_state,
        "Username",
        username,
        ModifyEntryMessage::UsernameInput,
    )
    .style(theme.text_input());

    let username_copy = icon_button(
        ButtonData {
            state: button_state,
            icon: Icon::FileEarmarkPerson,
            text: "Copy username",
            kind: ButtonKind::Normal,
            on_press: Some(ModifyEntryMessage::UsernameCopy),
        },
        "Copy username to clipboard",
        true,
        theme,
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(username)
        .push(username_copy)
        .into()
}

/// Create the row for the password field.
#[cfg_attr(coverage, no_coverage)]
#[allow(clippy::too_many_arguments)]
fn password_row<'a>(
    text_input_state: &'a mut text_input::State,
    password: &'a str,
    show_password: bool,
    toggle_state: &'a mut button::State,
    generate_state: &'a mut button::State,
    copy_state: &'a mut button::State,
    password_score: &'a mut Option<PasswordScore>,
    theme: &dyn Theme,
) -> Element<'a, ModifyEntryMessage> {
    let mut password = default_text_input(
        text_input_state,
        "Password",
        password,
        ModifyEntryMessage::PasswordInput,
    )
    .style(theme.text_input());
    if !show_password {
        password = password.password();
    }

    let password_show = password_toggle(
        toggle_state,
        show_password,
        ModifyEntryMessage::PasswordShow,
        theme,
    );

    let password_generate = icon_button(
        ButtonData {
            state: generate_state,
            icon: Icon::Dice3,
            text: "Generate password",
            kind: ButtonKind::Normal,
            on_press: Some(ModifyEntryMessage::PasswordGenerate),
        },
        "Generate a new random password",
        true,
        theme,
    );
    let password_copy = icon_button(
        ButtonData {
            state: copy_state,
            icon: Icon::FileEarmarkLock,
            text: "Copy password",
            kind: ButtonKind::Normal,
            on_press: Some(ModifyEntryMessage::PasswordCopy),
        },
        "Copy password to clipboard",
        true,
        theme,
    );

    let row = Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(password)
        .push(password_show)
        .push(password_generate)
        .push(password_copy);

    match password_score.as_mut() {
        Some(password_score) => Column::new()
            .spacing(DEFAULT_COLUMN_SPACING)
            .push(row)
            .push(password_score.view())
            .into(),
        None => row.into(),
    }
}

/// Create the row of the web address field.
#[cfg_attr(coverage, no_coverage)]
fn web_address_row<'a, P: Platform + 'static>(
    text_input_state: &'a mut text_input::State,
    web_address: &'a str,
    button_state: &'a mut button::State,
    theme: &dyn Theme,
) -> Element<'a, ModifyEntryMessage> {
    let web_address = default_text_input(
        text_input_state,
        "Web address",
        web_address,
        ModifyEntryMessage::WebAddressInput,
    )
    .style(theme.text_input());

    let open_in_browser = icon_button(
        ButtonData {
            state: button_state,
            icon: Icon::Globe2,
            text: "Open in browser",
            kind: ButtonKind::Normal,
            on_press: ModifyEntryMessage::OpenInBrowser.some_if(P::is_open_in_browser_available()),
        },
        "Open the web address in a browser",
        true,
        theme,
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(web_address)
        .push(open_in_browser)
        .into()
}

/// Create the text input of the email field.
#[cfg_attr(coverage, no_coverage)]
fn email_text_input<'a>(
    text_input_state: &'a mut text_input::State,
    email: &'a str,
    theme: &dyn Theme,
) -> Element<'a, ModifyEntryMessage> {
    default_text_input(
        text_input_state,
        "E-Mail address",
        email,
        ModifyEntryMessage::EmailInput,
    )
    .style(theme.text_input())
    .into()
}

/// Create the control row containing the cancel and submit buttons.
#[cfg_attr(coverage, no_coverage)]
fn control_button_row<'a>(
    cancel_button_state: &'a mut button::State,
    submit_button_state: &'a mut button::State,
    can_submit: bool,
    theme: &dyn Theme,
) -> Element<'a, ModifyEntryMessage> {
    let cancel = icon_button(
        ButtonData {
            state: cancel_button_state,
            icon: Icon::XSquare,
            text: "Cancel",
            kind: ButtonKind::Normal,
            on_press: Some(ModifyEntryMessage::Cancel),
        },
        "Cancel changes",
        false,
        theme,
    );

    let submit = icon_button(
        ButtonData {
            state: submit_button_state,
            icon: Icon::Save,
            text: "Submit",
            kind: ButtonKind::Primary,
            on_press: ModifyEntryMessage::Submit.some_if(can_submit),
        },
        "Submit changes",
        false,
        theme,
    );

    Row::new()
        .spacing(DEFAULT_ROW_SPACING)
        .push(cancel)
        .push(submit)
        .into()
}

/// Create the advanced area.
#[cfg_attr(coverage, no_coverage)]
fn advanced_area<'a, P: Platform + 'static>(
    button_state: &'a mut button::State,
    show_advanced: bool,
    advanced_state: &'a mut AdvancedState,
    state: State,
    entry_head: &'a EntryHead,
    entry_body: &'a EntryBody,
    theme: &dyn Theme,
) -> Element<'a, ModifyEntryMessage> {
    let advanced_button = Button::new(
        button_state,
        Row::new()
            .spacing(DEFAULT_ROW_SPACING)
            .push(
                Text::new(if show_advanced {
                    Icon::CaretDown
                } else {
                    Icon::CaretRight
                })
                .font(ICON_FONT),
            )
            .push(Text::new("Advanced")),
    )
    .style(theme.toggle_button_advanced_area())
    .on_press(ModifyEntryMessage::ToggleAdvanced);

    let advanced: Element<_> = if show_advanced {
        advanced_state
            .view::<P>(state, entry_head, entry_body, theme)
            .map(ModifyEntryMessage::Advanced)
    } else {
        Space::new(Length::Fill, Length::Shrink).into()
    };

    Column::new()
        .spacing(DEFAULT_COLUMN_SPACING)
        .push(advanced_button)
        .push(advanced)
        .into()
}

impl std::fmt::Debug for ModifyEntryView {
    #[cfg_attr(coverage, no_coverage)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("No debug info available for ModifyEntryView")
    }
}

/// The state of the entry
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// The entry was created.
    Create,
    /// An existing entry will be modified.
    Modify,
}

/// The state of the advanced view.
#[derive(Debug, Focus)]
pub struct AdvancedState {
    /// The state of the [`Button`](iced::Button) to delete the entry.
    delete: button::State,
    /// The state of the [`TextInput`](iced::TextInput) of the entry's auto type value.
    #[focus(enable)]
    auto_type: text_input::State,
}

/// The message produced by the advanced view.
#[derive(Clone, Debug)]
pub enum AdvancedStateMessage {
    /// The deletion of an entry was requested.
    DeleteEntryRequest,
    /// Change the auto type to the new value.
    AutoTypeInput(String),
}

impl AdvancedState {
    /// Create a new advanced state.
    pub fn new() -> Self {
        Self {
            delete: button::State::new(),
            auto_type: text_input::State::new(),
        }
    }

    /// Create the advanced view.
    #[cfg_attr(coverage, no_coverage)]
    pub fn view<P: Platform + 'static>(
        &mut self,
        state: State,
        entry_head: &EntryHead,
        _entry_body: &EntryBody,
        theme: &dyn Theme,
    ) -> Element<AdvancedStateMessage> {
        let delete: Element<_> = if state == State::Modify {
            icon_button(
                ButtonData {
                    state: &mut self.delete,
                    icon: Icon::Trash,
                    text: "Delete",
                    kind: ButtonKind::Warning,
                    on_press: Some(AdvancedStateMessage::DeleteEntryRequest),
                },
                "Delete this entry",
                false,
                theme,
            )
        } else {
            default_vertical_space().into()
        };

        let auto_type_label = Text::new("AutoType sequence:");

        let auto_type = default_text_input(
            &mut self.auto_type,
            "AutoType sequence",
            entry_head.auto_type_sequence(),
            AdvancedStateMessage::AutoTypeInput,
        )
        .style(theme.text_input());

        Column::new()
            .spacing(DEFAULT_COLUMN_SPACING)
            .push(default_vertical_space())
            .push(delete)
            .push(default_vertical_space())
            .push(auto_type_label)
            .push(auto_type)
            .into()
    }
}

/// The state of the modal.
#[derive(Debug)]
pub enum ModifyEntryModal {
    /// Confirm the deletion of the entry.
    DeleteRequest {
        /// The state of the cancel [`Button`](iced::Button).
        cancel_button_state: button::State,
        /// The state of the submit [`Button`](iced::Button).
        submit_button_state: button::State,
    },
    /// No modal.
    None,
}

/// The message send by the modal.
#[derive(Clone, Debug)]
pub enum ModifyEntryModalMessage {
    /// Close the modal.
    Close,
    /// Submit the deletion of the entry.
    SubmitDelete,
}

impl ModifyEntryModal {
    /// Create the modal to confirm the entry deletion.
    fn delete_request() -> Self {
        Self::DeleteRequest {
            cancel_button_state: button::State::new(),
            submit_button_state: button::State::new(),
        }
    }

    /// Create the view of the modal.
    #[cfg_attr(coverage, no_coverage)]
    pub fn view(&mut self, theme: &dyn Theme) -> Element<'_, ModifyEntryModalMessage> {
        match self {
            ModifyEntryModal::DeleteRequest {
                cancel_button_state,
                submit_button_state,
            } => Card::new(
                Text::new("Confirm deletion"),
                Text::new("Do you really want to delete this entry? This cannot be undone!"),
            )
            .foot(
                Row::new()
                    .spacing(DEFAULT_ROW_SPACING)
                    .push(icon_button(
                        ButtonData {
                            state: cancel_button_state,
                            icon: Icon::XSquare,
                            text: "Cancel",
                            kind: ButtonKind::Normal,
                            on_press: Some(ModifyEntryModalMessage::Close),
                        },
                        "Cancel the deletion of the entry",
                        false,
                        theme,
                    ))
                    .push(icon_button(
                        ButtonData {
                            state: submit_button_state,
                            icon: Icon::Save,
                            text: "Submit",
                            kind: ButtonKind::Warning,
                            on_press: Some(ModifyEntryModalMessage::SubmitDelete),
                        },
                        "Submit the deletion of the entry",
                        false,
                        theme,
                    )),
            )
            .style(theme.card_warning())
            .max_width(DEFAULT_MAX_WIDTH)
            .into(),
            ModifyEntryModal::None => Text::new("This message should never appear!").into(),
        }
    }
}

impl Default for ModifyEntryModal {
    fn default() -> Self {
        Self::None
    }
}

#[cfg(test)]
mod tests {

    use std::{
        any::{Any, TypeId},
        cell::RefCell,
        collections::HashMap,
        sync::Mutex,
    };

    use iced::Command;
    use mocktopus::mocking::*;
    use pwduck_core::{uuid, Uuid};
    use tempfile::tempdir;

    thread_local! {
        static CALL_MAP: RefCell<HashMap<TypeId, usize>> = RefCell::new(HashMap::new());
    }

    use crate::{PWDuckGuiError, TestPlatform};

    use super::{
        AdvancedState, AdvancedStateMessage, ModifyEntryMessage, ModifyEntryModal,
        ModifyEntryModalMessage, ModifyEntryView, State,
    };

    const DEFAULT_TITLE: &str = "default title";
    const DEFAULT_USERNAME: &str = "default username";
    const DEFAULT_PASSWORD: &str = "default password";

    fn default_mev_with_parent(parent: Uuid) -> ModifyEntryView {
        let head = pwduck_core::EntryHead::new(
            [1; uuid::SIZE].into(),
            parent,
            DEFAULT_TITLE.into(),
            [2; uuid::SIZE].into(),
        );
        let body = pwduck_core::EntryBody::new(
            [2; uuid::SIZE].into(),
            DEFAULT_USERNAME.into(),
            DEFAULT_PASSWORD.into(),
        );

        ModifyEntryView::with(State::Create, head, body)
    }

    fn default_mev() -> ModifyEntryView {
        default_mev_with_parent([0; uuid::SIZE].into())
    }

    fn equal_heads(a: &pwduck_core::EntryHead, b: &pwduck_core::EntryHead) -> bool {
        a.uuid() == b.uuid()
            && a.parent() == b.parent()
            && a.title() == b.title()
            && a.web_address() == b.web_address()
            && a.body() == b.body()
    }

    fn equal_bodies(a: &pwduck_core::EntryBody, b: &pwduck_core::EntryBody) -> bool {
        a.uuid() == b.uuid()
            && a.username() == b.username()
            && a.password() == b.password()
            && a.email() == b.email()
    }

    #[test]
    fn with() {
        let head = pwduck_core::EntryHead::new(
            [1; uuid::SIZE].into(),
            [0; uuid::SIZE].into(),
            "title".into(),
            [2; uuid::SIZE].into(),
        );
        let body = pwduck_core::EntryBody::new(
            [1; uuid::SIZE].into(),
            "username".into(),
            "password".into(),
        );

        let mev = ModifyEntryView::with(State::Create, head.clone(), body.clone());
        assert_eq!(mev.state, State::Create);
        assert!(equal_heads(&mev.entry_head, &head));
        assert!(equal_bodies(&mev.entry_body, &body));
        assert!(mev.title_state.is_focused());
        assert!(!mev.username_state.is_focused());
        assert!(!mev.password_state.is_focused());
        assert!(!mev.web_address_state.is_focused());
        assert!(!mev.email_state.is_focused());
        assert!(!mev.is_modified);
        assert!(!mev.show_advanced);

        let mev = ModifyEntryView::with(State::Modify, head.clone(), body.clone());
        assert_eq!(mev.state, State::Modify);
        assert!(equal_heads(&mev.entry_head, &head));
        assert!(equal_bodies(&mev.entry_body, &body));
        assert!(!mev.title_state.is_focused());
        assert!(!mev.username_state.is_focused());
        assert!(!mev.password_state.is_focused());
        assert!(!mev.web_address_state.is_focused());
        assert!(!mev.email_state.is_focused());
        assert!(!mev.is_modified);
        assert!(!mev.show_advanced);
    }

    #[test]
    fn contains_unsaved_changes() {
        let mev = default_mev();

        assert!(mev.entry_head().is_modified());
        assert!(mev.entry_body().is_modified());
        assert!(mev.contains_unsaved_changes());

        // TODO find a way to pass mocking from core to gui
    }

    #[test]
    fn update_title() {
        let mut mev = default_mev();

        assert_eq!(mev.entry_head().title().as_str(), DEFAULT_TITLE);
        assert!(!mev.is_modified);

        let _ = mev.update_title("title".into());

        assert_eq!(mev.entry_head().title().as_str(), "title");
        assert!(mev.is_modified);
    }

    #[test]
    fn update_username() {
        let mut mev = default_mev();

        assert_eq!(mev.entry_body().username().as_str(), DEFAULT_USERNAME);
        assert!(!mev.is_modified);

        let _ = mev.update_username("username".into());

        assert_eq!(mev.entry_body().username().as_str(), "username");
        assert!(mev.is_modified);
    }

    #[test]
    fn update_password() {
        let mut mev = default_mev();

        assert_eq!(mev.entry_body().password().as_str(), DEFAULT_PASSWORD);
        assert!(!mev.is_modified);

        let _ = mev.update_password("password".into());

        assert_eq!(mev.entry_body().password().as_str(), "password");
        assert!(mev.is_modified);
    }

    #[test]
    fn update_web_address() {
        let mut mev = default_mev();

        assert!(mev.entry_head().web_address().as_str().is_empty());
        assert!(!mev.is_modified);

        let _ = mev.update_web_address("https://example.com".into());

        assert_eq!(
            mev.entry_head().web_address().as_str(),
            "https://example.com"
        );
        assert!(mev.is_modified);
    }

    #[test]
    fn update_email() {
        let mut mev = default_mev();

        assert!(mev.entry_body().email().as_str().is_empty());
        assert!(!mev.is_modified);

        let _ = mev.update_email("example@web_address.de".into());

        assert_eq!(mev.entry_body().email().as_str(), "example@web_address.de");
        assert!(mev.is_modified);
    }

    #[test]
    fn toggle_password_visibility() {
        let mut mev = default_mev();

        assert!(!mev.password_show);

        let _ = mev.toggle_password_visibility();

        assert!(mev.password_show);

        let _ = mev.toggle_password_visibility();

        assert!(!mev.password_show);
    }

    #[test]
    fn toggle_advanced_visibility() {
        let mut mev = default_mev();

        assert!(!mev.show_advanced);

        let _ = mev.toggle_advanced_visibility();

        assert!(mev.show_advanced);

        let _ = mev.toggle_advanced_visibility();

        assert!(!mev.show_advanced);
    }

    #[test]
    fn set_password_score() {
        use iced::executor::Executor;

        let mut mev = default_mev();

        assert!(mev.password_score.is_none());

        let executor = iced::executor::Default::new().unwrap();

        executor.spawn(async move {
            let password_score =
                crate::utils::estimate_password_strength("this_is_a_password".into()).await;

            let _ = mev.set_password_score(password_score.clone());

            assert!(mev.password_score.is_some());
        });
    }

    #[test]
    fn submit() {
        let mev = default_mev();

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();
        let root = vault.get_root_uuid().unwrap();

        let mutex_mem_key = Mutex::new(mem_key);

        assert!(vault
            .get_item_list_for(&root, Some("default"))
            .groups()
            .is_empty());

        let expected_head = mev.entry_head().clone();

        mev.submit(&mut vault, &mutex_mem_key.lock().unwrap())
            .expect("Should not fail");

        assert!(equal_heads(
            &expected_head,
            vault
                .get_item_list_for(&root, Some("default"))
                .entries()
                .first()
                .unwrap()
        ));
    }

    #[test]
    fn request_entry_deletion() {
        let mut mev = default_mev();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::None);

        if let crate::ModalState::None = modal_state.inner() {
        } else {
            panic!("Modal state should be None");
        }

        let _ = mev.request_entry_deletion(&mut modal_state);

        if let crate::ModalState::ModifyEntry(ModifyEntryModal::DeleteRequest { .. }) =
            modal_state.inner()
        {
        } else {
            panic!("Modal state should be an delete request");
        }
    }

    #[test]
    fn update_auto_type_sequence() {
        let mut mev = default_mev();

        assert_eq!(
            mev.entry_head().auto_type_sequence().as_str(),
            pwduck_core::AutoTypeSequence::default().as_str()
        );

        let _ = mev.update_auto_type_sequence("Autotype".into());

        assert_eq!(mev.entry_head().auto_type_sequence().as_str(), "Autotype");
    }

    #[test]
    fn update_advanced() {
        let mut mev = default_mev();
        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::None);

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::request_entry_deletion.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::update_auto_type_sequence.type_id(), 0);

            ModifyEntryView::request_entry_deletion.mock_raw(|_self, _state| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::request_entry_deletion.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::update_auto_type_sequence.mock_raw(|_self, _sequence| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update_auto_type_sequence.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Request entry deletion
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::request_entry_deletion.type_id()],
                0
            );
            let _ = mev.update_advanced::<TestPlatform>(
                AdvancedStateMessage::DeleteEntryRequest,
                &mut modal_state,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::request_entry_deletion.type_id()],
                1
            );

            // Update auto type sequence.
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_auto_type_sequence.type_id()],
                0
            );
            let _ = mev.update_advanced::<TestPlatform>(
                AdvancedStateMessage::AutoTypeInput("Autotype".into()),
                &mut modal_state,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_auto_type_sequence.type_id()],
                1
            );

            assert!(call_map.borrow().values().all(|v| *v == 1));
        })
    }

    #[test]
    fn close_modal() {
        let mut mev = default_mev();

        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::ModifyEntry(
            ModifyEntryModal::delete_request(),
        ));

        if let crate::ModalState::ModifyEntry(ModifyEntryModal::DeleteRequest { .. }) =
            modal_state.inner()
        {
        } else {
            panic!("Modal state should be a delete request");
        }

        let _ = mev.close_modal(&mut modal_state);

        if let crate::ModalState::None = modal_state.inner() {
        } else {
            panic!("Modal state should be None");
        }
    }

    #[test]
    fn delete_entry() {
        let mut mev = default_mev();

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();
        let root = vault.get_root_uuid().unwrap();

        let mutex_mem_key = Mutex::new(mem_key);

        let _ = mev.submit(&mut vault, &mutex_mem_key.lock().unwrap());

        let expected_head = mev.entry_head().clone();

        assert!(equal_heads(
            &expected_head,
            vault
                .get_item_list_for(&root, Some("default"))
                .entries()
                .first()
                .unwrap()
        ));

        let _ = mev.delete_entry(&mut vault);

        assert!(vault
            .get_item_list_for(&root, Some("default"))
            .entries()
            .first()
            .is_none());
    }

    #[test]
    fn update_modal() {
        let mut mev = default_mev();

        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::ModifyEntry(
            ModifyEntryModal::delete_request(),
        ));

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::close_modal.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::delete_entry.type_id(), 0);

            ModifyEntryView::close_modal.mock_raw(|_self, _state| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::close_modal.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::delete_entry.mock_raw(|_self, _state| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::delete_entry.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });

            // Close modal
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::close_modal.type_id()],
                0
            );
            let _ = mev.update_modal(
                &ModifyEntryModalMessage::Close,
                &mut vault,
                &mut modal_state,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::close_modal.type_id()],
                1
            );

            // Delete entry
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::delete_entry.type_id()],
                0
            );
            let _ = mev.update_modal(
                &ModifyEntryModalMessage::SubmitDelete,
                &mut vault,
                &mut modal_state,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::delete_entry.type_id()],
                1
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::close_modal.type_id()],
                2
            );
        })
    }

    #[test]
    fn update() {
        let mut mev = default_mev();

        let mut modal_state = iced_aw::modal::State::new(crate::ModalState::ModifyEntry(
            ModifyEntryModal::delete_request(),
        ));

        let dir = tempdir().unwrap();
        let path = dir.path().join("TempVault");

        let password = "this_is_a_password";
        let mem_key = pwduck_core::MemKey::with_length(1);

        let mut vault =
            pwduck_core::Vault::generate(password, Option::<String>::None, &mem_key, &path)
                .unwrap();

        // WARNING: This is highly unsafe!
        #[allow(deref_nullptr)]
        let mut clipboard: &mut iced::Clipboard = unsafe { &mut *(std::ptr::null_mut()) };

        CALL_MAP.with(|call_map| unsafe {
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::update_title.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::update_username.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::copy_username.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::update_password.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::toggle_password_visibility.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::copy_password.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::update_web_address.type_id(), 0);
            call_map.borrow_mut().insert(
                ModifyEntryView::open_in_browser::<TestPlatform>.type_id(),
                0,
            );
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::update_email.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::set_password_score.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::toggle_advanced_visibility.type_id(), 0);
            call_map.borrow_mut().insert(
                ModifyEntryView::update_advanced::<TestPlatform>.type_id(),
                0,
            );
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::update_modal.type_id(), 0);
            call_map
                .borrow_mut()
                .insert(ModifyEntryView::submit.type_id(), 0);

            ModifyEntryView::update_title.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update_title.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::update_username.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update_username.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::copy_username.mock_raw(|_self, _clipboard| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::copy_username.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::update_password.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::toggle_password_visibility.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::toggle_password_visibility.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::copy_password.mock_raw(|_self, _clipboard| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::copy_password.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::update_web_address.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update_web_address.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::open_in_browser::<TestPlatform>.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::open_in_browser::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::update_email.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update_email.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::set_password_score.mock_raw(|_self, _value| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::set_password_score.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::toggle_advanced_visibility.mock_raw(|_self| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::toggle_advanced_visibility.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::update_advanced::<TestPlatform>.mock_raw(|_self, _message, _state| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update_advanced::<TestPlatform>.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::update_modal.mock_raw(|_self, _message, _vault, _state| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::update_modal.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Command::none())
            });
            ModifyEntryView::submit.mock_raw(|_self, _vault, _key| {
                call_map
                    .borrow_mut()
                    .get_mut(&ModifyEntryView::submit.type_id())
                    .map(|c| *c += 1);
                MockResult::Return(Ok(Command::none()))
            });

            // Update title
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_title.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::TitleInput("Title".into()),
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_title.type_id()],
                1
            );

            // Update username
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_username.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::UsernameInput("Username".into()),
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_username.type_id()],
                1
            );

            // Copy username
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::copy_username.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::UsernameCopy,
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::copy_username.type_id()],
                1
            );

            // Update password
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_password.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::PasswordInput("Password".into()),
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_password.type_id()],
                1
            );

            // Toggle password visibility
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::toggle_password_visibility.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::PasswordShow,
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::toggle_password_visibility.type_id()],
                1
            );

            // Copy password
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::copy_password.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::PasswordCopy,
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::copy_password.type_id()],
                1
            );

            // Update web address
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_web_address.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::WebAddressInput("Web".into()),
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_web_address.type_id()],
                1
            );

            // Open in browser
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::open_in_browser::<TestPlatform>.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::OpenInBrowser,
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::open_in_browser::<TestPlatform>.type_id()],
                1
            );

            // Result of opener
            let _ = mev
                .update::<TestPlatform>(
                    ModifyEntryMessage::Opener(Ok(())),
                    &mut vault,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");
            let _ = mev
                .update::<TestPlatform>(
                    ModifyEntryMessage::Opener(Err(crate::PWDuckGuiError::String("Oops".into()))),
                    &mut vault,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail");

            // Update email
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_email.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::EmailInput("Email".into()),
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_email.type_id()],
                1
            );

            // Set password score
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::set_password_score.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::PasswordScore(Err(pwduck_core::PWDuckCoreError::Error(
                    "".into(),
                ))),
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::set_password_score.type_id()],
                1
            );

            // Toggle advanced visibility
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::toggle_advanced_visibility.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::ToggleAdvanced,
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::toggle_advanced_visibility.type_id()],
                1
            );

            // Update advanced
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_advanced::<TestPlatform>.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::Advanced(AdvancedStateMessage::DeleteEntryRequest),
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_advanced::<TestPlatform>.type_id()],
                1
            );

            // Update modal
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_modal.type_id()],
                0
            );
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::Modal(ModifyEntryModalMessage::SubmitDelete),
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(
                call_map.borrow()[&ModifyEntryView::update_modal.type_id()],
                1
            );

            // Cancel
            let _ = mev
                .update::<TestPlatform>(
                    ModifyEntryMessage::Cancel,
                    &mut vault,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect("Should not fail");

            // Submit
            assert_eq!(call_map.borrow()[&ModifyEntryView::submit.type_id()], 0);
            let _ = mev.update::<TestPlatform>(
                ModifyEntryMessage::Submit,
                &mut vault,
                &mut modal_state,
                &mut clipboard,
            );
            assert_eq!(call_map.borrow()[&ModifyEntryView::submit.type_id()], 1);

            // Password generate
            let res = mev
                .update::<TestPlatform>(
                    ModifyEntryMessage::PasswordGenerate,
                    &mut vault,
                    &mut modal_state,
                    &mut clipboard,
                )
                .expect_err("Should fail");
            match res {
                PWDuckGuiError::Unreachable(_) => {}
                _ => panic!("Should contain unreachable warning."),
            }

            assert!(call_map.borrow().values().all(|v| *v == 1));
        });
    }

    #[test]
    fn new_advanced_state() {
        let state = AdvancedState::new();

        assert!(!state.auto_type.is_focused())
    }

    #[test]
    fn new_delete_request() {
        let modal_state = ModifyEntryModal::delete_request();

        if let ModifyEntryModal::DeleteRequest { .. } = modal_state {
        } else {
            panic!("State should be a delete request");
        }
    }

    #[test]
    fn default_modify_entry_modal() {
        let modal = ModifyEntryModal::default();
        if let ModifyEntryModal::None = modal {
        } else {
            panic!("ModifyEntryModal should be None");
        }
    }
}
