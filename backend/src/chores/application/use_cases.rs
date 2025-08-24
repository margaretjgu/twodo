use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::chores::domain::chore::{
    Chore, ChoreCreation, ChoreUpdate, ChoreInfo, ChoreFilter, ChoreStats, 
    ChoreComment, ChoreCommentInfo, AddComment, ChoreStatus, Priority
};
use crate::chores::domain::ports::{ChoreRepository, ChoreStatsRepository, ChoreCommentRepository, RecurrenceService};
use std::error::Error;

pub struct ChoreService {
    chore_repository: Arc<dyn ChoreRepository>,
    stats_repository: Arc<dyn ChoreStatsRepository>,
    comment_repository: Arc<dyn ChoreCommentRepository>,
    recurrence_service: Arc<dyn RecurrenceService>,
}

impl ChoreService {
    pub fn new(
        chore_repository: Arc<dyn ChoreRepository>,
        stats_repository: Arc<dyn ChoreStatsRepository>,
        comment_repository: Arc<dyn ChoreCommentRepository>,
        recurrence_service: Arc<dyn RecurrenceService>,
    ) -> Self {
        Self {
            chore_repository,
            stats_repository,
            comment_repository,
            recurrence_service,
        }
    }

    pub async fn create_chore(&self, creation: ChoreCreation, created_by: Uuid) -> Result<ChoreInfo, Box<dyn Error>> {
        // Validate input
        if creation.title.trim().is_empty() {
            return Err("Chore title cannot be empty".into());
        }
        if creation.title.len() > 200 {
            return Err("Chore title cannot exceed 200 characters".into());
        }

        let now = Utc::now();
        let chore_id = Uuid::new_v4();

        // Create chore
        let chore = Chore {
            id: chore_id,
            group_id: creation.group_id,
            title: creation.title.trim().to_string(),
            description: creation.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
            assigned_to: creation.assigned_to,
            created_by,
            category: creation.category.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()),
            priority: creation.priority,
            status: ChoreStatus::Pending,
            due_date: creation.due_date,
            estimated_duration: creation.estimated_duration,
            recurrence: creation.recurrence.clone(),
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        self.chore_repository.create_chore(&chore).await?;

        // Handle recurrence if specified
        if chore.recurrence.is_some() {
            let _recurring_instances = self.recurrence_service.create_recurring_instances(&chore).await?;
            // Note: In a full implementation, you'd save these instances
        }

        // Return chore info
        self.get_chore(&chore_id, &created_by).await?.ok_or("Failed to retrieve created chore".into())
    }

    pub async fn get_chore(&self, chore_id: &Uuid, user_id: &Uuid) -> Result<Option<ChoreInfo>, Box<dyn Error>> {
        // TODO: Verify user has access to this chore through group membership
        let chore = match self.chore_repository.get_chore_by_id(chore_id).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        // Convert to ChoreInfo (would need user/group name lookups in real implementation)
        let is_overdue = chore.due_date.map_or(false, |due| due < Utc::now() && chore.status != ChoreStatus::Completed);

        Ok(Some(ChoreInfo {
            id: chore.id,
            group_id: chore.group_id,
            group_name: "Group".to_string(), // TODO: Lookup group name
            title: chore.title,
            description: chore.description,
            assigned_to: chore.assigned_to,
            assigned_to_name: chore.assigned_to.map(|_| "User".to_string()), // TODO: Lookup username
            created_by: chore.created_by,
            created_by_name: "User".to_string(), // TODO: Lookup username
            category: chore.category,
            priority: chore.priority,
            status: chore.status,
            due_date: chore.due_date,
            estimated_duration: chore.estimated_duration,
            recurrence: chore.recurrence,
            created_at: chore.created_at,
            updated_at: chore.updated_at,
            completed_at: chore.completed_at,
            is_overdue,
        }))
    }

