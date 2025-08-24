use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

use crate::expenses::domain::expense::{
    Expense, ExpenseShare, ExpenseCreation, ExpenseInfo, SplitType, 
    UserBalance, GroupBalance, DebtSummary, SettleDebt, Payment, ExpenseFilter
};
use crate::expenses::domain::ports::{
    ExpenseRepository, ExpenseShareRepository, BalanceRepository, PaymentRepository
};
use std::error::Error;

pub struct ExpenseService {
    expense_repository: Arc<dyn ExpenseRepository>,
    share_repository: Arc<dyn ExpenseShareRepository>,
    balance_repository: Arc<dyn BalanceRepository>,
    payment_repository: Arc<dyn PaymentRepository>,
}

impl ExpenseService {
    pub fn new(
        expense_repository: Arc<dyn ExpenseRepository>,
        share_repository: Arc<dyn ExpenseShareRepository>,
        balance_repository: Arc<dyn BalanceRepository>,
        payment_repository: Arc<dyn PaymentRepository>,
    ) -> Self {
        Self {
            expense_repository,
            share_repository,
            balance_repository,
            payment_repository,
        }
    }

    pub async fn create_expense(&self, creation: ExpenseCreation, created_by: Uuid) -> Result<ExpenseInfo, Box<dyn Error>> {
        // Validate input
        if creation.description.trim().is_empty() {
            return Err("Expense description cannot be empty".into());
        }
        if creation.amount <= 0.0 {
            return Err("Expense amount must be positive".into());
        }
        if creation.participants.is_empty() {
            return Err("Expense must have at least one participant".into());
        }
        if !creation.participants.contains(&creation.paid_by) {
            return Err("The person who paid must be included in participants".into());
        }

        let now = Utc::now();
        let expense_id = Uuid::new_v4();

        // Create expense
        let expense = Expense {
            id: expense_id,
            group_id: creation.group_id,
            description: creation.description.trim().to_string(),
            amount: creation.amount,
            currency: creation.currency.clone(),
            paid_by: creation.paid_by,
            created_by,
            category: creation.category.clone(),
            date: creation.date.unwrap_or(now),
            created_at: now,
            updated_at: now,
        };

        self.expense_repository.create_expense(&expense).await?;

        // Calculate shares based on split type
        let mut shares = self.calculate_shares(&creation)?;
        
        // Fix expense_id for all shares (critical bug fix)
        for share in &mut shares {
            share.expense_id = expense_id;
        }
        
        self.share_repository.create_shares(&shares).await?;

        // Return expense info
        self.get_expense(&expense_id, &created_by).await?.ok_or("Failed to retrieve created expense".into())
    }

    fn calculate_shares(&self, creation: &ExpenseCreation) -> Result<Vec<ExpenseShare>, Box<dyn Error>> {
        let mut shares = Vec::new();

        match &creation.split_type {
            SplitType::Equal => {
                let share_amount = creation.amount / creation.participants.len() as f64;
                for user_id in &creation.participants {
                    shares.push(ExpenseShare {
                        expense_id: creation.group_id, // Will be set to expense_id by caller
                        user_id: *user_id,
                        amount: share_amount,
                        is_settled: false,
                    });
                }
            },
            SplitType::Exact(amounts) => {
                let total: f64 = amounts.values().sum();
                if (total - creation.amount).abs() > 0.01 {
                    return Err("Exact amounts must sum to total expense amount".into());
                }
                for user_id in &creation.participants {
                    if let Some(&amount) = amounts.get(user_id) {
                        shares.push(ExpenseShare {
                            expense_id: creation.group_id,
                            user_id: *user_id,
                            amount,
                            is_settled: false,
                        });
                    } else {
                        return Err("All participants must have exact amounts specified".into());
                    }
                }
            },
            SplitType::Percentage(percentages) => {
                let total_percent: f64 = percentages.values().sum();
                if (total_percent - 100.0).abs() > 0.01 {
                    return Err("Percentages must sum to 100%".into());
                }
                for user_id in &creation.participants {
                    if let Some(&percent) = percentages.get(user_id) {
                        let amount = creation.amount * (percent / 100.0);
                        shares.push(ExpenseShare {
                            expense_id: creation.group_id,
                            user_id: *user_id,
                            amount,
                            is_settled: false,
                        });
                    } else {
                        return Err("All participants must have percentages specified".into());
                    }
                }
            },
            SplitType::ByShares(share_counts) => {
                let total_shares: u32 = share_counts.values().sum();
                if total_shares == 0 {
                    return Err("Total shares cannot be zero".into());
                }
                for user_id in &creation.participants {
                    if let Some(&user_shares) = share_counts.get(user_id) {
                        let amount = creation.amount * (user_shares as f64 / total_shares as f64);
                        shares.push(ExpenseShare {
                            expense_id: creation.group_id,
                            user_id: *user_id,
                            amount,
                            is_settled: false,
                        });
                    } else {
                        return Err("All participants must have share counts specified".into());
                    }
                }
            },
        }

        // Note: expense_id will be set correctly by the caller

        Ok(shares)
    }

