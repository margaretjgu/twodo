use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: Uuid,
    pub group_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_all_day: bool,
    pub created_by: Uuid,
    pub category: Option<String>,
    pub color: Option<String>, // Hex color for UI
    pub recurrence: Option<RecurrenceRule>,
    pub reminder_minutes: Vec<u32>, // Minutes before event to send reminders
    pub visibility: EventVisibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventVisibility {
    Public,    // Visible to all group members
    Private,   // Only visible to creator and invited attendees
    Confidential, // Only visible to creator
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecurrenceRule {
    pub frequency: RecurrenceFrequency,
    pub interval: u32, // Repeat every N frequency units
    pub days_of_week: Option<Vec<Weekday>>,
    pub day_of_month: Option<u32>,
    pub week_of_month: Option<u32>, // For "second Tuesday of month" etc.
    pub month_of_year: Option<u32>,
    pub until: Option<DateTime<Utc>>, // End date
    pub count: Option<u32>, // Max number of occurrences
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
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventAttendee {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: AttendeeStatus,
    pub is_organizer: bool,
    pub invited_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AttendeeStatus {
    Pending,
    Accepted,
    Declined,
    Tentative,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventCreation {
    pub group_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_all_day: bool,
    pub category: Option<String>,
    pub color: Option<String>,
    pub recurrence: Option<RecurrenceRule>,
    pub reminder_minutes: Vec<u32>,
    pub visibility: EventVisibility,
    pub attendees: Vec<Uuid>, // User IDs to invite
    pub linked_chore_id: Option<Uuid>, // Link to a chore
    pub linked_expense_id: Option<Uuid>, // Link to an expense
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub is_all_day: Option<bool>,
    pub category: Option<String>,
    pub color: Option<String>,
    pub recurrence: Option<RecurrenceRule>,
    pub reminder_minutes: Option<Vec<u32>>,
    pub visibility: Option<EventVisibility>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventInfo {
    pub id: Uuid,
    pub group_id: Uuid,
    pub group_name: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_all_day: bool,
    pub created_by: Uuid,
    pub created_by_name: String,
    pub category: Option<String>,
    pub color: Option<String>,
    pub recurrence: Option<RecurrenceRule>,
    pub reminder_minutes: Vec<u32>,
    pub visibility: EventVisibility,
    pub attendees: Vec<EventAttendeeInfo>,
    pub user_status: Option<AttendeeStatus>, // Current user's attendance status
    pub can_edit: bool, // Whether current user can edit this event
    pub linked_chore_id: Option<Uuid>,
    pub linked_expense_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventAttendeeInfo {
    pub user_id: Uuid,
    pub username: String,
    pub status: AttendeeStatus,
    pub is_organizer: bool,
    pub responded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarView {
    pub events: Vec<EventInfo>,
    pub date_range: DateRange,
    pub view_type: ViewType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ViewType {
    Day,
    Week,
    Month,
    Year,
    Agenda,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventFilter {
    pub group_ids: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>, // Events where user is attendee
    pub created_by: Option<Uuid>,
    pub category: Option<String>,
    pub start_after: Option<DateTime<Utc>>,
    pub start_before: Option<DateTime<Utc>>,
    pub visibility: Option<EventVisibility>,
    pub attendee_status: Option<AttendeeStatus>,
    pub has_reminders: Option<bool>,
    pub is_recurring: Option<bool>,
    pub linked_to_chore: Option<bool>,
    pub linked_to_expense: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventConflict {
    pub event_id: Uuid,
    pub conflicting_event_id: Uuid,
    pub conflict_type: ConflictType,
    pub overlap_start: DateTime<Utc>,
    pub overlap_end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConflictType {
    Overlap,    // Events overlap in time
    Duplicate,  // Same event appears multiple times
    Location,   // Same location booked at same time
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteUsers {
    pub user_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespondToEvent {
    pub status: AttendeeStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventReminder {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub minutes_before: u32,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
