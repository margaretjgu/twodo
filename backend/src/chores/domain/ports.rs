use async_trait::async_trait;
use uuid::Uuid;
use super::chore::{Chore, ChoreInfo, ChoreUpdate, ChoreFilter, ChoreStats, ChoreComment, ChoreCommentInfo};
use std::error::Error;

#[async_trait]
pub trait ChoreRepository: Send + Sync {
    async fn create_chore(&self, chore: &Chore) -> Result<(), Box<dyn Error>>;
    async fn get_chore_by_id(&self, chore_id: &Uuid) -> Result<Option<Chore>, Box<dyn Error>>;
    async fn update_chore(&self, chore_id: &Uuid, update: &ChoreUpdate) -> Result<(), Box<dyn Error>>;
    async fn delete_chore(&self, chore_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn get_chores(&self, filter: &ChoreFilter) -> Result<Vec<ChoreInfo>, Box<dyn Error>>;
    async fn get_user_chores(&self, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<Vec<ChoreInfo>, Box<dyn Error>>;
    async fn get_group_chores(&self, group_id: &Uuid) -> Result<Vec<ChoreInfo>, Box<dyn Error>>;
    async fn get_overdue_chores(&self, group_id: Option<&Uuid>) -> Result<Vec<ChoreInfo>, Box<dyn Error>>;
}

#[async_trait]
pub trait ChoreStatsRepository: Send + Sync {
    async fn get_user_stats(&self, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<ChoreStats, Box<dyn Error>>;
    async fn get_group_stats(&self, group_id: &Uuid) -> Result<ChoreStats, Box<dyn Error>>;
}

#[async_trait]
pub trait ChoreCommentRepository: Send + Sync {
    async fn add_comment(&self, comment: &ChoreComment) -> Result<(), Box<dyn Error>>;
    async fn get_chore_comments(&self, chore_id: &Uuid) -> Result<Vec<ChoreCommentInfo>, Box<dyn Error>>;
    async fn delete_comment(&self, comment_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
pub trait RecurrenceService: Send + Sync {
    async fn create_recurring_instances(&self, chore: &Chore) -> Result<Vec<Chore>, Box<dyn Error>>;
    async fn check_and_create_next_instances(&self) -> Result<(), Box<dyn Error>>;
}