    pub async fn get_expense(&self, expense_id: &Uuid, _user_id: &Uuid) -> Result<Option<ExpenseInfo>, Box<dyn Error>> {
        // TODO: Verify user has access to this expense through group membership
        let expense = match self.expense_repository.get_expense_by_id(expense_id).await? {
            Some(e) => e,
            None => return Ok(None),
        };

        let shares = self.share_repository.get_expense_shares(expense_id).await?;
        
        // Convert to ExpenseInfo (would need user name lookups in real implementation)
        Ok(Some(ExpenseInfo {
            id: expense.id,
            group_id: expense.group_id,
            description: expense.description,
            amount: expense.amount,
            currency: expense.currency,
            paid_by: expense.paid_by,
            paid_by_name: "User".to_string(), // TODO: Lookup username
            created_by: expense.created_by,
            created_by_name: "User".to_string(), // TODO: Lookup username
            category: expense.category,
            date: expense.date,
            shares: shares.into_iter().map(|s| crate::expenses::domain::expense::ExpenseShareInfo {
                user_id: s.user_id,
                username: "User".to_string(), // TODO: Lookup username
                amount: s.amount,
                is_settled: s.is_settled,
            }).collect(),
            created_at: expense.created_at,
        }))
    }

    pub async fn get_group_expenses(&self, group_id: &Uuid, _user_id: &Uuid, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<ExpenseInfo>, Box<dyn Error>> {
        // TODO: Verify user is member of group
        self.expense_repository.get_group_expenses(group_id, limit, offset).await
    }

    pub async fn get_group_balances(&self, group_id: &Uuid, _user_id: &Uuid) -> Result<GroupBalance, Box<dyn Error>> {
        // TODO: Verify user is member of group
        self.balance_repository.calculate_group_balances(group_id).await
    }

    pub async fn get_user_balance(&self, user_id: &Uuid, group_id: &Uuid) -> Result<f64, Box<dyn Error>> {
        self.balance_repository.calculate_user_balance(user_id, group_id).await
    }

    pub async fn get_debt_summary(&self, group_id: &Uuid, _user_id: &Uuid) -> Result<Vec<DebtSummary>, Box<dyn Error>> {
        // TODO: Verify user is member of group
        self.balance_repository.get_debt_summary(group_id).await
    }

    pub async fn settle_debt(&self, group_id: &Uuid, settle: SettleDebt, _settled_by: Uuid) -> Result<(), Box<dyn Error>> {
        // TODO: Verify user has permission
        if settle.amount <= 0.0 {
            return Err("Settlement amount must be positive".into());
        }

        // Create payment record
        let payment = Payment {
            id: Uuid::new_v4(),
            group_id: *group_id,
            from_user: settle.debtor_id,
            to_user: settle.creditor_id,
            amount: settle.amount,
            currency: "USD".to_string(), // TODO: Get from group settings
            description: format!("Debt settlement: ${:.2}", settle.amount),
            created_at: Utc::now(),
        };

        self.payment_repository.create_payment(&payment).await?;

        // TODO: Update expense shares to mark relevant portions as settled

        Ok(())
    }

    pub async fn delete_expense(&self, expense_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // TODO: Verify user has permission (creator or group admin)
        
        // Delete shares first
        self.share_repository.delete_expense_shares(expense_id).await?;
        
        // Delete expense
        self.expense_repository.delete_expense(expense_id).await?;

        Ok(())
    }

    pub async fn get_user_debts(&self, user_id: &Uuid) -> Result<Vec<DebtSummary>, Box<dyn Error>> {
        self.balance_repository.get_user_debts(user_id).await
    }

    pub async fn search_expenses(&self, filter: ExpenseFilter, user_id: &Uuid) -> Result<Vec<ExpenseInfo>, Box<dyn Error>> {
        // TODO: Verify user has access to requested groups
        self.expense_repository.get_expenses(&filter).await
    }
}