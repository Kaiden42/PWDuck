//! TODO

use iced::{button, container, slider, text_input, Background, Color, Vector};
use iced_aw::{card, modal, number_input, split, tab_bar};

// TODO: replace this with constant functions once floating-point arithmetic is const
// see <https://github.com/rust-lang/rust/issues/57241>
use lazy_static::lazy_static;
lazy_static! {
    /// The primary color (#02e790).
    static ref PRIMARY_1: Color = Color::from_rgb(0.008, 0.906, 0.565);
    /// The primary color with a shade of 10%.
    static ref PRIMARY_2: Color = shade(*PRIMARY_1, 0.9);
    /// The primary color with a shade of 20%.
    static ref PRIMARY_3: Color = shade(*PRIMARY_1, 0.8);
    /// The primary color with a shade of 30%.
    static ref PRIMARY_4: Color = shade(*PRIMARY_1, 0.7);
    /// The primary color with a shade of 40%.
    static ref PRIMARY_5: Color = shade(*PRIMARY_1, 0.6);
    /// The primary color with a shade of 50%.
    static ref PRIMARY_6: Color = shade(*PRIMARY_1, 0.5);
    /// The secondary color (#797979).
    //static ref SECONDARY_1: Color = Color::from_rgb(0.475, 0.475, 0.475);
    static ref SECONDARY_1: Color = Color::from_rgb(0.835, 0.835, 0.835); // D5
    /// The secondary color with a shade of 10%.
    static ref SECONDARY_2: Color = shade(*SECONDARY_1, 0.9);
    /// The secondary color with a shade of 20%.
    static ref SECONDARY_3: Color = shade(*SECONDARY_1, 0.8);
    /// The secondary color with a shade of 30%.
    static ref SECONDARY_4: Color = shade(*SECONDARY_1, 0.7);
    /// The secondary color with a shade of 40%.
    static ref SECONDARY_5: Color = shade(*SECONDARY_1, 0.6);
    /// The secondary color with a shade of 50%.
    static ref SECONDARY_6: Color = shade(*SECONDARY_1, 0.5);
    /// The warning color (#ff0000)
    static ref WARNING_1: Color = Color::from_rgb(1.0, 0.0, 0.0);
    /// The warning color with a shade of 10%.
    static ref WARNING_2: Color = shade(*WARNING_1, 0.9);
    /// The warning color with a shade of 20%.
    static ref WARNING_3: Color = shade(*WARNING_1, 0.8);
    /// The warning color with a shade of 30%.
    static ref WARNING_4: Color = shade(*WARNING_1, 0.7);
    /// The warning color with a shade of 40%.
    static ref WARNING_5: Color = shade(*WARNING_1, 0.6);
    /// The warning color with a shade of 50%.
    static ref WARNING_6: Color = shade(*WARNING_1, 0.5);
    /// The text color of the light theme (#000000).
    //static ref TEXT_LIGHT: Color = Color::from_rgb(0.0, 0.0, 0.0);
    static ref TEXT_LIGHT: Color = Color::from_rgb(0.173, 0.173, 0.173);
    /// The text color of the dark theme (#ffffff).
    //static ref TEXT_DARK: Color = Color::from_rgb(1.0, 1.0, 1.0);
    static ref TEXT_DARK: Color = Color::from_rgb(0.98, 0.98, 0.98);
    /// The background color of the light theme (#fafafa).
    static ref BACKGROUND_LIGHT: Color = Color::from_rgb(0.98, 0.98, 0.98);
    /// The background color of the dark theme (#2c2c2c).
    static ref BACKGROUND_DARK: Color = Color::from_rgb(0.173, 0.173, 0.173);
}
/// Calculates the tint of the given color based on the specified tint-factor.
/// Thanks to: <https://maketintsandshades.com/about>
fn tint(mut color: Color, factor: f32) -> Color {
    color.r += (1.0 - color.r) * factor;
    color.g += (1.0 - color.g) * factor;
    color.b += (1.0 - color.b) * factor;
    color
}

/// Calculates the shade of the given color based on the specified shade-factor.
/// Thanks to: <https://maketintsandshades.com/about>
fn shade(mut color: Color, factor: f32) -> Color {
    color.r *= factor;
    color.g *= factor;
    color.b *= factor;
    color
}

