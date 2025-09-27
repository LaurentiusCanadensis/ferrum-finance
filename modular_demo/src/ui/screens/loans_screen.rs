use iced::{Color, Element, Length};
use iced::widget::{button, column, row, text, Space};
use crate::ui::widgets::{LoanWidget, SummaryCard};
use crate::ui::widgets::common::action_button;
use crate::ui::{PRIMARY_COLOR, SECONDARY_COLOR, SUCCESS_COLOR, WARNING_COLOR, SPACING_LARGE, SPACING_MEDIUM};
use crate::{Message, Loan, LoanForm, LoanStatus, LoanType};

pub struct LoansScreen;

impl LoansScreen {
    pub fn view<'a>(
        loans: &[Loan],
        loan_form: &LoanForm,
        show_form: bool,
    ) -> Element<'a, Message> {
        column![
            Self::header(),
            Space::with_height(Length::Fixed(SPACING_MEDIUM as f32)),
            Self::dashboard_summary(loans),
            Space::with_height(Length::Fixed(SPACING_LARGE as f32)),
            if show_form {
                LoanWidget::form_view(loan_form)
            } else {
                Self::loans_list_with_actions(loans)
            }
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn header<'a>() -> Element<'a, Message> {
        row![
            column![
                text("Loan Management")
                    .size(32)
                    .style(PRIMARY_COLOR),
                text("Track your loans, calculate payments, and manage your debt portfolio")
                    .size(16)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
            ],
            Space::with_width(Length::Fill),
            action_button(
                "Add New Loan",
                Message::ShowLoanForm,
                SECONDARY_COLOR
            )
        ]
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn dashboard_summary<'a>(loans: &[Loan]) -> Element<'a, Message> {
        let total_loans = loans.len();
        let active_loans = loans.iter()
            .filter(|l| l.status == LoanStatus::Active)
            .count();
        let paid_loans = loans.iter()
            .filter(|l| l.status == LoanStatus::Paid)
            .count();

        // Calculate portfolio metrics
        let total_principal = loans.iter()
            .filter(|l| l.currency == crate::Currency::USD && l.status == LoanStatus::Active)
            .map(|l| l.principal_amount)
            .sum::<rust_decimal::Decimal>();

        let avg_interest_rate = if !loans.is_empty() {
            loans.iter()
                .filter(|l| l.status == LoanStatus::Active)
                .map(|l| l.interest_rate)
                .sum::<rust_decimal::Decimal>() / rust_decimal::Decimal::from(active_loans.max(1))
        } else {
            rust_decimal::Decimal::ZERO
        };

        column![
            text("Loan Portfolio Overview")
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),
            row![
                SummaryCard::new(
                    "Total Loans",
                    &total_loans.to_string(),
                    PRIMARY_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Active Loans",
                    &active_loans.to_string(),
                    WARNING_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Paid Off",
                    &paid_loans.to_string(),
                    SUCCESS_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Outstanding (USD)",
                    &format!("{:.2}", total_principal),
                    SECONDARY_COLOR
                ),
            ],
            Space::with_height(Length::Fixed(10.0)),
            row![
                SummaryCard::new(
                    "Avg Interest Rate",
                    &format!("{:.2}%", avg_interest_rate),
                    Color::from_rgb(0.8, 0.4, 0.2)
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Portfolio Health",
                    &Self::calculate_portfolio_health(loans),
                    if Self::calculate_portfolio_health_score(loans) > 7.0 {
                        SUCCESS_COLOR
                    } else if Self::calculate_portfolio_health_score(loans) > 5.0 {
                        WARNING_COLOR
                    } else {
                        Color::from_rgb(0.8, 0.2, 0.2)
                    }
                ),
                Space::with_width(Length::Fill),
                Space::with_width(Length::Fill),
            ]
        ]
        .spacing(10)
        .into()
    }

    fn loans_list_with_actions<'a>(loans: &[Loan]) -> Element<'a, Message> {
        if loans.is_empty() {
            return Self::empty_state();
        }

        column![
            row![
                text("Your Loans")
                    .size(20)
                    .style(PRIMARY_COLOR),
                Space::with_width(Length::Fill),
                row![
                    button(text("Payment Calculator"))
                        .on_press(Message::ShowPaymentCalculator)
                        .padding([8, 16])
                        .style(|_theme, _status| iced::widget::button::Appearance {
                            background: Some(iced::Background::Color(SECONDARY_COLOR)),
                            text_color: Color::WHITE,
                            border: iced::Border::with_radius(5),
                            ..iced::widget::button::Appearance::default()
                        }),
                    Space::with_width(Length::Fixed(10.0)),
                    button(text("Generate Report"))
                        .on_press(Message::GenerateLoanReport)
                        .padding([8, 16])
                        .style(|_theme, _status| iced::widget::button::Appearance {
                            background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                            text_color: Color::from_rgb(0.3, 0.3, 0.3),
                            border: iced::Border::with_radius(5),
                            ..iced::widget::button::Appearance::default()
                        })
                ]
                .spacing(10)
            ]
            .align_items(iced::Alignment::Center),
            Space::with_height(Length::Fixed(15.0)),
            LoanWidget::loan_list(loans),
        ]
        .spacing(10)
        .into()
    }

    fn empty_state<'a>() -> Element<'a, Message> {
        column![
            Space::with_height(Length::Fixed(40.0)),
            text("No Loans Yet")
                .size(24)
                .style(Color::from_rgb(0.6, 0.6, 0.6)),
            Space::with_height(Length::Fixed(15.0)),
            text("Add your first loan to start tracking payments and managing debt")
                .size(16)
                .style(Color::from_rgb(0.5, 0.5, 0.5)),
            Space::with_height(Length::Fixed(20.0)),
            action_button(
                "Add Your First Loan",
                Message::ShowLoanForm,
                SECONDARY_COLOR
            ),
            Space::with_height(Length::Fixed(40.0)),
        ]
        .align_items(iced::Alignment::Center)
        .spacing(10)
        .into()
    }

    pub fn loan_by_type_view<'a>(
        loans: &[Loan],
        filter_type: Option<LoanType>
    ) -> Element<'a, Message> {
        let filtered_loans: Vec<&Loan> = if let Some(l_type) = filter_type {
            loans.iter()
                .filter(|l| l.loan_type == l_type)
                .collect()
        } else {
            loans.iter().collect()
        };

        column![
            row![
                button(text("← Back to All Loans"))
                    .on_press(Message::NavigateTo(crate::Screen::Loans))
                    .style(|_theme, _status| iced::widget::button::Appearance {
                        background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                        text_color: PRIMARY_COLOR,
                        border: iced::Border::with_radius(5),
                        ..iced::widget::button::Appearance::default()
                    }),
                Space::with_width(Length::Fill),
            ],
            Space::with_height(Length::Fixed(20.0)),
            text(if let Some(l_type) = filter_type {
                format!("{:?} Loans", l_type)
            } else {
                "All Loans".to_string()
            })
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),
            Self::loan_type_summary(&filtered_loans, filter_type),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn loan_type_summary<'a>(
        loans: &[&Loan],
        loan_type: Option<LoanType>
    ) -> Element<'a, Message> {
        let total_principal = loans.iter()
            .filter(|l| l.currency == crate::Currency::USD) // USD only for simplicity
            .map(|l| l.principal_amount)
            .sum::<rust_decimal::Decimal>();

        let avg_interest = if !loans.is_empty() {
            loans.iter()
                .map(|l| l.interest_rate)
                .sum::<rust_decimal::Decimal>() / rust_decimal::Decimal::from(loans.len())
        } else {
            rust_decimal::Decimal::ZERO
        };

        let avg_term = if !loans.is_empty() {
            loans.iter()
                .map(|l| l.term_months as f64)
                .sum::<f64>() / loans.len() as f64
        } else {
            0.0
        };

        let type_color = match loan_type {
            Some(LoanType::Personal) => Color::from_rgb(0.2, 0.6, 0.8),
            Some(LoanType::Mortgage) => Color::from_rgb(0.2, 0.8, 0.2),
            Some(LoanType::Auto) => Color::from_rgb(0.8, 0.4, 0.2),
            Some(LoanType::Business) => Color::from_rgb(0.6, 0.2, 0.8),
            Some(LoanType::Student) => Color::from_rgb(0.8, 0.6, 0.2),
            None => SECONDARY_COLOR,
        };

        column![
            row![
                SummaryCard::new(
                    "Total Count",
                    &loans.len().to_string(),
                    type_color
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Total Principal (USD)",
                    &format!("{:.2}", total_principal),
                    type_color
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Avg Interest Rate",
                    &format!("{:.2}%", avg_interest),
                    type_color
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Avg Term (months)",
                    &format!("{:.0}", avg_term),
                    type_color
                ),
            ]
        ]
        .spacing(10)
        .into()
    }

    // Helper function to calculate portfolio health
    fn calculate_portfolio_health(loans: &[Loan]) -> String {
        let score = Self::calculate_portfolio_health_score(loans);
        match score as u8 {
            9..=10 => "Excellent".to_string(),
            7..=8 => "Good".to_string(),
            5..=6 => "Fair".to_string(),
            3..=4 => "Poor".to_string(),
            _ => "Critical".to_string(),
        }
    }

    fn calculate_portfolio_health_score(loans: &[Loan]) -> f64 {
        if loans.is_empty() {
            return 10.0; // No debt is excellent
        }

        let active_loans = loans.iter().filter(|l| l.status == LoanStatus::Active).count();
        let defaulted_loans = loans.iter().filter(|l| l.status == LoanStatus::Defaulted).count();
        let avg_interest = loans.iter()
            .filter(|l| l.status == LoanStatus::Active)
            .map(|l| l.interest_rate)
            .sum::<rust_decimal::Decimal>() / rust_decimal::Decimal::from(active_loans.max(1));

        // Simple scoring algorithm (real-world would be more complex)
        let mut score = 10.0;

        // Penalize for high number of active loans
        if active_loans > 5 {
            score -= 2.0;
        } else if active_loans > 3 {
            score -= 1.0;
        }

        // Penalize for defaulted loans
        score -= (defaulted_loans as f64) * 3.0;

        // Penalize for high average interest rates
        let avg_interest_f64 = avg_interest.to_string().parse::<f64>().unwrap_or(0.0);
        if avg_interest_f64 > 15.0 {
            score -= 2.0;
        } else if avg_interest_f64 > 10.0 {
            score -= 1.0;
        }

        score.max(0.0).min(10.0)
    }
}