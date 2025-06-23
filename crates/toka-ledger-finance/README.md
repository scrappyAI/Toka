# Toka Ledger Finance

Financial ledger operations and calculations for the Toka platform.

## Overview

This crate provides financial ledger operations including payment processing, fee calculations, and financial reporting. It extends the core ledger functionality with financial transaction types and business logic.

## Features

- Financial transaction types (payments, fees, credits)
- Fee calculation engines
- Payment processing workflows
- Financial reporting and analytics
- Currency conversion support
- Audit trail for financial operations

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-ledger-finance = "0.1.0"
```

### Example

```rust
use toka_ledger_finance::{FinancialLedger, Payment, FeeCalculator};
use toka_primitives::MicroUSD;

let ledger = FinancialLedger::new();
let calculator = FeeCalculator::new();

// Process a payment
let payment = Payment::new(
    from_agent,
    to_agent,
    MicroUSD::from_dollars(10.50),
    "service_payment"
);

ledger.process_payment(payment).await?;

// Calculate fees
let fees = calculator.calculate_fees(
    transaction_amount,
    fee_schedule
)?;

// Generate financial report
let report = ledger.generate_report(
    start_date,
    end_date
).await?;
```

## Integration

This crate integrates with:
- `toka-ledger-core` for core ledger functionality
- `toka-primitives` for currency types
- `toka-core` for domain business rules

## Design Philosophy

- **Financial Accuracy**: Precise calculations and audit trails
- **Compliance**: Support for financial reporting requirements
- **Flexibility**: Configurable fee schedules and payment workflows
- **Security**: Secure handling of financial transactions

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option. 