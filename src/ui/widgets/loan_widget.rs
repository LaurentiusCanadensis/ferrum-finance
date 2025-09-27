use iced::{Color, Element, Length};
use iced::widget::{column, pick_list, text, text_input, Space};
use crate::ui::widgets::common::{form_section, action_button};
use crate::ui::{SECONDARY_COLOR, ERROR_COLOR, SPACING_SMALL, LIGHT_TEXT_COLOR};
use crate::{Message, Loan, LoanForm, LoanType, LoanStatus};

pub struct LoanWidget;

impl LoanWidget {
    pub fn form_view<'a>(loan_form: &LoanForm) -> Element<'a, Message> {
        let mut form_content = column![];

        // Loan type selection
        form_content = form_content.push(
            column![
                text("Loan Type").size(14),
                pick_list(
                    vec![LoanType::Personal, LoanType::Mortgage, LoanType::Auto,
                         LoanType::Business, LoanType::Student],
                    loan_form.loan_type,
                    Message::LoanTypeSelected,
                ).padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Principal amount input
        form_content = form_content.push(
            column![
                text("Principal Amount").size(14),
                text_input("0.00", &loan_form.principal)
                    .on_input(Message::LoanPrincipalChanged)
                    .padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Interest rate input
        form_content = form_content.push(
            column![
                text("Interest Rate (% APR)").size(14),
                text_input("0.00", &loan_form.interest_rate)
                    .on_input(Message::LoanInterestRateChanged)
                    .padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Term input
        form_content = form_content.push(
            column![
                text("Term (months)").size(14),
                text_input("12", &loan_form.term_months)
                    .on_input(Message::LoanTermChanged)
                    .padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Error messages
        if !loan_form.errors.is_empty() {
            for error in &loan_form.errors {
                form_content = form_content.push(
                    text(error).size(12).style(ERROR_COLOR)
                );
            }
            form_content = form_content.push(Space::with_height(Length::Fixed(10.0)));
        }

        // Submit button
        form_content = form_content.push(
            action_button("Create Loan", Message::CreateLoan, SECONDARY_COLOR)
        );

        form_section("Create New Loan", form_content.spacing(SPACING_SMALL).into())
    }

    pub fn loan_list<'a>(loans: &[Loan]) -> Element<'a, Message> {
        if loans.is_empty() {
            return column![
                text("No loans created yet").size(16),
                text("Use the form above to create your first loan").size(14)
                    .style(LIGHT_TEXT_COLOR),
            ].spacing(10).into();
        }

        let mut list = column![
            text("Your Loans").size(20),
            Space::with_height(Length::Fixed(10.0)),
        ];

        for loan in loans {
            let loan_item = Self::loan_item(loan);
            list = list.push(loan_item);
            list = list.push(Space::with_height(Length::Fixed(10.0)));
        }

        list.spacing(5).into()
    }

    fn loan_item<'a>(loan: &Loan) -> Element<'a, Message> {
        let status_color = match loan.status {
            LoanStatus::Active => Color::from_rgb(0.2, 0.6, 0.2),
            LoanStatus::Paid => Color::from_rgb(0.6, 0.6, 0.6),
            LoanStatus::Defaulted => Color::from_rgb(0.8, 0.2, 0.2),
            LoanStatus::Suspended => Color::from_rgb(0.8, 0.6, 0.2),
        };

        // Calculate monthly payment (simplified)
        let monthly_payment = Self::calculate_monthly_payment(
            loan.principal_amount,
            loan.interest_rate,
            loan.term_months
        );

        let loan_info = column![
            text(format!("{:?} Loan", loan.loan_type)).size(16),
            text(format!("{:.2}% APR • {} months", loan.interest_rate, loan.term_months))
                .size(12)
                .style(LIGHT_TEXT_COLOR),
            text(format!("Monthly: {:.2} {}", monthly_payment, loan.currency))
                .size(12)
                .style(Color::from_rgb(0.2, 0.4, 0.8)),
            text(format!("{:?}", loan.status))
                .size(12)
                .style(status_color),
        ].spacing(2);

        let principal_info = column![
            text(format!("{:.2}", loan.principal_amount))
                .size(16),
            text(loan.currency.to_string())
                .size(12)
                .style(SECONDARY_COLOR),
        ].spacing(2);

        iced::widget::container(
            iced::widget::row![
                loan_info,
                Space::with_width(Length::Fill),
                principal_info,
            ]
            .padding(15)
            .align_items(iced::Alignment::Center)
        )
        .padding(5)
        .style(|_theme| iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
            border: iced::Border::with_radius(5),
            ..iced::widget::container::Appearance::default()
        })
        .into()
    }

    pub fn loan_summary<'a>(loans: &[Loan]) -> Element<'a, Message> {
        let total_loans = loans.len();
        let active_loans = loans.iter()
            .filter(|l| l.status == LoanStatus::Active)
            .count();
        let total_principal = loans.iter()
            .filter(|l| l.currency == crate::Currency::USD) // For simplicity, show only USD
            .map(|l| l.principal_amount)
            .sum::<rust_decimal::Decimal>();

        let avg_interest_rate = if !loans.is_empty() {
            loans.iter()
                .map(|l| l.interest_rate)
                .sum::<rust_decimal::Decimal>() / rust_decimal::Decimal::from(loans.len())
        } else {
            rust_decimal::Decimal::ZERO
        };

        column![
            text("Loan Portfolio Summary").size(18),
            Space::with_height(Length::Fixed(10.0)),
            text(format!("Total Loans: {}", total_loans)).size(14),
            text(format!("Active Loans: {}", active_loans)).size(14),
            text(format!("Total Principal (USD): {:.2}", total_principal)).size(14),
            text(format!("Average Interest Rate: {:.2}%", avg_interest_rate)).size(14),
        ]
        .spacing(5)
        .into()
    }

    // Helper function to calculate monthly payment using standard loan formula
    fn calculate_monthly_payment(
        principal: rust_decimal::Decimal,
        annual_rate: rust_decimal::Decimal,
        term_months: u32,
    ) -> rust_decimal::Decimal {
        if annual_rate == rust_decimal::Decimal::ZERO || term_months == 0 {
            return principal / rust_decimal::Decimal::from(term_months.max(1));
        }

        let monthly_rate = annual_rate / rust_decimal::Decimal::from(100) / rust_decimal::Decimal::from(12);
        let num_payments = rust_decimal::Decimal::from(term_months);

        // Simplified calculation - in a real app, you'd use proper compound interest formula
        let interest_factor = rust_decimal::Decimal::ONE + (monthly_rate * num_payments / rust_decimal::Decimal::from(2));
        principal * interest_factor / num_payments
    }

    pub fn loan_details<'a>(loan: &Loan) -> Element<'a, Message> {
        let monthly_payment = Self::calculate_monthly_payment(
            loan.principal_amount,
            loan.interest_rate,
            loan.term_months
        );

        let total_payment = monthly_payment * rust_decimal::Decimal::from(loan.term_months);
        let total_interest = total_payment - loan.principal_amount;

        column![
            text(format!("{:?} Loan Details", loan.loan_type)).size(18),
            Space::with_height(Length::Fixed(10.0)),
            text(format!("Principal: {:.2} {}", loan.principal_amount, loan.currency)).size(14),
            text(format!("Interest Rate: {:.2}% APR", loan.interest_rate)).size(14),
            text(format!("Term: {} months", loan.term_months)).size(14),
            Space::with_height(Length::Fixed(5.0)),
            text(format!("Monthly Payment: {:.2} {}", monthly_payment, loan.currency)).size(14)
                .style(Color::from_rgb(0.2, 0.4, 0.8)),
            text(format!("Total Interest: {:.2} {}", total_interest, loan.currency)).size(14)
                .style(Color::from_rgb(0.8, 0.4, 0.2)),
            text(format!("Total Payment: {:.2} {}", total_payment, loan.currency)).size(14),
            Space::with_height(Length::Fixed(5.0)),
            text(format!("Status: {:?}", loan.status)).size(14),
            text(format!("Created: {}", loan.created_at.format("%Y-%m-%d"))).size(12)
                .style(LIGHT_TEXT_COLOR),
        ]
        .spacing(5)
        .into()
    }
}