# Domain Logic

This crate contains the domain logic for the Toka project, including pricing models, fee structures, and economic rules. It is intentionally separate from the ledger crate to maintain clean separation of concerns.

## Architecture Philosophy

### Domain Crate Responsibilities
- **Pricing Policies**: Defines how credit packages are priced and made available (e.g., `PricingPolicy` trait, `DefaultPricingPolicy`).
- **Pricing Service**: Provides a clean API (`PricingService`) for accessing credit package information.
- **Detailed Pricing Configuration**: Holds the underlying data for pricing (e.g., `PlatformPricingConfig`, `CreditPackage`, `FeeSchedule`). This includes:
    - Credit package definitions and pricing.
    - Fee structures for various actions (content unlocks, cashouts).
    - Configuration for dynamic take rates (`TakeRateSlidingWindowConfig`).
- **Fee Calculations**: Platform fees, creator tier-based fees, cashout fees. Utilizes `get_effective_platform_take_rate()` for scenarios involving the platform's general take rate.
- **Business Rules**: Creator Empowerment Fund allocation, fairness multipliers.
- **Payout Settings**: KYC thresholds, supported payout types.
- **Economic Policy**: Platform take rates, discount structures.

### Ledger Crate Responsibilities  
- **Pure Accounting**: Double-entry bookkeeping, debits/credits
- **Transaction Integrity**: Atomic operations, balance validation
- **Storage Abstraction**: Persistence, Write-Ahead Logging
- **Account Management**: Balance tracking, account creation

## Key Benefits of This Separation

1. **Single Responsibility**: Each crate has a clear, focused purpose
2. **Extensible Pricing**: The `PricingPolicy` trait allows for different pricing strategies (dynamic, A/B testing, regional) to be easily implemented and swapped without altering core services.
3. **Independent Evolution**: Business rules and pricing strategies can change without affecting accounting logic
4. **Easier Testing**: Domain logic (including different pricing policies) and accounting can be tested separately
5. **Better Maintainability**: Changes to pricing don't risk ledger integrity
6. **Cleaner APIs**: Each crate exposes only what it's responsible for. The `PricingService` offers a well-defined interface for UI components or other services to fetch pricing information.

## Usage Example

See the `content_unlock_flow` example for a complete demonstration:

```bash
cargo run --example content_unlock_flow -p domain
```

This example shows how:
- The `PricingService` (from the domain crate) is used to fetch credit package information.
- Detailed `PlatformPricingConfig` (from the domain crate) is used for internal fee calculations.
- The `Ledger` (from the ledger crate) handles the actual accounting transactions.
- All components work together while maintaining separation.

## Key Types

### Extensible Pricing API (Domain Crate)
```rust
use domain::pricing_model::{DefaultPricingPolicy, PricingService, PlatformPricingConfig};
// ... (assuming pricing_config_data is loaded or created) ...
let pricing_config_data: PlatformPricingConfig = /* ... */;

// Create a policy (e.g., the default one using PlatformPricingConfig)
let pricing_policy = DefaultPricingPolicy::new(pricing_config_data.clone());

// Create the service with the chosen policy
let pricing_service = PricingService::new(pricing_policy);

// Use the service to get available packages
let packages = pricing_service.get_available_packages();
if let Some(starter_pack) = pricing_service.get_package_by_tier("Starter") {
    // Use starter_pack details
}
```

### Detailed Fee Calculation (Domain Crate)
```rust
use domain::PlatformPricingConfig;
// ... (assuming pricing_config_data is loaded or created) ...
let pricing_config_data: PlatformPricingConfig = /* ... */;
let creator_tier = /* ... */;
let unlock_amount = /* ... */;

// Business logic calculates amounts using the detailed config
// Specific fees like content unlock might have their own percentage in FeeSchedule
let platform_fee_for_unlock = pricing_config_data.calculate_content_unlock_fee(&creator_tier, unlock_amount).unwrap_or(0);

// For general platform take rate scenarios:
let effective_take_rate = pricing_config_data.get_effective_platform_take_rate();
// ... use effective_take_rate in other calculations ...
```

### Ledger Integration
```rust
use ledger::{Ledger, MemoryStorage};

let mut ledger = Ledger::new();
let mut storage = MemoryStorage::new();

// Business logic determines what to do, ledger executes the accounting
{
    let mut tx = ledger.begin_transaction(&mut storage);
    tx.transfer("user", "creator", amount, "Content unlock".to_string(), None)?;
    tx.commit()?;
}
```

## Design Patterns

### Dependency Direction
```
┌─────────────┐    ┌─────────────┐
│   Domain    │───▶│   Ledger    │
│  (business &│    │ (mechanics) │
│   pricing)  │    └─────────────┘
└─────────────┘
```

The domain crate depends on the ledger crate for executing transactions, but the ledger crate has no knowledge of business rules or pricing policies. This keeps the ledger focused purely on accounting mechanics.

### Flow Example (Content Unlock)
1.  **UI/Application Layer**: User wants to purchase credits.
2.  **Domain Service (`PricingService`)**: Lists available `CreditPackageView`s. User selects a package.
3.  **Application Layer**: Initiates credit purchase.
4.  **Ledger Execution**: `mint` new credits to the user's account.
5.  **UI/Application Layer**: User wants to unlock content for 50 credits. Creator is identified as "Micro" tier.
6.  **Domain Logic (`PlatformPricingConfig`)**: Calculates platform fee (e.g., "Platform fee is 20% of 50 = 10 credits, creator gets 40 credits"). This might use specific fee schedules or the `get_effective_platform_take_rate()` for more general scenarios.
7.  **Ledger Execution**: Executes the transfers atomically (user to creator, user to platform revenue).
8.  **Domain Logic (`PlatformPricingConfig`)**: Calculates empowerment fund allocation (e.g., "Allocate 25% of platform fee (2.5 credits -> 2 credits) to empowerment fund").
9.  **Ledger Execution**: Executes empowerment fund transfer from platform revenue.

## Testing Strategy

### Domain Logic Tests
- `PricingPolicy` implementations (e.g., `DefaultPricingPolicy`) for package listing and retrieval.
- `PricingService` behavior with different policies.
- Fee calculation accuracy using `PlatformPricingConfig`.
- Business rule validation.
- Edge cases in economic logic.

### Integration Tests  
- Business + Ledger working together
- End-to-end transaction flows
- Multi-step business processes

### Ledger Tests (in ledger crate)
- Accounting integrity
- Transaction atomicity  
- Balance validation
- Storage persistence

This separation makes the codebase much more maintainable and allows each domain to evolve independently while working together seamlessly. The `PricingPolicy` trait, in particular, offers a powerful way to adapt and extend pricing strategies over time. 