/// The theme of the application.
pub trait Theme: std::fmt::Debug {
    /// The style sheet of a [`TextInput`](iced::TextInput).
    fn text_input(&self) -> Box<dyn text_input::StyleSheet>;
    /// The style sheet if the password mismatch.
    fn password_missmatch(&self) -> Box<dyn text_input::StyleSheet>;

    /// The style sheet of a [`Container`](iced::Container).
    fn container(&self) -> Box<dyn container::StyleSheet>;
    /// The style sheet of a [`Container`](iced::Container) with accent.
    fn container_accent(&self) -> Box<dyn container::StyleSheet>;

    /// The style sheet of a default [`Button`](iced::Button).
    fn button(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of a primary [`Button`](iced::Button).
    fn button_primary(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of a warning [`Button`](iced::Button).
    fn button_warning(&self) -> Box<dyn button::StyleSheet>;

    /// The style sheet of a default [`Modal`](iced_aw::Modal).
    fn modal(&self) -> Box<dyn modal::StyleSheet>;
    /// The style sheet of a warning [`Modal`](iced_aw::Modal).
    fn modal_warning(&self) -> Box<dyn modal::StyleSheet>;
    /// The style sheet of a default [`Card`](iced_aw::Card).
    fn card(&self) -> Box<dyn card::StyleSheet>;
    /// The style sheet of a warning [`Card`](iced_aw::Card).
    fn card_warning(&self) -> Box<dyn card::StyleSheet>;

    /// The style sheet of the [`TabBar`](iced_aw::TabBar).
    fn tab_bar(&self) -> Box<dyn tab_bar::StyleSheet>;
    /// The style sheet of the [`Split`](iced_aw::Split).
    fn split(&self) -> Box<dyn split::StyleSheet>;

    /// The style sheet of a group list item.
    fn list_item_group(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of an entry list item.
    fn list_item_entry(&self) -> Box<dyn button::StyleSheet>;

    /// The style sheet of the [`Tooltip`](iced::Tooltip).
    fn tooltip(&self) -> Box<dyn container::StyleSheet>;
    /// The style sheet for an active toggle [`Button`](iced::Button);
    fn toggle_button_active(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet for an inactive toggle [`Button`](iced::Button).
    fn toggle_button_inactive(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of the toggle [`Button`](iced::Button) for the advanced area.
    fn toggle_button_advanced_area(&self) -> Box<dyn button::StyleSheet>;

    /// The style sheet of a tree node.
    fn tree_node(&self) -> Box<dyn button::StyleSheet>;
    /// The style sheet of the expand [`Button`](iced::Button) of the tree view.
    fn tree_expand_button(&self) -> Box<dyn button::StyleSheet>;

    /// The style sheet of a [`NumberInput`](iced::Button).
    fn number_input(&self) -> Box<dyn number_input::StyleSheet>;
    /// The style sheet of a [`Slider`](iced::Slider).
    fn slider(&self) -> Box<dyn slider::StyleSheet>;
}

/// The light theme of the application.
#[derive(Debug)]
pub struct Light;

impl Theme for Light {
    #[allow(clippy::missing_docs_in_private_items)]
    fn text_input(&self) -> Box<dyn text_input::StyleSheet> {
        struct Style;
        impl text_input::StyleSheet for Style {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: Background::Color(*BACKGROUND_LIGHT),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_1,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_color: *PRIMARY_2,
                    ..self.active()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..*TEXT_LIGHT
                }
            }

            fn value_color(&self) -> Color {
                *TEXT_LIGHT
            }

            fn selection_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..*TEXT_LIGHT
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
                    background: Background::Color(*BACKGROUND_LIGHT),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: *WARNING_2,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style { ..self.active() }
            }

            fn placeholder_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..*WARNING_2
                }
            }

            fn value_color(&self) -> Color {
                *WARNING_2
            }

            fn selection_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..*WARNING_2
                }
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
                    text_color: Some(*TEXT_LIGHT),
                    background: Some(Background::Color(*BACKGROUND_LIGHT)),
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
                    text_color: Some(*TEXT_LIGHT),
                    //background: Some(Color::from_rgb(0.97, 0.97, 0.97).into()),
                    background: Some(
                        //Color {
                        //    r: BACKGROUND_LIGHT.r + 0.03,
                        //    g: BACKGROUND_LIGHT.g + 0.03,
                        //    b: BACKGROUND_LIGHT.b + 0.03,
                        //    a: BACKGROUND_LIGHT.a,
                        //}
                        //.into(),
                        // TODO
                        tint(*BACKGROUND_LIGHT, 0.1).into(), //shade(*BACKGROUND_LIGHT, 0.99).into()
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
                    background: Some(Background::Color(*SECONDARY_1)),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_LIGHT,
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
                    background: Some(Background::Color(*PRIMARY_2)),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_LIGHT,
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
                    background: Some(Background::Color(*WARNING_2)),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_DARK,
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
                        a: 0.5,
                        ..*BACKGROUND_LIGHT
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
                        ..*WARNING_1
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
                    background: Background::Color(*BACKGROUND_LIGHT),
                    border_radius: 10.0,
                    border_width: 1.0,
                    border_color: *PRIMARY_2,
                    head_background: Background::Color(*PRIMARY_2),
                    head_text_color: *TEXT_LIGHT,
                    body_background: Color::TRANSPARENT.into(),
                    body_text_color: *TEXT_LIGHT,
                    foot_background: Color::TRANSPARENT.into(),
                    foot_text_color: *TEXT_LIGHT,
                    close_color: *TEXT_LIGHT,
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
                    background: Background::Color(*BACKGROUND_LIGHT),
                    border_radius: 10.0,
                    border_width: 1.0,
                    border_color: *WARNING_2,
                    head_background: Background::Color(*WARNING_2),
                    head_text_color: *TEXT_DARK,
                    body_background: Color::TRANSPARENT.into(),
                    body_text_color: *TEXT_LIGHT,
                    foot_background: Color::TRANSPARENT.into(),
                    foot_text_color: *TEXT_LIGHT,
                    close_color: *TEXT_DARK,
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
                    tab_label_background: Background::Color(if is_active {
                        *PRIMARY_2
                    } else {
                        *PRIMARY_1
                    }),
                    tab_label_border_color: Color::TRANSPARENT,
                    tab_label_border_width: 0.0,
                    icon_color: *TEXT_LIGHT,
                    text_color: *TEXT_LIGHT,
                }
            }

