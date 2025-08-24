use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::event::{
    Event, EventInfo, EventUpdate, EventFilter, CalendarView, DateRange, ViewType,
    EventAttendee, EventAttendeeInfo, AttendeeStatus, EventConflict, EventReminder
};
use std::error::Error;

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn create_event(&self, event: &Event) -> Result<(), Box<dyn Error>>;
    async fn get_event_by_id(&self, event_id: &Uuid) -> Result<Option<Event>, Box<dyn Error>>;
    async fn update_event(&self, event_id: &Uuid, update: &EventUpdate) -> Result<(), Box<dyn Error>>;
    async fn delete_event(&self, event_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn get_events(&self, filter: &EventFilter) -> Result<Vec<EventInfo>, Box<dyn Error>>;
    async fn get_events_in_range(&self, start: &DateTime<Utc>, end: &DateTime<Utc>, group_id: Option<&Uuid>, user_id: &Uuid) -> Result<Vec<EventInfo>, Box<dyn Error>>;
    async fn search_events(&self, query: &str, user_id: &Uuid) -> Result<Vec<EventInfo>, Box<dyn Error>>;
}

#[async_trait]
pub trait EventAttendeeRepository: Send + Sync {
    async fn add_attendees(&self, attendees: &[EventAttendee]) -> Result<(), Box<dyn Error>>;
    async fn update_attendee_status(&self, event_id: &Uuid, user_id: &Uuid, status: AttendeeStatus) -> Result<(), Box<dyn Error>>;
    async fn remove_attendee(&self, event_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn get_event_attendees(&self, event_id: &Uuid) -> Result<Vec<EventAttendeeInfo>, Box<dyn Error>>;
    async fn get_user_events(&self, user_id: &Uuid, start: Option<&DateTime<Utc>>, end: Option<&DateTime<Utc>>) -> Result<Vec<EventInfo>, Box<dyn Error>>;
    async fn is_user_invited(&self, event_id: &Uuid, user_id: &Uuid) -> Result<bool, Box<dyn Error>>;
}

#[async_trait]
pub trait CalendarViewService: Send + Sync {
    async fn get_day_view(&self, date: &DateTime<Utc>, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<CalendarView, Box<dyn Error>>;
    async fn get_week_view(&self, date: &DateTime<Utc>, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<CalendarView, Box<dyn Error>>;
    async fn get_month_view(&self, date: &DateTime<Utc>, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<CalendarView, Box<dyn Error>>;
    async fn get_agenda_view(&self, start: &DateTime<Utc>, end: &DateTime<Utc>, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<CalendarView, Box<dyn Error>>;
}

#[async_trait]
pub trait ConflictDetectionService: Send + Sync {
    async fn detect_conflicts(&self, event: &Event) -> Result<Vec<EventConflict>, Box<dyn Error>>;
    async fn get_user_conflicts(&self, user_id: &Uuid, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Result<Vec<EventConflict>, Box<dyn Error>>;
    async fn get_location_conflicts(&self, location: &str, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Result<Vec<EventConflict>, Box<dyn Error>>;
}

#[async_trait]
pub trait RecurrenceService: Send + Sync {
    async fn generate_recurring_events(&self, base_event: &Event, limit: Option<u32>) -> Result<Vec<Event>, Box<dyn Error>>;
    async fn update_recurring_series(&self, event_id: &Uuid, update: &EventUpdate, update_scope: RecurrenceUpdateScope) -> Result<(), Box<dyn Error>>;
    async fn delete_recurring_series(&self, event_id: &Uuid, delete_scope: RecurrenceDeleteScope) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Clone)]
pub enum RecurrenceUpdateScope {
    ThisEvent,      // Only update this occurrence
    ThisAndFuture,  // Update this and all future occurrences
    AllEvents,      // Update entire series
}

#[derive(Debug, Clone)]
pub enum RecurrenceDeleteScope {
    ThisEvent,      // Only delete this occurrence
    ThisAndFuture,  // Delete this and all future occurrences
    AllEvents,      // Delete entire series
}

#[async_trait]
pub trait ReminderService: Send + Sync {
    async fn create_reminders(&self, event: &Event) -> Result<Vec<EventReminder>, Box<dyn Error>>;
    async fn get_pending_reminders(&self, before: &DateTime<Utc>) -> Result<Vec<EventReminder>, Box<dyn Error>>;
    async fn mark_reminder_sent(&self, reminder_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn send_reminder_notifications(&self) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
pub trait EventIntegrationService: Send + Sync {
    async fn link_to_chore(&self, event_id: &Uuid, chore_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn link_to_expense(&self, event_id: &Uuid, expense_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn get_chore_events(&self, chore_id: &Uuid) -> Result<Vec<EventInfo>, Box<dyn Error>>;
    async fn get_expense_events(&self, expense_id: &Uuid) -> Result<Vec<EventInfo>, Box<dyn Error>>;
}