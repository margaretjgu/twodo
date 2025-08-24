use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Expense {
    pub id: Uuid,
    pub group_id: Uuid,
    pub description: String,
    pub amount: f64, // Total amount in cents to avoid floating point issues in production
    pub currency: String,
    pub paid_by: Uuid, // User who paid the expense
    pub created_by: Uuid, // User who created the expense entry
    pub category: Option<String>,
    pub date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpenseShare {
    pub expense_id: Uuid,
    pub user_id: Uuid,
    pub amount: f64, // Amount this user owes for this expense
    pub is_settled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SplitType {
    Equal, // Split equally among participants
    Exact(HashMap<Uuid, f64>), // Exact amounts per person
    Percentage(HashMap<Uuid, f64>), // Percentage per person (must sum to 100)
    ByShares(HashMap<Uuid, u32>), // Split by shares (e.g., 2 shares for Alice, 1 share for Bob)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpenseCreation {
    pub group_id: Uuid,
    pub description: String,
    pub amount: f64,
    pub currency: String,
    pub paid_by: Uuid,
    pub split_type: SplitType,
    pub participants: Vec<Uuid>, // Users involved in the expense
    pub category: Option<String>,
    pub date: Option<DateTime<Utc>>, // Optional, defaults to now
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpenseInfo {
    pub id: Uuid,
    pub group_id: Uuid,
    pub description: String,
    pub amount: f64,
    pub currency: String,
    pub paid_by: Uuid,
    pub paid_by_name: String,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub category: Option<String>,
    pub date: DateTime<Utc>,
    pub shares: Vec<ExpenseShareInfo>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpenseShareInfo {
    pub user_id: Uuid,
    pub username: String,
    pub amount: f64,
    pub is_settled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBalance {
    pub user_id: Uuid,
    pub username: String,
    pub net_balance: f64, // Positive = owed money, Negative = owes money
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupBalance {
    pub group_id: Uuid,
    pub group_name: String,
    pub balances: Vec<UserBalance>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebtSummary {
    pub creditor_id: Uuid,
    pub creditor_name: String,
    pub debtor_id: Uuid,
    pub debtor_name: String,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettleDebt {
    pub creditor_id: Uuid,
    pub debtor_id: Uuid,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payment {
    pub id: Uuid,
    pub group_id: Uuid,
    pub from_user: Uuid,
    pub to_user: Uuid,
    pub amount: f64,
    pub currency: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpenseFilter {
    pub group_id: Option<Uuid>,
    pub paid_by: Option<Uuid>,
    pub involving_user: Option<Uuid>,
    pub category: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}