    pub async fn update_chore(&self, chore_id: &Uuid, user_id: &Uuid, update: ChoreUpdate) -> Result<(), Box<dyn Error>> {
        // TODO: Verify user has permission to update this chore
        
        // Validate updates
        if let Some(ref title) = update.title {
            if title.trim().is_empty() {
                return Err("Chore title cannot be empty".into());
            }
            if title.len() > 200 {
                return Err("Chore title cannot exceed 200 characters".into());
            }
        }

        self.chore_repository.update_chore(chore_id, &update).await
    }

    pub async fn complete_chore(&self, chore_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // TODO: Verify user has permission
        let update = ChoreUpdate {
            status: Some(ChoreStatus::Completed),
            title: None,
            description: None,
            assigned_to: None,
            category: None,
            priority: None,
            due_date: None,
            estimated_duration: None,
            recurrence: None,
        };

        self.chore_repository.update_chore(chore_id, &update).await
    }

    pub async fn assign_chore(&self, chore_id: &Uuid, assignee_id: &Uuid, assigner_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // TODO: Verify assigner has permission
        let update = ChoreUpdate {
            assigned_to: Some(*assignee_id),
            status: Some(ChoreStatus::Pending),
            title: None,
            description: None,
            category: None,
            priority: None,
            due_date: None,
            estimated_duration: None,
            recurrence: None,
        };

        self.chore_repository.update_chore(chore_id, &update).await
    }

    pub async fn get_user_chores(&self, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<Vec<ChoreInfo>, Box<dyn Error>> {
        self.chore_repository.get_user_chores(user_id, group_id).await
    }

    pub async fn get_group_chores(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Vec<ChoreInfo>, Box<dyn Error>> {
        // TODO: Verify user is member of group
        self.chore_repository.get_group_chores(group_id).await
    }

    pub async fn search_chores(&self, filter: ChoreFilter, user_id: &Uuid) -> Result<Vec<ChoreInfo>, Box<dyn Error>> {
        // TODO: Verify user has access to requested groups
        self.chore_repository.get_chores(&filter).await
    }

    pub async fn get_overdue_chores(&self, group_id: Option<&Uuid>, user_id: &Uuid) -> Result<Vec<ChoreInfo>, Box<dyn Error>> {
        // TODO: Verify user has access
        self.chore_repository.get_overdue_chores(group_id).await
    }

    pub async fn get_user_stats(&self, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<ChoreStats, Box<dyn Error>> {
        self.stats_repository.get_user_stats(user_id, group_id).await
    }

    pub async fn get_group_stats(&self, group_id: &Uuid, user_id: &Uuid) -> Result<ChoreStats, Box<dyn Error>> {
        // TODO: Verify user is member of group
        self.stats_repository.get_group_stats(group_id).await
    }

    pub async fn add_comment(&self, chore_id: &Uuid, user_id: &Uuid, add_comment: AddComment) -> Result<(), Box<dyn Error>> {
        // TODO: Verify user has access to this chore
        if add_comment.content.trim().is_empty() {
            return Err("Comment cannot be empty".into());
        }

        let comment = ChoreComment {
            id: Uuid::new_v4(),
            chore_id: *chore_id,
            user_id: *user_id,
            content: add_comment.content.trim().to_string(),
            created_at: Utc::now(),
        };

        self.comment_repository.add_comment(&comment).await
    }

    pub async fn get_chore_comments(&self, chore_id: &Uuid, user_id: &Uuid) -> Result<Vec<ChoreCommentInfo>, Box<dyn Error>> {
        // TODO: Verify user has access to this chore
        self.comment_repository.get_chore_comments(chore_id).await
    }

    pub async fn delete_chore(&self, chore_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // TODO: Verify user has permission (creator, assignee, or group admin)
        self.chore_repository.delete_chore(chore_id).await
    }

    pub async fn process_recurring_chores(&self) -> Result<(), Box<dyn Error>> {
        // Background task to create next instances of recurring chores
        self.recurrence_service.check_and_create_next_instances().await
    }
}