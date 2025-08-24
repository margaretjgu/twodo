use async_trait::async_trait;
use uuid::Uuid;
use super::expense::{Expense, ExpenseShare, ExpenseInfo, UserBalance, GroupBalance, DebtSummary, Payment, ExpenseFilter};
use std::error::Error;

#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    async fn create_expense(&self, expense: &Expense) -> Result<(), Box<dyn Error>>;
    async fn get_expense_by_id(&self, expense_id: &Uuid) -> Result<Option<Expense>, Box<dyn Error>>;
    async fn update_expense(&self, expense: &Expense) -> Result<(), Box<dyn Error>>;
    async fn delete_expense(&self, expense_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn get_expenses(&self, filter: &ExpenseFilter) -> Result<Vec<ExpenseInfo>, Box<dyn Error>>;
    async fn get_group_expenses(&self, group_id: &Uuid, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<ExpenseInfo>, Box<dyn Error>>;
}

#[async_trait]
pub trait ExpenseShareRepository: Send + Sync {
    async fn create_shares(&self, shares: &[ExpenseShare]) -> Result<(), Box<dyn Error>>;
    async fn get_expense_shares(&self, expense_id: &Uuid) -> Result<Vec<ExpenseShare>, Box<dyn Error>>;
    async fn update_share(&self, share: &ExpenseShare) -> Result<(), Box<dyn Error>>;
    async fn delete_expense_shares(&self, expense_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn get_user_shares(&self, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<Vec<ExpenseShare>, Box<dyn Error>>;
}

#[async_trait]
pub trait BalanceRepository: Send + Sync {
    async fn calculate_group_balances(&self, group_id: &Uuid) -> Result<GroupBalance, Box<dyn Error>>;
    async fn calculate_user_balance(&self, user_id: &Uuid, group_id: &Uuid) -> Result<f64, Box<dyn Error>>;
    async fn get_debt_summary(&self, group_id: &Uuid) -> Result<Vec<DebtSummary>, Box<dyn Error>>;
    async fn get_user_debts(&self, user_id: &Uuid) -> Result<Vec<DebtSummary>, Box<dyn Error>>;
}

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn create_payment(&self, payment: &Payment) -> Result<(), Box<dyn Error>>;
    async fn get_group_payments(&self, group_id: &Uuid) -> Result<Vec<Payment>, Box<dyn Error>>;
    async fn get_user_payments(&self, user_id: &Uuid) -> Result<Vec<Payment>, Box<dyn Error>>;
}