//! Configurable economy engine, wallet tracking, and financial transactions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors arising from financial economy transactions.
#[derive(Debug, Error, PartialEq)]
pub enum EconomyError {
    #[error(
        "Insufficient funds: current balance is {balance:.2}, but required amount is {required:.2}"
    )]
    InsufficientFunds { balance: f64, required: f64 },
    #[error("Invalid transaction amount: {0:.2}")]
    InvalidAmount(f64),
}

/// Category classification for financial transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionCategory {
    JobPayout,
    CustomerTip,
    SpeedBonus,
    CollisionRepair,
    VehiclePurchase,
    VehicleUpgrade,
    RoadsideRecovery,
    ExplorationReward,
}

/// Record of an individual financial transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub timestamp_secs: u64,
    pub amount: f64,
    pub category: TransactionCategory,
    pub description: String,
    pub balance_after: f64,
}

/// Global economic rate and price configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomyConfig {
    /// Active currency symbol (e.g. "₹" or "$").
    pub currency_symbol: String,
    /// Base fare for urban taxi trips.
    pub base_taxi_fare: f64,
    /// Per-kilometer compensation for taxi passengers.
    pub taxi_per_km: f64,
    /// Maximum tip multiplier for 5-star passenger ride comfort (e.g. 0.35 = up to 35% tip).
    pub max_taxi_comfort_tip_rate: f64,
    /// Base fee for parcel courier dispatch.
    pub base_delivery_fee: f64,
    /// Per-kilometer compensation for parcel deliveries.
    pub delivery_per_km: f64,
    /// Fragile cargo payout multiplier.
    pub fragile_cargo_rate: f64,
    /// Base contract rate for heavy freight haulage.
    pub base_freight_rate: f64,
    /// Per-kilometer compensation for long-distance commercial freight.
    pub freight_per_km: f64,
    /// Fixed cost for emergency roadside recovery towing.
    pub recovery_cost: f64,
    /// Cost per impulse unit of collision vehicle damage.
    pub collision_damage_rate: f64,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            currency_symbol: "₹".to_string(),
            base_taxi_fare: 50.0,
            taxi_per_km: 22.0,
            max_taxi_comfort_tip_rate: 0.35,
            base_delivery_fee: 80.0,
            delivery_per_km: 18.0,
            fragile_cargo_rate: 1.5,
            base_freight_rate: 350.0,
            freight_per_km: 45.0,
            recovery_cost: 250.0,
            collision_damage_rate: 15.0,
        }
    }
}

/// In-game player financial wallet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wallet {
    balance: f64,
    transactions: Vec<Transaction>,
}

impl Default for Wallet {
    fn default() -> Self {
        Self::with_balance(500.0) // Starter capital
    }
}

impl Wallet {
    /// Creates a new wallet initialized with a specified balance.
    pub fn with_balance(initial_balance: f64) -> Self {
        Self {
            balance: initial_balance.max(0.0),
            transactions: Vec::new(),
        }
    }

    /// Current available liquid funds.
    pub fn balance(&self) -> f64 {
        self.balance
    }

    /// Checks if the wallet has sufficient funds to cover an expense.
    pub fn can_afford(&self, amount: f64) -> bool {
        amount >= 0.0 && self.balance >= amount
    }

    /// Credits an incoming amount into the wallet.
    pub fn credit(
        &mut self,
        amount: f64,
        category: TransactionCategory,
        description: impl Into<String>,
        timestamp_secs: u64,
    ) -> Result<f64, EconomyError> {
        if amount < 0.0 {
            return Err(EconomyError::InvalidAmount(amount));
        }
        self.balance += amount;
        self.transactions.push(Transaction {
            timestamp_secs,
            amount,
            category,
            description: description.into(),
            balance_after: self.balance,
        });
        Ok(self.balance)
    }

    /// Debits an outgoing expense from the wallet.
    pub fn debit(
        &mut self,
        amount: f64,
        category: TransactionCategory,
        description: impl Into<String>,
        timestamp_secs: u64,
    ) -> Result<f64, EconomyError> {
        if amount < 0.0 {
            return Err(EconomyError::InvalidAmount(amount));
        }
        if self.balance < amount {
            return Err(EconomyError::InsufficientFunds {
                balance: self.balance,
                required: amount,
            });
        }
        self.balance -= amount;
        self.transactions.push(Transaction {
            timestamp_secs,
            amount: -amount,
            category,
            description: description.into(),
            balance_after: self.balance,
        });
        Ok(self.balance)
    }

    /// Returns a slice of recent recorded transactions.
    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_credit_debit_flow() {
        let mut wallet = Wallet::with_balance(100.0);
        assert_eq!(wallet.balance(), 100.0);

        let bal1 = wallet
            .credit(250.0, TransactionCategory::JobPayout, "Taxi fare", 100)
            .unwrap();
        assert_eq!(bal1, 350.0);
        assert_eq!(wallet.balance(), 350.0);

        let bal2 = wallet
            .debit(
                50.0,
                TransactionCategory::CollisionRepair,
                "Bumper repair",
                110,
            )
            .unwrap();
        assert_eq!(bal2, 300.0);

        // Attempting to overdraw
        let err = wallet.debit(
            500.0,
            TransactionCategory::VehiclePurchase,
            "Sports car",
            120,
        );
        assert!(matches!(err, Err(EconomyError::InsufficientFunds { .. })));
        assert_eq!(wallet.balance(), 300.0);
        assert_eq!(wallet.transactions().len(), 2);
    }
}
