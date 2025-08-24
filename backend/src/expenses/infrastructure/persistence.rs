use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;
use crate::auth::infrastructure::persistence::persistent_memory_repository::PersistentMemoryUserRepository;
use crate::auth::domain::ports::UserRepository;

use crate::expenses::domain::expense::{
    Expense, ExpenseShare, ExpenseInfo, ExpenseShareInfo, UserBalance, 
    GroupBalance, DebtSummary, Payment, ExpenseFilter
};
use crate::expenses::domain::ports::{
    ExpenseRepository, ExpenseShareRepository, BalanceRepository, PaymentRepository
};
use std::error::Error;

// Global storage for demo purposes (similar to auth implementation)
static EXPENSES: Lazy<Mutex<HashMap<Uuid, Expense>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static EXPENSE_SHARES: Lazy<Mutex<HashMap<Uuid, Vec<ExpenseShare>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static PAYMENTS: Lazy<Mutex<HashMap<Uuid, Payment>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct InMemoryExpenseRepository {
    user_repo: PersistentMemoryUserRepository,
}

impl InMemoryExpenseRepository {
    pub fn new() -> Self {
        Self {
            user_repo: PersistentMemoryUserRepository::new(),
        }
    }
    
    async fn get_username(&self, user_id: &Uuid) -> String {
        match self.user_repo.get_user_by_id(user_id).await {
            Ok(Some(user)) => user.username,
            Ok(None) => format!("Unknown User ({})", user_id),
            Err(_) => format!("Error loading user ({})", user_id),
        }
    }
}

#[async_trait]
impl ExpenseRepository for InMemoryExpenseRepository {
    async fn create_expense(&self, expense: &Expense) -> Result<(), Box<dyn Error>> {
        let mut expenses = EXPENSES.lock().unwrap();
        expenses.insert(expense.id, expense.clone());
        Ok(())
    }

    async fn get_expense_by_id(&self, expense_id: &Uuid) -> Result<Option<Expense>, Box<dyn Error>> {
        let expenses = EXPENSES.lock().unwrap();
        Ok(expenses.get(expense_id).cloned())
    }

    async fn update_expense(&self, expense: &Expense) -> Result<(), Box<dyn Error>> {
        let mut expenses = EXPENSES.lock().unwrap();
        expenses.insert(expense.id, expense.clone());
        Ok(())
    }

    async fn delete_expense(&self, expense_id: &Uuid) -> Result<(), Box<dyn Error>> {
        let mut expenses = EXPENSES.lock().unwrap();
        expenses.remove(expense_id);
        
        // Also remove associated shares
        let mut shares = EXPENSE_SHARES.lock().unwrap();
        shares.remove(expense_id);
        
        Ok(())
    }

