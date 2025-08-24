use worker::{D1Database, Error as WorkerError};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::chores::domain::chore::{
    Chore, ChoreInfo, ChoreCreation, ChoreStatus, Priority, ChoreAssignment,
};

pub struct DirectD1ChoreService {
    db: D1Database,
}

impl DirectD1ChoreService {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }

    async fn get_username(&self, user_id: &Uuid) -> Result<String, WorkerError> {
        let stmt = self.db.prepare("SELECT username FROM users WHERE id = ?1");
        let result = stmt.bind(&[user_id.to_string().into()])?.first::<Value>(None).await?;
        
        if let Some(row) = result {
            Ok(row["username"].as_str().unwrap_or("Unknown User").to_string())
        } else {
            Ok("Unknown User".to_string())
        }
    }

    async fn get_group_name(&self, group_id: &Uuid) -> Result<String, WorkerError> {
        let stmt = self.db.prepare("SELECT name FROM groups WHERE id = ?1");
        let result = stmt.bind(&[group_id.to_string().into()])?.first::<Value>(None).await?;
        
        if let Some(row) = result {
            Ok(row["name"].as_str().unwrap_or("Unknown Group").to_string())
        } else {
            Ok("Unknown Group".to_string())
        }
    }

    pub async fn create_chore_from_creation(&self, creation: ChoreCreation, created_by: Uuid) -> Result<ChoreInfo, WorkerError> {
        let chore = Chore {
            id: Uuid::new_v4(),
            group_id: creation.group_id,
            title: creation.title.clone(),
            description: creation.description.clone(),
            assigned_to: creation.assigned_to,
            created_by,
            status: ChoreStatus::Pending,
            priority: creation.priority,
            due_date: creation.due_date,
            category: creation.category.clone(),
            estimated_duration: creation.estimated_duration,
            recurrence: creation.recurrence.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };

        // Create the chore
        self.create_chore(&chore).await?;

        // Return chore info
        let created_by_name = self.get_username(&created_by).await.unwrap_or_else(|_| "Unknown User".to_string());
        let group_name = self.get_group_name(&chore.group_id).await.unwrap_or_else(|_| "Unknown Group".to_string());
        let assigned_to_name = if let Some(assigned_to) = &chore.assigned_to {
            Some(self.get_username(assigned_to).await.unwrap_or_else(|_| "Unknown User".to_string()))
        } else {
            None
        };
        
        Ok(ChoreInfo {
            id: chore.id,
            group_id: chore.group_id,
            group_name,
            title: chore.title,
            description: chore.description,
            assigned_to: chore.assigned_to,
            assigned_to_name,
            created_by,
            created_by_name,
            status: chore.status,
            priority: chore.priority,
            due_date: chore.due_date,
            category: chore.category,
            estimated_duration: chore.estimated_duration,
            recurrence: chore.recurrence,
            created_at: chore.created_at,
            updated_at: chore.updated_at,
            completed_at: chore.completed_at,
            is_overdue: false, // Simplified for now
        })
    }

    pub async fn create_chore(&self, chore: &Chore) -> Result<(), WorkerError> {
        let status_str = match chore.status {
            ChoreStatus::Pending => "pending",
            ChoreStatus::InProgress => "in_progress",
            ChoreStatus::Completed => "completed",
            ChoreStatus::Overdue => "overdue",
            ChoreStatus::Cancelled => "cancelled",
        };

        let priority_str = match chore.priority {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Urgent => "urgent",
        };

        let stmt = self.db.prepare("INSERT INTO chores (id, group_id, title, description, assigned_to, created_by, status, priority, due_date, category, estimated_duration, created_at, updated_at, completed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)");
        
        stmt.bind(&[
            chore.id.to_string().into(),
            chore.group_id.to_string().into(),
            chore.title.clone().into(),
            chore.description.clone().unwrap_or_default().into(),
            chore.assigned_to.map(|a| a.to_string()).unwrap_or_default().into(),
            chore.created_by.to_string().into(),
            status_str.into(),
            priority_str.into(),
            chore.due_date.map(|d| d.to_rfc3339()).unwrap_or_default().into(),
            chore.category.clone().unwrap_or_default().into(),
            chore.estimated_duration.unwrap_or(0).into(),
            chore.created_at.to_rfc3339().into(),
            chore.updated_at.to_rfc3339().into(),
            chore.completed_at.map(|d| d.to_rfc3339()).unwrap_or_default().into(),
        ])?
        .run()
        .await?;

        Ok(())
    }

    pub async fn get_chore_by_id(&self, chore_id: &Uuid, _user_id: &Uuid) -> Result<Option<ChoreInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT * FROM chores WHERE id = ?1");
        let result = stmt.bind(&[chore_id.to_string().into()])?.first::<Value>(None).await?;

        if let Some(row) = result {
            let created_by = Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let group_id = Uuid::parse_str(row["group_id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let assigned_to = if let Some(assigned_str) = row["assigned_to"].as_str() {
                if !assigned_str.is_empty() {
                    Some(Uuid::parse_str(assigned_str)
                        .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?)
                } else {
                    None
                }
            } else {
                None
            };

            let status = match row["status"].as_str().unwrap_or("pending") {
                "in_progress" => ChoreStatus::InProgress,
                "completed" => ChoreStatus::Completed,
                "overdue" => ChoreStatus::Overdue,
                "cancelled" => ChoreStatus::Cancelled,
                _ => ChoreStatus::Pending,
            };

            let priority = match row["priority"].as_str().unwrap_or("medium") {
                "low" => Priority::Low,
                "high" => Priority::High,
                "urgent" => Priority::Urgent,
                _ => Priority::Medium,
            };

            let created_by_name = self.get_username(&created_by).await.unwrap_or_else(|_| "Unknown User".to_string());
            let group_name = self.get_group_name(&group_id).await.unwrap_or_else(|_| "Unknown Group".to_string());
            let assigned_to_name = if let Some(assigned_to) = &assigned_to {
                Some(self.get_username(assigned_to).await.unwrap_or_else(|_| "Unknown User".to_string()))
            } else {
                None
            };

            let chore_info = ChoreInfo {
                id: *chore_id,
                group_id,
                group_name,
                title: row["title"].as_str().unwrap_or("").to_string(),
                description: Some(row["description"].as_str().unwrap_or("").to_string()),
                assigned_to,
                assigned_to_name,
                created_by,
                created_by_name,
                status,
                priority,
                due_date: if let Some(due_str) = row["due_date"].as_str() {
                    if !due_str.is_empty() {
                        Some(DateTime::parse_from_rfc3339(due_str)
                            .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                            .with_timezone(&Utc))
                    } else {
                        None
                    }
                } else {
                    None
                },
                category: Some(row["category"].as_str().unwrap_or("").to_string()),
                estimated_duration: Some(row["estimated_duration"].as_i64().unwrap_or(0) as u32),
                recurrence: None, // Simplified for now
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row["updated_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                is_overdue: false, // Simplified for now
                completed_at: if let Some(completed_str) = row["completed_at"].as_str() {
                    if !completed_str.is_empty() {
                        Some(DateTime::parse_from_rfc3339(completed_str)
                            .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                            .with_timezone(&Utc))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            Ok(Some(chore_info))
        } else {
            Ok(None)
        }
    }

    pub async fn get_group_chores(&self, group_id: &Uuid, _user_id: &Uuid) -> Result<Vec<ChoreInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT * FROM chores WHERE group_id = ?1 ORDER BY created_at DESC");
        let results = stmt.bind(&[group_id.to_string().into()])?.all().await?;

        let mut chores = Vec::new();
        for row in results.results::<Value>()? {
            let chore_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let created_by = Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let assigned_to = if let Some(assigned_str) = row["assigned_to"].as_str() {
                if !assigned_str.is_empty() {
                    Some(Uuid::parse_str(assigned_str)
                        .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?)
                } else {
                    None
                }
            } else {
                None
            };

            let status = match row["status"].as_str().unwrap_or("pending") {
                "in_progress" => ChoreStatus::InProgress,
                "completed" => ChoreStatus::Completed,
                "overdue" => ChoreStatus::Overdue,
                "cancelled" => ChoreStatus::Cancelled,
                _ => ChoreStatus::Pending,
            };

            let priority = match row["priority"].as_str().unwrap_or("medium") {
                "low" => Priority::Low,
                "high" => Priority::High,
                "urgent" => Priority::Urgent,
                _ => Priority::Medium,
            };

            let created_by_name = self.get_username(&created_by).await.unwrap_or_else(|_| "Unknown User".to_string());
            let group_name = self.get_group_name(group_id).await.unwrap_or_else(|_| "Unknown Group".to_string());
            let assigned_to_name = if let Some(assigned_to) = &assigned_to {
                Some(self.get_username(assigned_to).await.unwrap_or_else(|_| "Unknown User".to_string()))
            } else {
                None
            };

            chores.push(ChoreInfo {
                id: chore_id,
                group_id: *group_id,
                group_name,
                title: row["title"].as_str().unwrap_or("").to_string(),
                description: Some(row["description"].as_str().unwrap_or("").to_string()),
                assigned_to,
                assigned_to_name,
                created_by,
                created_by_name,
                status,
                priority,
                due_date: if let Some(due_str) = row["due_date"].as_str() {
                    if !due_str.is_empty() {
                        Some(DateTime::parse_from_rfc3339(due_str)
                            .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                            .with_timezone(&Utc))
                    } else {
                        None
                    }
                } else {
                    None
                },
                category: Some(row["category"].as_str().unwrap_or("").to_string()),
                estimated_duration: Some(row["estimated_duration"].as_i64().unwrap_or(0) as u32),
                recurrence: None, // Simplified for now
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row["updated_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                is_overdue: false, // Simplified for now
                completed_at: if let Some(completed_str) = row["completed_at"].as_str() {
                    if !completed_str.is_empty() {
                        Some(DateTime::parse_from_rfc3339(completed_str)
                            .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                            .with_timezone(&Utc))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        }

        Ok(chores)
    }

    pub async fn update_chore_status(&self, chore_id: &Uuid, status: ChoreStatus, _user_id: &Uuid) -> Result<(), WorkerError> {
        let status_str = match status {
            ChoreStatus::Pending => "pending",
            ChoreStatus::InProgress => "in_progress",
            ChoreStatus::Completed => "completed",
            ChoreStatus::Overdue => "overdue",
            ChoreStatus::Cancelled => "cancelled",
        };

        let completed_at = if status == ChoreStatus::Completed {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        let stmt = if let Some(completed_time) = completed_at {
            let update_stmt = self.db.prepare("UPDATE chores SET status = ?1, completed_at = ?2, updated_at = ?3 WHERE id = ?4");
            update_stmt.bind(&[
                status_str.into(),
                completed_time.into(),
                Utc::now().to_rfc3339().into(),
                chore_id.to_string().into(),
            ])?
        } else {
            let update_stmt = self.db.prepare("UPDATE chores SET status = ?1, completed_at = '', updated_at = ?2 WHERE id = ?3");
            update_stmt.bind(&[
                status_str.into(),
                Utc::now().to_rfc3339().into(),
                chore_id.to_string().into(),
            ])?
        };

        stmt.run().await?;
        Ok(())
    }

    pub async fn delete_chore(&self, chore_id: &Uuid, _user_id: &Uuid) -> Result<(), WorkerError> {
        let stmt = self.db.prepare("DELETE FROM chores WHERE id = ?1");
        stmt.bind(&[chore_id.to_string().into()])?.run().await?;
        Ok(())
    }

    pub async fn assign_chore(&self, assignment: ChoreAssignment, _user_id: &Uuid) -> Result<(), WorkerError> {
        let stmt = self.db.prepare("UPDATE chores SET assigned_to = ?1, updated_at = ?2 WHERE id = ?3");
        stmt.bind(&[
            assignment.assigned_to.to_string().into(),
            Utc::now().to_rfc3339().into(),
            assignment.chore_id.to_string().into(),
        ])?
        .run()
        .await?;

        Ok(())
    }

    pub async fn get_user_chores(&self, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<Vec<ChoreInfo>, WorkerError> {
        let (query, bind_params) = if let Some(group_id) = group_id {
            ("SELECT * FROM chores WHERE assigned_to = ?1 AND group_id = ?2 ORDER BY created_at DESC", 
             vec![user_id.to_string(), group_id.to_string()])
        } else {
            ("SELECT * FROM chores WHERE assigned_to = ?1 ORDER BY created_at DESC",
             vec![user_id.to_string()])
        };

        let stmt = self.db.prepare(query);
        let bind_values: Vec<_> = bind_params.into_iter().map(|s| s.into()).collect();
        let results = stmt.bind(&bind_values)?.all().await?;

        let mut chores = Vec::new();
        for row in results.results::<Value>()? {
            let chore_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let created_by = Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let group_id = Uuid::parse_str(row["group_id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;

            let status = match row["status"].as_str().unwrap_or("pending") {
                "in_progress" => ChoreStatus::InProgress,
                "completed" => ChoreStatus::Completed,
                "overdue" => ChoreStatus::Overdue,
                "cancelled" => ChoreStatus::Cancelled,
                _ => ChoreStatus::Pending,
            };

            let priority = match row["priority"].as_str().unwrap_or("medium") {
                "low" => Priority::Low,
                "high" => Priority::High,
                "urgent" => Priority::Urgent,
                _ => Priority::Medium,
            };

            let created_by_name = self.get_username(&created_by).await.unwrap_or_else(|_| "Unknown User".to_string());
            let group_name = self.get_group_name(&group_id).await.unwrap_or_else(|_| "Unknown Group".to_string());

            chores.push(ChoreInfo {
                id: chore_id,
                group_id,
                group_name,
                title: row["title"].as_str().unwrap_or("").to_string(),
                description: Some(row["description"].as_str().unwrap_or("").to_string()),
                assigned_to: Some(*user_id),
                assigned_to_name: Some(self.get_username(user_id).await.unwrap_or_else(|_| "Unknown User".to_string())),
                created_by,
                created_by_name,
                status,
                priority,
                due_date: if let Some(due_str) = row["due_date"].as_str() {
                    if !due_str.is_empty() {
                        Some(DateTime::parse_from_rfc3339(due_str)
                            .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                            .with_timezone(&Utc))
                    } else {
                        None
                    }
                } else {
                    None
                },
                category: Some(row["category"].as_str().unwrap_or("").to_string()),
                estimated_duration: Some(row["estimated_duration"].as_i64().unwrap_or(0) as u32),
                recurrence: None, // Simplified for now
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row["updated_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                is_overdue: false, // Simplified for now
                completed_at: if let Some(completed_str) = row["completed_at"].as_str() {
                    if !completed_str.is_empty() {
                        Some(DateTime::parse_from_rfc3339(completed_str)
                            .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                            .with_timezone(&Utc))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        }

        Ok(chores)
    }
}
