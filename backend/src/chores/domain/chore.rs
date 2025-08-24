use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chore {
    pub id: Uuid,
    pub group_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>, // User assigned to this chore
    pub created_by: Uuid,
    pub category: Option<String>,
    pub priority: Priority,
    pub status: ChoreStatus,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_duration: Option<u32>, // Duration in minutes
    pub recurrence: Option<RecurrencePattern>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChoreStatus {
    Pending,
    InProgress,
    Completed,
    Overdue,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecurrencePattern {
    pub frequency: RecurrenceFrequency,
    pub interval: u32, // Every N days/weeks/months
    pub days_of_week: Option<Vec<Weekday>>, // For weekly recurrence
    pub day_of_month: Option<u32>, // For monthly recurrence
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoreCreation {
    pub group_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub category: Option<String>,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_duration: Option<u32>,
    pub recurrence: Option<RecurrencePattern>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoreUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub category: Option<String>,
    pub priority: Option<Priority>,
    pub status: Option<ChoreStatus>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_duration: Option<u32>,
    pub recurrence: Option<RecurrencePattern>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoreInfo {
    pub id: Uuid,
    pub group_id: Uuid,
    pub group_name: String,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub category: Option<String>,
    pub priority: Priority,
    pub status: ChoreStatus,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_duration: Option<u32>,
    pub recurrence: Option<RecurrencePattern>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub is_overdue: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoreComment {
    pub id: Uuid,
    pub chore_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoreCommentInfo {
    pub id: Uuid,
    pub chore_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoreFilter {
    pub group_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub status: Option<ChoreStatus>,
    pub priority: Option<Priority>,
    pub category: Option<String>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub include_completed: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoreStats {
    pub total_chores: usize,
    pub completed_chores: usize,
    pub pending_chores: usize,
    pub overdue_chores: usize,
    pub completion_rate: f64, // Percentage
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddComment {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoreAssignment {
    pub chore_id: Uuid,
    pub assigned_to: Uuid,
}