    async fn get_expenses(&self, filter: &ExpenseFilter) -> Result<Vec<ExpenseInfo>, Box<dyn Error>> {
        // Clone the data we need and release locks immediately
        let (filtered_expenses, expense_shares_map) = {
            let expenses = EXPENSES.lock().unwrap();
            let shares = EXPENSE_SHARES.lock().unwrap();
            
            let filtered: Vec<Expense> = expenses
                .values()
                .filter(|expense| {
                    // Apply filters
                    if let Some(group_id) = &filter.group_id {
                        if expense.group_id != *group_id {
                            return false;
                        }
                    }
                    if let Some(user_id) = &filter.involving_user {
                        if expense.created_by != *user_id && expense.paid_by != *user_id {
                            // Check if user is in shares
                            if let Some(expense_shares) = shares.get(&expense.id) {
                                if !expense_shares.iter().any(|s| s.user_id == *user_id) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                    }
                    true
                })
                .cloned()
                .collect();
            
            (filtered, shares.clone())
        };

        // Build results with async username lookups (locks are now released)
        let mut results = Vec::new();
        for expense in filtered_expenses {
            let paid_by_name = self.get_username(&expense.paid_by).await;
            let created_by_name = self.get_username(&expense.created_by).await;
            
            // Get shares and their usernames
            let expense_shares = expense_shares_map.get(&expense.id).cloned().unwrap_or_default();
            let mut share_infos = Vec::new();
            for share in expense_shares {
                let username = self.get_username(&share.user_id).await;
                share_infos.push(ExpenseShareInfo {
                    user_id: share.user_id,
                    username,
                    amount: share.amount,
                    is_settled: share.is_settled,
                });
            }
            
            results.push(ExpenseInfo {
                id: expense.id,
                group_id: expense.group_id,
                description: expense.description.clone(),
                amount: expense.amount,
                currency: expense.currency.clone(),
                paid_by: expense.paid_by,
                created_by: expense.created_by,
                category: expense.category.clone(),
                date: expense.date,
                paid_by_name,
                created_by_name,
                shares: share_infos,
                created_at: expense.created_at,
            });
        }

        // Sort by date (newest first)
        results.sort_by(|a, b| b.date.cmp(&a.date));
        
        // Apply limit and offset
        if let Some(offset) = filter.offset {
            if offset >= results.len() {
                return Ok(vec![]);
            }
            results = results.into_iter().skip(offset).collect();
        }
        
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    async fn get_group_expenses(&self, group_id: &Uuid, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<ExpenseInfo>, Box<dyn Error>> {
        let filter = ExpenseFilter {
            group_id: Some(*group_id),
            paid_by: None,
            involving_user: None,
            category: None,
            limit,
            offset,
            date_from: None,
            date_to: None,
        };
        self.get_expenses(&filter).await
    }
}

pub struct InMemoryExpenseShareRepository;

impl InMemoryExpenseShareRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ExpenseShareRepository for InMemoryExpenseShareRepository {
    async fn create_shares(&self, shares: &[ExpenseShare]) -> Result<(), Box<dyn Error>> {
        if shares.is_empty() {
            return Ok(());
        }
        
        let expense_id = shares[0].expense_id;
        let mut expense_shares = EXPENSE_SHARES.lock().unwrap();
        expense_shares.insert(expense_id, shares.to_vec());
        Ok(())
    }

    async fn get_expense_shares(&self, expense_id: &Uuid) -> Result<Vec<ExpenseShare>, Box<dyn Error>> {
        let shares = EXPENSE_SHARES.lock().unwrap();
        Ok(shares.get(expense_id).cloned().unwrap_or_default())
    }

    async fn update_share(&self, share: &ExpenseShare) -> Result<(), Box<dyn Error>> {
        let mut expense_shares = EXPENSE_SHARES.lock().unwrap();
        if let Some(shares) = expense_shares.get_mut(&share.expense_id) {
            if let Some(existing_share) = shares.iter_mut().find(|s| s.user_id == share.user_id) {
                existing_share.amount = share.amount;
                existing_share.is_settled = share.is_settled;
            }
        }
        Ok(())
    }

    async fn delete_expense_shares(&self, expense_id: &Uuid) -> Result<(), Box<dyn Error>> {
        let mut expense_shares = EXPENSE_SHARES.lock().unwrap();
        expense_shares.remove(expense_id);
        Ok(())
    }

    async fn get_user_shares(&self, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<Vec<ExpenseShare>, Box<dyn Error>> {
        let shares = EXPENSE_SHARES.lock().unwrap();
        let expenses = EXPENSES.lock().unwrap();
        
        let mut user_shares = Vec::new();
        
        for (expense_id, expense_shares) in shares.iter() {
            // Check group filter
            if let Some(group_id) = group_id {
                if let Some(expense) = expenses.get(expense_id) {
                    if expense.group_id != *group_id {
                        continue;
                    }
                }
            }
            
            // Add user's shares from this expense
            for share in expense_shares {
                if share.user_id == *user_id {
                    user_shares.push(share.clone());
                }
            }
        }
        
        Ok(user_shares)
    }
}

pub struct InMemoryBalanceRepository {
    user_repo: PersistentMemoryUserRepository,
}

impl InMemoryBalanceRepository {
    pub fn new() -> Self {
        Self {
            user_repo: PersistentMemoryUserRepository::new(),
        }
    }
    
    async fn get_username(&self, user_id: &Uuid) -> String {
        match self.user_repo.get_user_by_id(user_id).await {
            Ok(Some(user)) => user.username,
            Ok(None) => format!("Unknown User ({})", user_id),
            Err(_) => format!("Error loading user ({})", user_id),
        }
    }
}

#[async_trait]
impl BalanceRepository for InMemoryBalanceRepository {
    async fn calculate_group_balances(&self, group_id: &Uuid) -> Result<GroupBalance, Box<dyn Error>> {
        // Calculate balances and release locks immediately
        let user_balances_map = {
            let expenses = EXPENSES.lock().unwrap();
            let shares = EXPENSE_SHARES.lock().unwrap();
            let payments = PAYMENTS.lock().unwrap();
            
            let mut balances_map = HashMap::new();
            
            // Calculate balances for this group
            for expense in expenses.values() {
                if expense.group_id != *group_id {
                    continue;
                }
                
                // Add amount paid by user
                *balances_map.entry(expense.paid_by).or_insert(0.0) += expense.amount;
                
                // Subtract amounts owed by users
                if let Some(expense_shares) = shares.get(&expense.id) {
                    for share in expense_shares {
                        *balances_map.entry(share.user_id).or_insert(0.0) -= share.amount;
                    }
                }
            }
            
            // Account for payments made/received in this group
            for payment in payments.values() {
                if payment.group_id != *group_id {
                    continue;
                }
                
                // Subtract from payer (they paid out money, reducing their positive balance)
                *balances_map.entry(payment.from_user).or_insert(0.0) -= payment.amount;
                
                // Add to receiver (they received money, increasing their positive balance)
                *balances_map.entry(payment.to_user).or_insert(0.0) += payment.amount;
            }
            
            balances_map
        };
        
        // Convert to UserBalance vec with async username lookups (locks are now released)
        let mut balances = Vec::new();
        for (user_id, net_balance) in user_balances_map {
            let username = self.get_username(&user_id).await;
            balances.push(UserBalance {
                user_id,
                username,
                net_balance,
            });
        }
        
        Ok(GroupBalance {
            group_id: *group_id,
            group_name: format!("Group {}", group_id), // TODO: Get real group name from group repository
            balances,
        })
    }

    async fn calculate_user_balance(&self, user_id: &Uuid, group_id: &Uuid) -> Result<f64, Box<dyn Error>> {
        let group_balance = self.calculate_group_balances(group_id).await?;
        Ok(group_balance.balances.iter()
            .find(|b| b.user_id == *user_id)
            .map(|b| b.net_balance)
            .unwrap_or(0.0))
    }

    async fn get_debt_summary(&self, group_id: &Uuid) -> Result<Vec<DebtSummary>, Box<dyn Error>> {
        let group_balance = self.calculate_group_balances(group_id).await?;
        let mut debt_summaries = Vec::new();
        
        // Simple debt resolution: users with negative balances owe users with positive balances
        let mut creditors: Vec<_> = group_balance.balances.iter()
            .filter(|b| b.net_balance > 0.01)
            .map(|b| (b.user_id, b.net_balance, b.username.clone()))
            .collect();
        
        let mut debtors: Vec<_> = group_balance.balances.iter()
            .filter(|b| b.net_balance < -0.01)
            .map(|b| (b.user_id, -b.net_balance, b.username.clone()))
            .collect();
        
        // Sort by amount
        creditors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        debtors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Match debtors with creditors
        while !debtors.is_empty() && !creditors.is_empty() {
            let (debtor_id, debt_amount, debtor_name) = debtors.remove(0);
            let (creditor_id, credit_amount, creditor_name) = creditors.remove(0);
            
            let settlement_amount = debt_amount.min(credit_amount);
            
            debt_summaries.push(DebtSummary {
                debtor_id,
                debtor_name: debtor_name.clone(),
                creditor_id,
                creditor_name: creditor_name.clone(),
                amount: settlement_amount,
                currency: "USD".to_string(), // Demo currency
            });
            
            // Put back any remaining amounts
            if debt_amount > settlement_amount {
                debtors.insert(0, (debtor_id, debt_amount - settlement_amount, debtor_name));
            }
            if credit_amount > settlement_amount {
                creditors.insert(0, (creditor_id, credit_amount - settlement_amount, creditor_name));
            }
        }
        
        Ok(debt_summaries)
    }

    async fn get_user_debts(&self, user_id: &Uuid) -> Result<Vec<DebtSummary>, Box<dyn Error>> {
        // For now, get debts across all groups - in production you'd filter by groups user belongs to
        let group_ids: std::collections::HashSet<_> = {
            let expenses = EXPENSES.lock().unwrap();
            expenses.values().map(|e| e.group_id).collect()
        }; // Release lock here
        
        let mut user_debts = Vec::new();
        
        for group_id in group_ids {
            let group_debts = self.get_debt_summary(&group_id).await?;
            for debt in group_debts {
                if debt.debtor_id == *user_id || debt.creditor_id == *user_id {
                    user_debts.push(debt);
                }
            }
        }
        
        Ok(user_debts)
    }
}

pub struct InMemoryPaymentRepository;

impl InMemoryPaymentRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PaymentRepository for InMemoryPaymentRepository {
    async fn create_payment(&self, payment: &Payment) -> Result<(), Box<dyn Error>> {
        let mut payments = PAYMENTS.lock().unwrap();
        payments.insert(payment.id, payment.clone());
        Ok(())
    }

    async fn get_group_payments(&self, group_id: &Uuid) -> Result<Vec<Payment>, Box<dyn Error>> {
        let payments = PAYMENTS.lock().unwrap();
        Ok(payments.values()
            .filter(|p| p.group_id == *group_id)
            .cloned()
            .collect())
    }

    async fn get_user_payments(&self, user_id: &Uuid) -> Result<Vec<Payment>, Box<dyn Error>> {
        let payments = PAYMENTS.lock().unwrap();
        Ok(payments.values()
            .filter(|p| p.from_user == *user_id || p.to_user == *user_id)
            .cloned()
            .collect())
    }
}
