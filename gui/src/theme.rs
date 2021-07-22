//! TODO

use iced::{button, container, text_input, Color, Vector};
use iced_aw::{card, modal, split, tab_bar};

/// #02e790
const PRIMARY: Color = Color::from_rgb(0.008, 0.906, 0.565);
/// #01dd73
const PRIMARY_DARK: Color = Color::from_rgb(0.004, 0.867, 0.451);
/// #b3f8de
const PRIMARY_LIGHT: Color = Color::from_rgb(0.702, 0.973, 0.871);

/// #797979
const ACCENT: Color = Color::from_rgb(0.475, 0.475, 0.475);
/// #5c5c5c
const ACCENT_DARK: Color = Color::from_rgb(0.361, 0.361, 0.361);
/// #d7d7d7
const ACCENT_LIGHT: Color = Color::from_rgb(0.843, 0.843, 0.843);

/// #ff0000
const WARNING: Color = Color::from_rgb(1.0, 0.0, 0.0);
/// #ff0000
const WARNING_DARK: Color = Color::from_rgb(1.0, 0.0, 0.0);
/// #ffb3b3
const WARNING_LIGHT: Color = Color::from_rgb(1.0, 0.702, 0.702);

/// #000000
const TEXT_LIGHT: Color = Color::from_rgb(0.0, 0.0, 0.0);
/// #ffffff
const TEXT_DARK: Color = Color::from_rgb(1.0, 1.0, 1.0);

/// #fafafa
const BACKGROUND_LIGHT: Color = Color::from_rgb(0.98, 0.98, 0.98);
/// #2c2c2c
const BACKGROUND_DARK: Color = Color::from_rgb(0.173, 0.173, 0.173);

/// The theme of the application.
pub trait Theme: std::fmt::Debug {
    /// The style sheet of a text input.
    fn text_input(&self) -> Box<dyn text_input::StyleSheet>;
    /// The style sheet if the password mismatch.
    fn password_missmatch(&self) -> Box<dyn text_input::StyleSheet>;

    /// The style sheet of a container.
    fn container(&self) -> Box<dyn container::StyleSheet>;
    /// The style sheet of a container with accent.
    fn container_accent(&self) -> Box<dyn container::StyleSheet>;

