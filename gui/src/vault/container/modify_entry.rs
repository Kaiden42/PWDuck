//! TODO

use getset::{CopyGetters, Getters, MutGetters, Setters};
use iced::{
    button, scrollable, text_input, Button, Column, Command, Element, Length, Row, Scrollable,
    Space, Text,
};
use iced_aw::{modal, Card};
use iced_focus::Focus;
use pwduck_core::{EntryBody, EntryHead, PWDuckCoreError, PasswordInfo, Uuid, Vault};

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

    /// TODO
    #[getset(get_copy)]
    show_advanced: bool,
    /// TODO
    advanced_button_state: button::State,
    /// TODO
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

    /// TODO
    Modal(ModifyEntryModalMessage),
}
impl SomeIf for ModifyEntryMessage {}

#[cfg_attr(test, mockable)]
impl ModifyEntryView {
    /// Create a new [`ModifyEntryView`](ModifyEntryView).
    ///
    /// It expects:
    ///     - A new entry was created or an existing will be modified.
    ///     - The head of the entry to modify.
    ///     - The body of teh entry to modify.
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
        self.entry_head_mut().set_title(title);
        self.is_modified = true;
        Command::none()
    }

    /// Update the username and replace it with the given value.
    fn update_username(&mut self, username: String) -> Command<ModifyEntryMessage> {
        self.entry_body_mut().set_username(username);
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
        self.entry_body_mut().set_password(password);
        self.is_modified = true;
        self.estimate_password_strength()
    }

    /// Update the web address and replace it with the given value.
    fn update_web_address(&mut self, web_address: String) -> Command<ModifyEntryMessage> {
        self.entry_head_mut().set_web_address(web_address);
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
        self.entry_body_mut().set_email(email);
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
    fn toggle_advanced_visiblity(&mut self) -> Command<ModifyEntryMessage> {
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
    fn submit(&self, vault: &mut Vault) -> Result<Command<ModifyEntryMessage>, PWDuckGuiError> {
        // TODO async
        let mem_key = crate::MEM_KEY.lock()?;
        let masterkey = vault
            .masterkey()
            .as_unprotected(&mem_key, vault.salt(), vault.nonce())?;

        vault.insert_entry(self.entry_head.clone(), self.entry_body.clone(), &masterkey)?;

        Ok(Command::none())
    }

    /// Update the advanced state with the given message.
    fn update_advanced<P: Platform + 'static>(
        &mut self,
        message: AdvancedStateMessage,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyEntryMessage> {
        match message {
            AdvancedStateMessage::DeleteEntryRequest => {
                *modal_state = modal::State::new(crate::ModalState::ModifyEntry(
                    ModifyEntryModal::delete_request(),
                ));
                modal_state.show(true);
                Command::none()
            }
            AdvancedStateMessage::AutoTypeInput(auto_type_sequence) => {
                self.entry_head
                    .set_auto_type_sequence(auto_type_sequence.into());
                self.is_modified = true;
                Command::none()
            }
        }
    }

    /// Update the state of the modal.
    fn update_modal(
        &mut self,
        message: &ModifyEntryModalMessage,
        vault: &mut Vault,
        modal_state: &mut iced_aw::modal::State<crate::ModalState>,
    ) -> Command<ModifyEntryMessage> {
        match message {
            ModifyEntryModalMessage::Close => {
                *modal_state = modal::State::default();
                Command::none()
            }
            ModifyEntryModalMessage::SubmitDelete => {
                *modal_state = modal::State::default();
                vault.delete_entry(self.entry_head.uuid());
                Command::none()
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
            ModifyEntryMessage::ToggleAdvanced => Ok(self.toggle_advanced_visiblity()),
            ModifyEntryMessage::Advanced(message) => {
                Ok(self.update_advanced::<P>(message, modal_state))
            }
            ModifyEntryMessage::Modal(message) => {
                Ok(self.update_modal(&message, vault, modal_state))
            }
            //ModifyEntryMessage::PasswordGenerate
            //| ModifyEntryMessage::Cancel
            //| ModifyEntryMessage::Submit => {
            //    PWDuckGuiError::Unreachable("ModifyEntryMessage".into()).into()
            //}
            ModifyEntryMessage::Cancel => Ok(Command::none()),
            ModifyEntryMessage::Submit => self.submit(vault),
            ModifyEntryMessage::PasswordGenerate => {
                PWDuckGuiError::Unreachable("ModifyEntryMessage".into()).into()
            }
        }
    }

    /// Create the view of the [`ModifyEntryView`](ModifyEntryView).
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

        //let password_score: Element<_> = self.password_score.as_mut().map_or_else(
        //    || Container::new(default_vertical_space()).into(),
        //    PasswordScore::view,
        //);

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

    //if let Some(password_score) = password_score.as_mut() {
    //    Column::new()
    //        .spacing(DEFAULT_COLUMN_SPACING)
    //        .push(row)
    //        .push(password_score.view())
    //        .into()
    //} else {
    //    row.into()
    //}

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