            fn hovered(&self, is_active: bool) -> tab_bar::Style {
                tab_bar::Style {
                    tab_label_background: Background::Color(*PRIMARY_3),
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
                    background: Some(Background::Color(*BACKGROUND_LIGHT)),
                    first_background: Some(Background::Color(*BACKGROUND_LIGHT)),
                    second_background: Some(Background::Color(*BACKGROUND_LIGHT)),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    divider_background: Background::Color(*BACKGROUND_LIGHT),
                    divider_border_width: 0.0,
                    divider_border_color: Color::TRANSPARENT,
                }
            }

            fn hovered(&self) -> split::Style {
                split::Style {
                    divider_background: Background::Color(*SECONDARY_1),
                    ..self.active()
                }
            }

            fn dragged(&self) -> split::Style {
                split::Style {
                    divider_background: Background::Color(*SECONDARY_3),
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
                    background: Some(Background::Color(*BACKGROUND_LIGHT)),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_2,
                    text_color: *TEXT_LIGHT,
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
                    background: Some(Background::Color(*BACKGROUND_LIGHT)),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_1,
                    text_color: *TEXT_LIGHT,
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
                    text_color: Some(*TEXT_LIGHT),
                    background: Some(Background::Color(*BACKGROUND_LIGHT)),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_1,
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
                    background: Some(Background::Color(*PRIMARY_2)),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_LIGHT,
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
                    background: Some(Background::Color(*PRIMARY_1)),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_LIGHT,
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
                    text_color: *TEXT_LIGHT,
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
                    text_color: *TEXT_LIGHT,
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(*SECONDARY_1)),
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
                    background: Some(Background::Color(*BACKGROUND_LIGHT)),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_3,
                    text_color: *SECONDARY_3,
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(*SECONDARY_3)),
                    border_color: *SECONDARY_3,
                    text_color: *BACKGROUND_LIGHT,
                    ..self.active()
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn number_input(&self) -> Box<dyn number_input::StyleSheet> {
        struct Style;
        impl number_input::StyleSheet for Style {
            fn active(&self) -> number_input::Style {
                number_input::Style {
                    button_background: None,
                    icon_color: *TEXT_LIGHT,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn slider(&self) -> Box<dyn slider::StyleSheet> {
        struct Style;
        impl slider::StyleSheet for Style {
            fn active(&self) -> slider::Style {
                slider::Style {
                    rail_colors: (
                        Color {
                            a: 0.5,
                            ..*SECONDARY_6
                        },
                        *BACKGROUND_LIGHT,
                    ),
                    handle: iced::slider::Handle {
                        shape: iced::slider::HandleShape::Rectangle {
                            width: 8,
                            border_radius: 4.0,
                        },
                        color: *SECONDARY_4,
                        border_color: *SECONDARY_5,
                        border_width: 1.0,
                    },
                }
            }

            fn hovered(&self) -> slider::Style {
                let active = self.active();
                slider::Style {
                    handle: iced::slider::Handle {
                        color: *SECONDARY_6,
                        ..active.handle
                    },
                    ..active
                }
            }

            fn dragging(&self) -> slider::Style {
                let active = self.active();
                slider::Style {
                    handle: iced::slider::Handle {
                        color: *SECONDARY_5,
                        ..active.handle
                    },
                    ..active
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

/// The dark theme of the application.
#[derive(Debug)]
pub struct Dark;

impl Theme for Dark {
    #[allow(clippy::missing_docs_in_private_items)]
    fn text_input(&self) -> Box<dyn text_input::StyleSheet> {
        struct Style;
        impl text_input::StyleSheet for Style {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: Background::Color(*BACKGROUND_DARK),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_6,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_color: *PRIMARY_5,
                    ..self.active()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..*TEXT_DARK
                }
            }

            fn value_color(&self) -> Color {
                *TEXT_DARK
            }

            fn selection_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..*TEXT_DARK
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
                    background: Background::Color(*BACKGROUND_DARK),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: *WARNING_5,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style { ..self.active() }
            }

            fn placeholder_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..*WARNING_5
                }
            }

            fn value_color(&self) -> Color {
                *WARNING_5
            }

            fn selection_color(&self) -> Color {
                Color {
                    a: 0.87,
                    ..*WARNING_5
                }
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
                    text_color: Some(*TEXT_DARK),
                    background: Some(Background::Color(*BACKGROUND_DARK)),
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
                    text_color: Some(*TEXT_DARK),
                    background: Some(
                        //Color {
                        //    r: BACKGROUND_DARK.r + 0.03,
                        //    g: BACKGROUND_DARK.g + 0.03,
                        //    b: BACKGROUND_DARK.b + 0.03,
                        //    a: BACKGROUND_DARK.a,
                        //}
                        //.into(),
                        // TODO
                        tint(*BACKGROUND_DARK, 0.03).into(),
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
                    background: Some(Background::Color(*SECONDARY_6)),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_DARK,
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
                    background: Some(Background::Color(*PRIMARY_5)),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_DARK,
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
                    background: Some(Background::Color(*WARNING_5)),
                    border_radius: 5.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_DARK,
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
                        a: 0.5,
                        ..*BACKGROUND_DARK
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
                        ..*WARNING_6
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
                    background: Background::Color(*BACKGROUND_DARK),
                    border_radius: 10.0,
                    border_width: 1.0,
                    border_color: *PRIMARY_5,
                    head_background: Background::Color(*PRIMARY_5),
                    head_text_color: *TEXT_DARK,
                    body_background: Color::TRANSPARENT.into(),
                    body_text_color: *TEXT_DARK,
                    foot_background: Color::TRANSPARENT.into(),
                    foot_text_color: *TEXT_DARK,
                    close_color: *TEXT_DARK,
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
                    background: Background::Color(*BACKGROUND_DARK),
                    border_radius: 10.0,
                    border_width: 1.0,
                    border_color: *WARNING_5,
                    head_background: Background::Color(*WARNING_5),
                    head_text_color: *TEXT_DARK,
                    body_background: Color::TRANSPARENT.into(),
                    body_text_color: *TEXT_DARK,
                    foot_background: Color::TRANSPARENT.into(),
                    foot_text_color: *TEXT_DARK,
                    close_color: *TEXT_DARK,
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
                    tab_label_background: Background::Color(if is_active {
                        *PRIMARY_5
                    } else {
                        *PRIMARY_6
                    }),
                    tab_label_border_color: Color::TRANSPARENT,
                    tab_label_border_width: 0.0,
                    icon_color: *TEXT_DARK,
                    text_color: *TEXT_DARK,
                }
            }

            fn hovered(&self, is_active: bool) -> tab_bar::Style {
                tab_bar::Style {
                    tab_label_background: Background::Color(*PRIMARY_4),
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
                    background: Some(Background::Color(*BACKGROUND_DARK)),
                    first_background: Some(Background::Color(*BACKGROUND_DARK)),
                    second_background: Some(Background::Color(*BACKGROUND_DARK)),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    divider_background: Background::Color(*BACKGROUND_DARK),
                    divider_border_width: 0.0,
                    divider_border_color: Color::TRANSPARENT,
                }
            }

            fn hovered(&self) -> split::Style {
                split::Style {
                    divider_background: Background::Color(*SECONDARY_6),
                    ..self.active()
                }
            }

            fn dragged(&self) -> split::Style {
                split::Style {
                    divider_background: Background::Color(*SECONDARY_4),
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
                    background: Some(Background::Color(*BACKGROUND_DARK)),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_5,
                    text_color: *TEXT_DARK,
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
                    background: Some(Background::Color(*BACKGROUND_DARK)),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_6,
                    text_color: *TEXT_DARK,
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
                    text_color: Some(*TEXT_DARK),
                    background: Some(Background::Color(*BACKGROUND_DARK)),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_6,
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
                    background: Some(Background::Color(*PRIMARY_5)),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_DARK,
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
                    background: Some(Background::Color(*PRIMARY_6)),
                    border_radius: 2.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: *TEXT_DARK,
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
                    text_color: *TEXT_DARK,
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
                    text_color: *TEXT_DARK,
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(*SECONDARY_6)),
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
                    background: Some(Background::Color(*BACKGROUND_DARK)),
                    border_radius: 2.0,
                    border_width: 1.0,
                    border_color: *SECONDARY_4,
                    text_color: *SECONDARY_4,
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(*SECONDARY_4)),
                    border_color: *SECONDARY_4,
                    text_color: *BACKGROUND_DARK,
                    ..self.active()
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn number_input(&self) -> Box<dyn number_input::StyleSheet> {
        struct Style;
        impl number_input::StyleSheet for Style {
            fn active(&self) -> number_input::Style {
                number_input::Style {
                    button_background: None,
                    icon_color: *TEXT_DARK,
                }
            }
        }
        Style.into()
    }

    #[allow(clippy::missing_docs_in_private_items)]
    fn slider(&self) -> Box<dyn slider::StyleSheet> {
        struct Style;
        impl slider::StyleSheet for Style {
            fn active(&self) -> slider::Style {
                slider::Style {
                    rail_colors: (
                        Color {
                            a: 0.5,
                            ..*SECONDARY_1
                        },
                        *BACKGROUND_DARK,
                    ),
                    handle: iced::slider::Handle {
                        shape: iced::slider::HandleShape::Rectangle {
                            width: 8,
                            border_radius: 4.0,
                        },
                        color: *SECONDARY_3,
                        border_color: *SECONDARY_2,
                        border_width: 1.0,
                    },
                }
            }

            fn hovered(&self) -> slider::Style {
                let active = self.active();
                slider::Style {
                    handle: iced::slider::Handle {
                        color: *SECONDARY_1,
                        ..active.handle
                    },
                    ..active
                }
            }

            fn dragging(&self) -> slider::Style {
                let active = self.active();
                slider::Style {
                    handle: iced::slider::Handle {
                        color: *SECONDARY_2,
                        ..active.handle
                    },
                    ..active
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