    /// The style sheet of a default button.
    fn button(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of a primary button.
    fn button_primary(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of a warning button.
    fn button_warning(&self) -> Box<dyn button::StyleSheet>;

    /// The style sheet of a default modal.
    fn modal(&self) -> Box<dyn modal::StyleSheet>;
    /// The style sheet of a warning modal.
    fn modal_warning(&self) -> Box<dyn modal::StyleSheet>;
    /// The style sheet of a default card.
    fn card(&self) -> Box<dyn card::StyleSheet>;
    /// The style sheet of a warning card.
    fn card_warning(&self) -> Box<dyn card::StyleSheet>;

    /// The style sheet of the `TabBar`.
    fn tab_bar(&self) -> Box<dyn tab_bar::StyleSheet>;
    /// The style sheet of the `Split`.
    fn split(&self) -> Box<dyn split::StyleSheet>;

    /// The style sheet of a group list item.
    fn list_item_group(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of an entry list item.
    fn list_item_entry(&self) -> Box<dyn button::StyleSheet>;

    /// The style sheet of the tooltip.
    fn tooltip(&self) -> Box<dyn container::StyleSheet>;
    /// The style sheet for an active toggle button;
    fn toggle_button_active(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet for an inactive toggle button.
    fn toggle_button_inactive(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of the toggle button for the advanced area.
    fn toggle_button_advanced_area(&self) -> Box<dyn button::StyleSheet>;

    /// The style sheet of a tree node.
    fn tree_node(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of the expand button of the tree view.
    fn tree_expand_button(&self) -> Box<dyn button::StyleSheet>;
}

/// TODO
#[derive(Debug)]
pub struct Light;

impl Theme for Light {
    #[allow(clippy::missing_docs_in_private_items)]
    fn text_input(&self) -> Box<dyn text_input::StyleSheet> {
        struct Style;
        impl text_input::StyleSheet for Style {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: BACKGROUND_LIGHT.into(),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: ACCENT_LIGHT,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_color: PRIMARY,
                    ..self.active()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..TEXT_LIGHT
                }
            }

            fn value_color(&self) -> Color {
                TEXT_LIGHT
            }

            fn selection_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..TEXT_LIGHT
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn password_missmatch(&self) -> Box<dyn text_input::StyleSheet> {
        struct Style;
        impl text_input::StyleSheet for Style {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: BACKGROUND_LIGHT.into(),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: WARNING,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style { ..self.active() }
            }

            fn placeholder_color(&self) -> Color {
                Color { a: 0.87, ..WARNING }
            }

            fn value_color(&self) -> Color {
                WARNING
            }

            fn selection_color(&self) -> Color {
                Color { a: 0.87, ..WARNING }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn container(&self) -> Box<dyn container::StyleSheet> {
        struct Style;
        impl container::StyleSheet for Style {
            fn style(&self) -> container::Style {
                container::Style {
                    text_color: Some(TEXT_LIGHT),
                    background: Some(BACKGROUND_LIGHT.into()),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn container_accent(&self) -> Box<dyn container::StyleSheet> {
        struct Style;
        impl container::StyleSheet for Style {
            fn style(&self) -> container::Style {
                container::Style {
                    text_color: Some(TEXT_LIGHT),
                    //background: Some(Color::from_rgb(0.97, 0.97, 0.97).into()),
                    background: Some(
                        Color {
                            r: BACKGROUND_LIGHT.r - 0.03,
                            g: BACKGROUND_LIGHT.g - 0.03,
                            b: BACKGROUND_LIGHT.b - 0.03,
                            a: BACKGROUND_LIGHT.a,
                        }
                        .into(),
                    ),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn button(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::default(),
                    background: Some(PRIMARY_LIGHT.into()),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn button_primary(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::default(),
                    background: Some(PRIMARY.into()),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn button_warning(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::default(),
                    background: Some(WARNING.into()),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn modal(&self) -> Box<dyn modal::StyleSheet> {
        struct Style;
        impl modal::StyleSheet for Style {
            fn active(&self) -> modal::Style {
                modal::Style {
                    background: Color {
                        a: 0.3,
                        ..ACCENT_LIGHT
                    }
                    .into(),
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn modal_warning(&self) -> Box<dyn modal::StyleSheet> {
        struct Style;
        impl modal::StyleSheet for Style {
            fn active(&self) -> modal::Style {
                modal::Style {
                    background: Color {
                        a: 0.3,
                        ..WARNING_LIGHT
                    }
                    .into(),
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn card(&self) -> Box<dyn card::StyleSheet> {
        struct Style;
        impl card::StyleSheet for Style {
            fn active(&self) -> card::Style {
                card::Style {
                    background: BACKGROUND_LIGHT.into(),
                    border_radius: 10.0,
                    border_width: 1.0,
                    border_color: PRIMARY,
                    head_background: PRIMARY.into(),
                    head_text_color: TEXT_LIGHT,
                    body_background: Color::TRANSPARENT.into(),
                    body_text_color: TEXT_LIGHT,
                    foot_background: Color::TRANSPARENT.into(),
                    foot_text_color: TEXT_LIGHT,
                    close_color: TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn card_warning(&self) -> Box<dyn card::StyleSheet> {
        struct Style;
        impl card::StyleSheet for Style {
            fn active(&self) -> card::Style {
                card::Style {
                    background: BACKGROUND_LIGHT.into(),
                    border_radius: 10.0,
                    border_width: 1.0,
                    border_color: WARNING,
                    head_background: WARNING.into(),
                    head_text_color: TEXT_DARK,
                    body_background: Color::TRANSPARENT.into(),
                    body_text_color: TEXT_LIGHT,
                    foot_background: Color::TRANSPARENT.into(),
                    foot_text_color: TEXT_LIGHT,
                    close_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn tab_bar(&self) -> Box<dyn tab_bar::StyleSheet> {
        struct Style;
        impl tab_bar::StyleSheet for Style {
            fn active(&self, is_active: bool) -> tab_bar::Style {
                tab_bar::Style {
                    background: None,
                    border_color: None,
                    border_width: 0.0,
                    tab_label_background: if is_active {
                        PRIMARY.into()
                    } else {
                        PRIMARY_LIGHT.into()
                    },
                    tab_label_border_color: Color::TRANSPARENT,
                    tab_label_border_width: 0.0,
                    icon_color: TEXT_LIGHT,
                    text_color: TEXT_LIGHT,
                }
            }

            fn hovered(&self, is_active: bool) -> tab_bar::Style {
                tab_bar::Style {
                    tab_label_background: PRIMARY_DARK.into(),
                    ..self.active(is_active)
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn split(&self) -> Box<dyn split::StyleSheet> {
        struct Style;
        impl split::StyleSheet for Style {
            fn active(&self) -> split::Style {
                split::Style {
                    background: Some(BACKGROUND_LIGHT.into()),
                    first_background: Some(BACKGROUND_LIGHT.into()),
                    second_background: Some(BACKGROUND_LIGHT.into()),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    divider_background: BACKGROUND_LIGHT.into(),
                    divider_border_width: 0.0,
                    divider_border_color: Color::TRANSPARENT,
                }
            }

            fn hovered(&self) -> split::Style {
                split::Style {
                    divider_background: ACCENT_LIGHT.into(),
                    ..self.active()
                }
            }

            fn dragged(&self) -> split::Style {
                split::Style {
                    divider_background: ACCENT_DARK.into(),
                    ..self.active()
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn list_item_group(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(1.0, 1.0),
                    background: Some(BACKGROUND_LIGHT.into()),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: ACCENT,
                    text_color: TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn list_item_entry(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(1.0, 1.0),
                    background: Some(BACKGROUND_LIGHT.into()),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: ACCENT_LIGHT,
                    text_color: TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn tooltip(&self) -> Box<dyn container::StyleSheet> {
        struct Style;
        impl container::StyleSheet for Style {
            fn style(&self) -> container::Style {
                container::Style {
                    text_color: Some(TEXT_LIGHT),
                    background: Some(BACKGROUND_LIGHT.into()),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: ACCENT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn toggle_button_active(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: Some(PRIMARY.into()),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn toggle_button_inactive(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: Some(PRIMARY_LIGHT.into()),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn toggle_button_advanced_area(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: Color::TRANSPARENT.into(),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn tree_node(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: None,
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_LIGHT,
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(PRIMARY_LIGHT.into()),
                    ..self.active()
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn tree_expand_button(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: Some(BACKGROUND_LIGHT.into()),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: PRIMARY,
                    text_color: PRIMARY,
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(PRIMARY.into()),
                    border_color: PRIMARY,
                    text_color: BACKGROUND_LIGHT,
                    ..self.active()
                }
            }
        }
        Style.into()
    }
}

impl From<Light> for Box<dyn Theme> {
    fn from(theme: Light) -> Self {
        Box::new(theme)
    }
}

/// TODO
#[derive(Debug)]
pub struct Dark;

impl Theme for Dark {
    fn text_input(&self) -> Box<dyn text_input::StyleSheet> {
        struct Style;
        impl text_input::StyleSheet for Style {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: BACKGROUND_DARK.into(),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: ACCENT_LIGHT,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_color: PRIMARY,
                    ..self.active()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..TEXT_DARK
                }
            }

            fn value_color(&self) -> Color {
                TEXT_DARK
            }

            fn selection_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..TEXT_DARK
                }
            }
        }
        Style.into()
    }

    fn password_missmatch(&self) -> Box<dyn text_input::StyleSheet> {
        struct Style;
        impl text_input::StyleSheet for Style {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: BACKGROUND_DARK.into(),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: WARNING,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style { ..self.active() }
            }

            fn placeholder_color(&self) -> Color {
                Color { a: 0.87, ..WARNING }
            }

            fn value_color(&self) -> Color {
                WARNING
            }

            fn selection_color(&self) -> Color {
                Color { a: 0.87, ..WARNING }
            }
        }
        Style.into()
    }

    fn container(&self) -> Box<dyn container::StyleSheet> {
        struct Style;
        impl container::StyleSheet for Style {
            fn style(&self) -> container::Style {
                container::Style {
                    text_color: Some(TEXT_DARK),
                    background: Some(BACKGROUND_DARK.into()),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
        }
        Style.into()
    }

    fn container_accent(&self) -> Box<dyn container::StyleSheet> {
        struct Style;
        impl container::StyleSheet for Style {
            fn style(&self) -> container::Style {
                container::Style {
                    text_color: Some(TEXT_DARK),
                    background: Some(
                        Color {
                            r: BACKGROUND_DARK.r + 0.03,
                            g: BACKGROUND_DARK.g + 0.03,
                            b: BACKGROUND_DARK.b + 0.03,
                            a: BACKGROUND_DARK.a,
                        }
                        .into(),
                    ),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
        }
        Style.into()
    }

    fn button(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::default(),
                    background: Some(PRIMARY_LIGHT.into()),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn button_primary(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::default(),
                    background: Some(PRIMARY.into()),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn button_warning(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::default(),
                    background: Some(WARNING.into()),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn modal(&self) -> Box<dyn modal::StyleSheet> {
        struct Style;
        impl modal::StyleSheet for Style {
            fn active(&self) -> modal::Style {
                modal::Style {
                    background: Color {
                        a: 0.3,
                        ..ACCENT_DARK
                    }
                    .into(),
                }
            }
        }
        Style.into()
    }

    fn modal_warning(&self) -> Box<dyn modal::StyleSheet> {
        struct Style;
        impl modal::StyleSheet for Style {
            fn active(&self) -> modal::Style {
                modal::Style {
                    background: Color {
                        a: 0.3,
                        ..WARNING_DARK
                    }
                    .into(),
                }
            }
        }
        Style.into()
    }

    fn card(&self) -> Box<dyn card::StyleSheet> {
        struct Style;
        impl card::StyleSheet for Style {
            fn active(&self) -> card::Style {
                card::Style {
                    background: BACKGROUND_DARK.into(),
                    border_radius: 10.0,
                    border_width: 1.0,
                    border_color: PRIMARY,
                    head_background: PRIMARY.into(),
                    head_text_color: TEXT_DARK,
                    body_background: Color::TRANSPARENT.into(),
                    body_text_color: TEXT_DARK,
                    foot_background: Color::TRANSPARENT.into(),
                    foot_text_color: TEXT_DARK,
                    close_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn card_warning(&self) -> Box<dyn card::StyleSheet> {
        struct Style;
        impl card::StyleSheet for Style {
            fn active(&self) -> card::Style {
                card::Style {
                    background: BACKGROUND_DARK.into(),
                    border_radius: 10.0,
                    border_width: 1.0,
                    border_color: WARNING,
                    head_background: WARNING.into(),
                    head_text_color: TEXT_DARK,
                    body_background: Color::TRANSPARENT.into(),
                    body_text_color: TEXT_DARK,
                    foot_background: Color::TRANSPARENT.into(),
                    foot_text_color: TEXT_DARK,
                    close_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn tab_bar(&self) -> Box<dyn tab_bar::StyleSheet> {
        struct Style;
        impl tab_bar::StyleSheet for Style {
            fn active(&self, is_active: bool) -> tab_bar::Style {
                tab_bar::Style {
                    background: None,
                    border_color: None,
                    border_width: 0.0,
                    tab_label_background: if is_active {
                        PRIMARY.into()
                    } else {
                        PRIMARY_LIGHT.into()
                    },
                    tab_label_border_color: Color::TRANSPARENT,
                    tab_label_border_width: 0.0,
                    icon_color: TEXT_DARK,
                    text_color: TEXT_DARK,
                }
            }

            fn hovered(&self, is_active: bool) -> tab_bar::Style {
                tab_bar::Style {
                    tab_label_background: PRIMARY_DARK.into(),
                    ..self.active(is_active)
                }
            }
        }
        Style.into()
    }

    fn split(&self) -> Box<dyn split::StyleSheet> {
        struct Style;
        impl split::StyleSheet for Style {
            fn active(&self) -> split::Style {
                split::Style {
                    background: Some(BACKGROUND_DARK.into()),
                    first_background: Some(BACKGROUND_DARK.into()),
                    second_background: Some(BACKGROUND_DARK.into()),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    divider_background: BACKGROUND_DARK.into(),
                    divider_border_width: 0.0,
                    divider_border_color: Color::TRANSPARENT,
                }
            }

            fn hovered(&self) -> split::Style {
                split::Style {
                    divider_background: ACCENT_DARK.into(),
                    ..self.active()
                }
            }

            fn dragged(&self) -> split::Style {
                split::Style {
                    divider_background: ACCENT_LIGHT.into(),
                    ..self.active()
                }
            }
        }
        Style.into()
    }

    fn list_item_group(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(1.0, 1.0),
                    background: Some(BACKGROUND_DARK.into()),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: ACCENT_DARK,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn list_item_entry(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(1.0, 1.0),
                    background: Some(BACKGROUND_DARK.into()),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: ACCENT,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn tooltip(&self) -> Box<dyn container::StyleSheet> {
        struct Style;
        impl container::StyleSheet for Style {
            fn style(&self) -> container::Style {
                container::Style {
                    text_color: Some(TEXT_DARK),
                    background: Some(BACKGROUND_DARK.into()),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: ACCENT_DARK,
                }
            }
        }
        Style.into()
    }

    fn toggle_button_active(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: Some(PRIMARY.into()),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn toggle_button_inactive(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: Some(PRIMARY_LIGHT.into()),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn toggle_button_advanced_area(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: Color::TRANSPARENT.into(),
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    fn tree_node(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: None,
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: TEXT_DARK,
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(PRIMARY_DARK.into()),
                    ..self.active()
                }
            }
        }
        Style.into()
    }

    fn tree_expand_button(&self) -> Box<dyn button::StyleSheet> {
        struct Style;
        impl button::StyleSheet for Style {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::new(0.0, 0.0),
                    background: Some(BACKGROUND_DARK.into()),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: PRIMARY_DARK,
                    text_color: PRIMARY_DARK,
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(PRIMARY.into()),
                    border_color: PRIMARY,
                    text_color: BACKGROUND_DARK,
                    ..self.active()
                }
            }
        }
        Style.into()
    }
}

impl From<Dark> for Box<dyn Theme> {
    fn from(theme: Dark) -> Self {
        Box::new(theme)
    }
}
