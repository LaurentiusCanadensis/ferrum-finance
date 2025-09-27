use iced::{Alignment, Color, Element, Length};
use iced::widget::{button, column, container, row, text, Image, Space};
use crate::ui::{PRIMARY_COLOR, BACKGROUND_COLOR, TEXT_COLOR, LIGHT_TEXT_COLOR};
use crate::messages::Message;
use crate::types::Screen;

pub struct SummaryCard;

impl SummaryCard {
    pub fn new<'a>(title: &str, value: &str, color: Color) -> Element<'a, Message> {
        container(
            column![
                text(title).size(14).style(LIGHT_TEXT_COLOR),
                text(value).size(24).style(color),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        )
        .padding(20)
        .style(|_theme| container::Appearance {
            background: Some(iced::Background::Color(BACKGROUND_COLOR)),
            border: iced::Border::with_radius(8),
            ..container::Appearance::default()
        })
        .into()
    }

    pub fn clickable<'a>(
        title: &str,
        value: &str,
        color: Color,
        message: Message
    ) -> Element<'a, Message> {
        button(
            column![
                text(title).size(14).style(LIGHT_TEXT_COLOR),
                text(value).size(24).style(color),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        )
        .on_press(message)
        .padding(20)
        .style(|_theme, _status| button::Appearance {
            background: Some(iced::Background::Color(BACKGROUND_COLOR)),
            text_color: TEXT_COLOR,
            border: iced::Border::with_radius(8),
            ..button::Appearance::default()
        })
        .into()
    }
}

pub struct NavigationBar;

impl NavigationBar {
    pub fn new<'a>(current_screen: &Screen) -> iced::widget::Row<'a, Message> {
        let nav_button = |label: &str, screen: Screen, current: &Screen| {
            let is_active = current == &screen;
            let button_text = if is_active {
                format!("• {}", label)
            } else {
                label.to_string()
            };

            button(text(button_text))
                .on_press(Message::NavigateTo(screen))
                .padding([10, 20])
                .style(if is_active {
                    |_theme, _status| button::Appearance {
                        background: Some(iced::Background::Color(PRIMARY_COLOR)),
                        text_color: Color::WHITE,
                        border: iced::Border::with_radius(5),
                        ..button::Appearance::default()
                    }
                } else {
                    |_theme, _status| button::Appearance {
                        background: Some(iced::Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                        text_color: TEXT_COLOR,
                        border: iced::Border::with_radius(5),
                        ..button::Appearance::default()
                    }
                })
        };

        row![
            nav_button("Dashboard", Screen::Dashboard, current_screen),
            nav_button("Accounts", Screen::Accounts, current_screen),
            nav_button("Transactions", Screen::Transactions, current_screen),
            nav_button("Loans", Screen::Loans, current_screen),
            nav_button("Reports", Screen::Reports, current_screen),
        ]
        .align_items(Alignment::Center)
        .spacing(10)
        .padding([0, 20, 20, 20])
    }
}

pub struct HeaderView;

impl HeaderView {
    pub fn new<'a>() -> iced::widget::Row<'a, Message> {
        let logo = Image::new(crate::ui::get_app_logo())
            .width(Length::Fixed(50.0))
            .height(Length::Fixed(50.0));

        row![
            logo,
            Space::with_width(Length::Fixed(15.0)),
            column![
                text("FerrumFinance")
                    .size(28)
                    .style(PRIMARY_COLOR),
                text("Multi-Currency Financial Management")
                    .size(14)
                    .style(LIGHT_TEXT_COLOR),
            ]
        ]
        .align_items(Alignment::Center)
        .padding(20)
    }
}

// Utility function for creating action buttons
pub fn action_button<'a>(
    label: &str,
    message: Message,
    color: Color,
) -> Element<'a, Message> {
    button(text(label))
        .on_press(message)
        .padding([12, 24])
        .style(move |_theme, _status| button::Appearance {
            background: Some(iced::Background::Color(color)),
            text_color: Color::WHITE,
            border: iced::Border::with_radius(5),
            ..button::Appearance::default()
        })
        .into()
}

// Utility function for creating form sections
pub fn form_section<'a>(title: &str, content: Element<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            text(title).size(20).style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),
            content,
        ]
        .spacing(10)
    )
    .padding(20)
    .style(|_theme| container::Appearance {
        background: Some(iced::Background::Color(BACKGROUND_COLOR)),
        border: iced::Border::with_radius(8),
        ..container::Appearance::default()
    })
    .into()
}