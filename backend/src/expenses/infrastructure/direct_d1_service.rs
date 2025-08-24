use worker::{D1Database, Error as WorkerError};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::expenses::domain::expense::{
    Expense, ExpenseInfo, ExpenseCreation, ExpenseShare, Payment, UserBalance, GroupBalance, SettleDebt, SplitType,
};
use crate::auth::infrastructure::PersistentMemoryUserRepository;
use crate::auth::domain::ports::UserRepository;

pub struct DirectD1ExpenseService {
    db: D1Database,
    user_repo: PersistentMemoryUserRepository,
}

impl DirectD1ExpenseService {
    pub fn new(db: D1Database) -> Self {
        Self {
            db,
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

    pub async fn create_expense(&self, expense: &Expense) -> Result<(), WorkerError> {
        let stmt = self.db.prepare("INSERT INTO expenses (id, group_id, description, amount, currency, paid_by, created_by, category, date, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)");
        
        stmt.bind(&[
            expense.id.to_string().into(),
            expense.group_id.to_string().into(),
            expense.description.clone().into(),
            expense.amount.into(),
            expense.currency.clone().into(),
            expense.paid_by.to_string().into(),
            expense.created_by.to_string().into(),
            expense.category.clone().unwrap_or_default().into(),
            expense.date.to_rfc3339().into(),
            expense.created_at.to_rfc3339().into(),
            expense.updated_at.to_rfc3339().into(),
        ])?
        .run()
        .await?;

        Ok(())
    }

    pub async fn create_shares(&self, shares: &[ExpenseShare]) -> Result<(), WorkerError> {
        for share in shares {
            let stmt = self.db.prepare("INSERT INTO expense_shares (expense_id, user_id, amount, is_settled) VALUES (?1, ?2, ?3, ?4)");
            
            stmt.bind(&[
                share.expense_id.to_string().into(),
                share.user_id.to_string().into(),
                share.amount.into(),
                (share.is_settled as i32).into(),
            ])?
            .run()
            .await?;
        }

        Ok(())
    }

    pub async fn create_payment(&self, payment: &Payment) -> Result<(), WorkerError> {
        let stmt = self.db.prepare("INSERT INTO payments (id, group_id, from_user, to_user, amount, currency, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)");
        
        stmt.bind(&[
            payment.id.to_string().into(),
            payment.group_id.to_string().into(),
            payment.from_user.to_string().into(),
            payment.to_user.to_string().into(),
            payment.amount.into(),
            payment.currency.clone().into(),
            payment.description.clone().into(),
            payment.created_at.to_rfc3339().into(),
        ])?
        .run()
        .await?;

        Ok(())
    }

    pub async fn get_group_expenses(&self, group_id: &Uuid) -> Result<Vec<ExpenseInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT * FROM expenses WHERE group_id = ?1 ORDER BY created_at DESC");
        let results = stmt.bind(&[group_id.to_string().into()])?.all().await?;

        let mut expense_infos = Vec::new();
        
        // Using the worker v0.6.0 API
        for row in results.results::<Value>()? {
            let expense_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let paid_by = Uuid::parse_str(row["paid_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let created_by = Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;

            let paid_by_name = self.get_username(&paid_by).await;
            let created_by_name = self.get_username(&created_by).await;

            // Get shares for this expense
            let shares = self.get_expense_shares(&expense_id).await?;

            expense_infos.push(ExpenseInfo {
                id: expense_id,
                group_id: *group_id,
                description: row["description"].as_str().unwrap_or("").to_string(),
                amount: row["amount"].as_f64().unwrap_or(0.0),
                currency: row["currency"].as_str().unwrap_or("USD").to_string(),
                paid_by,
                created_by,
                category: Some(row["category"].as_str().unwrap_or("").to_string()),
                date: DateTime::parse_from_rfc3339(row["date"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                paid_by_name,
                created_by_name,
                shares,
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
            });
        }

        Ok(expense_infos)
    }

    pub async fn get_expense_shares(&self, expense_id: &Uuid) -> Result<Vec<crate::expenses::domain::expense::ExpenseShareInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT * FROM expense_shares WHERE expense_id = ?1");
        let results = stmt.bind(&[expense_id.to_string().into()])?.all().await?;

        let mut shares = Vec::new();
        for row in results.results::<Value>()? {
            let user_id = Uuid::parse_str(row["user_id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let username = self.get_username(&user_id).await;

            shares.push(crate::expenses::domain::expense::ExpenseShareInfo {
                user_id,
                username,
                amount: row["amount"].as_f64().unwrap_or(0.0),
                is_settled: row["is_settled"].as_i64().unwrap_or(0) != 0,
            });
        }

        Ok(shares)
    }

    pub async fn calculate_group_balances(&self, group_id: &Uuid) -> Result<GroupBalance, WorkerError> {
        let mut balances_map = std::collections::HashMap::new();

        // Get all expenses for this group (add to paid_by user)
        let expense_stmt = self.db.prepare("SELECT paid_by, amount FROM expenses WHERE group_id = ?1");
        let expense_results = expense_stmt.bind(&[group_id.to_string().into()])?.all().await?;

        for row in expense_results.results::<Value>()? {
            let paid_by_str = row["paid_by"].as_str().unwrap_or("");
            let amount = row["amount"].as_f64().unwrap_or(0.0);
            
            let paid_by = Uuid::parse_str(paid_by_str)
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            
            *balances_map.entry(paid_by).or_insert(0.0) += amount;
        }

        // Get all shares for this group (subtract from user_id)
        let share_stmt = self.db.prepare("SELECT es.user_id, es.amount FROM expense_shares es JOIN expenses e ON es.expense_id = e.id WHERE e.group_id = ?1");
        let share_results = share_stmt.bind(&[group_id.to_string().into()])?.all().await?;

        for row in share_results.results::<Value>()? {
            let user_id_str = row["user_id"].as_str().unwrap_or("");
            let amount = row["amount"].as_f64().unwrap_or(0.0);
            
            let user_id = Uuid::parse_str(user_id_str)
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            
            *balances_map.entry(user_id).or_insert(0.0) -= amount;
        }

        // Get all payments for this group (accounting for debt settlement)
        let payment_stmt = self.db.prepare("SELECT from_user, to_user, amount FROM payments WHERE group_id = ?1");
        let payment_results = payment_stmt.bind(&[group_id.to_string().into()])?.all().await?;

        for row in payment_results.results::<Value>()? {
            let from_user_str = row["from_user"].as_str().unwrap_or("");
            let to_user_str = row["to_user"].as_str().unwrap_or("");
            let amount = row["amount"].as_f64().unwrap_or(0.0);
            
            let from_user = Uuid::parse_str(from_user_str)
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let to_user = Uuid::parse_str(to_user_str)
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            
            // Add to payer (reduces their debt, makes balance more positive)
            *balances_map.entry(from_user).or_insert(0.0) += amount;
            // Subtract from receiver (reduces what they're owed, makes balance less positive)
            *balances_map.entry(to_user).or_insert(0.0) -= amount;
        }

        // Convert to UserBalance vec with usernames
        let mut balances = Vec::new();
        for (user_id, net_balance) in balances_map {
            let username = self.get_username(&user_id).await;
            balances.push(UserBalance {
                user_id,
                username,
                net_balance,
            });
        }

        Ok(GroupBalance {
            group_id: *group_id,
            group_name: format!("Group {}", group_id), // TODO: Get actual group name
            balances,
        })
    }

    pub async fn delete_expense(&self, expense_id: &Uuid) -> Result<(), WorkerError> {
        // Delete shares first (foreign key constraint)
        let delete_shares_stmt = self.db.prepare("DELETE FROM expense_shares WHERE expense_id = ?1");
        delete_shares_stmt.bind(&[expense_id.to_string().into()])?.run().await?;

        // Delete expense
        let delete_expense_stmt = self.db.prepare("DELETE FROM expenses WHERE id = ?1");
        delete_expense_stmt.bind(&[expense_id.to_string().into()])?.run().await?;

        Ok(())
    }

    // Additional methods needed by handlers
    pub async fn create_expense_from_creation(&self, creation: ExpenseCreation, created_by: Uuid) -> Result<(), WorkerError> {
        let expense = Expense {
            id: Uuid::new_v4(),
            group_id: creation.group_id,
            description: creation.description.clone(),
            amount: creation.amount,
            currency: creation.currency.clone(),
            paid_by: creation.paid_by,
            created_by,
            category: creation.category.clone(),
            date: creation.date.unwrap_or_else(|| Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.create_expense(&expense).await?;

        // Calculate and create shares based on split_type
        let expense_shares = self.calculate_shares_from_creation(&creation, &expense).await?;
        if !expense_shares.is_empty() {
            self.create_shares(&expense_shares).await?;
        }

        Ok(())
    }

    pub async fn get_expense(&self, expense_id: &Uuid, _user_id: &Uuid) -> Result<Option<ExpenseInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT * FROM expenses WHERE id = ?1");
        let result = stmt.bind(&[expense_id.to_string().into()])?.first::<Value>(None).await?;

        if let Some(row) = result {
            let paid_by = Uuid::parse_str(row["paid_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let created_by = Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let group_id = Uuid::parse_str(row["group_id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;

            let paid_by_name = self.get_username(&paid_by).await;
            let created_by_name = self.get_username(&created_by).await;
            let shares = self.get_expense_shares(expense_id).await?;

            let expense_info = ExpenseInfo {
                id: *expense_id,
                group_id,
                description: row["description"].as_str().unwrap_or("").to_string(),
                amount: row["amount"].as_f64().unwrap_or(0.0),
                currency: row["currency"].as_str().unwrap_or("USD").to_string(),
                paid_by,
                created_by,
                category: Some(row["category"].as_str().unwrap_or("").to_string()),
                date: DateTime::parse_from_rfc3339(row["date"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                paid_by_name,
                created_by_name,
                shares,
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
            };

            Ok(Some(expense_info))
        } else {
            Ok(None)
        }
    }

    pub async fn get_group_balances(&self, group_id: &Uuid, _user_id: &Uuid) -> Result<GroupBalance, WorkerError> {
        self.calculate_group_balances(group_id).await
    }

    pub async fn get_group_expenses_with_pagination(&self, group_id: &Uuid, _user_id: &Uuid, _limit: Option<usize>, _offset: Option<usize>) -> Result<Vec<ExpenseInfo>, WorkerError> {
        // For now, ignore pagination and return all expenses
        self.get_group_expenses(group_id).await
    }

    pub async fn settle_debt(&self, group_id: &Uuid, settle: SettleDebt, settled_by: Uuid) -> Result<(), WorkerError> {
        let payment = Payment {
            id: Uuid::new_v4(),
            group_id: *group_id,
            from_user: settle.debtor_id,
            to_user: settle.creditor_id,
            amount: settle.amount,
            currency: "USD".to_string(), // Default currency
            description: "Debt settlement".to_string(), // Default description
            created_at: Utc::now(),
        };

        self.create_payment(&payment).await
    }

    // Helper method to calculate shares from ExpenseCreation
    async fn calculate_shares_from_creation(&self, creation: &ExpenseCreation, expense: &Expense) -> Result<Vec<ExpenseShare>, WorkerError> {
        let mut shares = Vec::new();
        
        match &creation.split_type {
            SplitType::Equal => {
                // Split equally among all participants
                let amount_per_person = expense.amount / creation.participants.len() as f64;
                for participant_id in &creation.participants {
                    shares.push(ExpenseShare {
                        expense_id: expense.id,
                        user_id: *participant_id,
                        amount: amount_per_person,
                        is_settled: false,
                    });
                }
            },
            SplitType::Exact(amounts) => {
                // Use exact amounts specified
                for participant_id in &creation.participants {
                    if let Some(&amount) = amounts.get(participant_id) {
                        shares.push(ExpenseShare {
                            expense_id: expense.id,
                            user_id: *participant_id,
                            amount,
                            is_settled: false,
                        });
                    }
                }
            },
            SplitType::Percentage(percentages) => {
                // Calculate amounts based on percentages
                for participant_id in &creation.participants {
                    if let Some(&percentage) = percentages.get(participant_id) {
                        let amount = expense.amount * (percentage / 100.0);
                        shares.push(ExpenseShare {
                            expense_id: expense.id,
                            user_id: *participant_id,
                            amount,
                            is_settled: false,
                        });
                    }
                }
            },
            SplitType::ByShares(share_counts) => {
                // Calculate amounts based on share counts
                let total_shares: u32 = share_counts.values().sum();
                if total_shares > 0 {
                    for participant_id in &creation.participants {
                        if let Some(&user_shares) = share_counts.get(participant_id) {
                            let amount = expense.amount * (user_shares as f64 / total_shares as f64);
                            shares.push(ExpenseShare {
                                expense_id: expense.id,
                                user_id: *participant_id,
                                amount,
                                is_settled: false,
                            });
                        }
                    }
                }
            },
        }
        
        Ok(shares)
    }
}
