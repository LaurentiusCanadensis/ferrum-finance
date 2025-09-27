# FerrumFinance

A comprehensive multi-currency financial management system built with Rust and Iced.

## Features

- **Multi-Currency Support**: Handle transactions in multiple currencies with real-time exchange rates
- **Loan Management**: Track loans, payment schedules, and interest calculations
- **Account Management**: Organize finances across different account types (Asset, Liability, Equity, Revenue, Expense)
- **Transaction Tracking**: Record and categorize all financial transactions
- **Professional GUI**: Modern interface built with Iced framework
- **Logo Integration**: Custom branding with integrated logo assets
- **Splash Screen**: Professional startup experience
- **Database Storage**: Persistent data storage with SQLite

## Architecture

- **Frontend**: Iced GUI framework with custom themes and layouts
- **Backend**: Rust with comprehensive business logic
- **Database**: SQLite for reliable data persistence
- **Models**: Comprehensive financial data structures
- **Services**: Transaction aggregation, reporting, and analytics

## Getting Started

### Prerequisites

- Rust 1.70+
- Cargo

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/ferrum-finance.git
   cd ferrum-finance
   ```

2. Build and run:
   ```bash
   cargo run
   ```

## Usage

The application launches with a splash screen showing the FerrumFinance logo for 3 seconds, then transitions to the main dashboard with navigation for:

- **Dashboard**: Overview of accounts, transactions, and loans
- **Accounts**: Manage financial accounts across multiple currencies
- **Transactions**: Record and track financial transactions
- **Loans**: Manage loan details, payment schedules, and calculations
- **Reports**: Financial analytics and reporting

## Development

This project was developed using an agent-based approach with comprehensive planning and implementation phases.

### Project Structure

```
ferrum-finance/
├── src/
│   ├── main.rs           # Main application entry
│   ├── lib.rs            # Library exports
│   ├── assets/           # Logo and image assets
│   ├── models/           # Data models
│   └── storage/          # Database layer
├── requirements/         # Project requirements
└── examples/            # Usage examples
```

## Technologies

- **Rust**: Systems programming language
- **Iced**: Cross-platform GUI framework
- **SQLite**: Embedded database
- **rust_decimal**: Precise decimal arithmetic
- **tokio**: Async runtime
- **serde**: Serialization framework

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

🦀 Built with Rust | 🎨 Powered by Iced | 💰 FerrumFinance# ferrum-finance
