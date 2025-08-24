use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use crate::calendar::domain::event::{
    Event, EventCreation, EventUpdate, EventInfo, EventFilter, CalendarView, 
    EventAttendee, AttendeeStatus, InviteUsers, RespondToEvent, EventConflict,
    ViewType, DateRange
};
use crate::calendar::domain::ports::{
    EventRepository, EventAttendeeRepository, CalendarViewService, ConflictDetectionService,
    RecurrenceService, ReminderService, EventIntegrationService,
    RecurrenceUpdateScope, RecurrenceDeleteScope
};
use std::error::Error;

pub struct CalendarService {
    event_repository: Arc<dyn EventRepository>,
    attendee_repository: Arc<dyn EventAttendeeRepository>,
    view_service: Arc<dyn CalendarViewService>,
    conflict_service: Arc<dyn ConflictDetectionService>,
    recurrence_service: Arc<dyn RecurrenceService>,
    reminder_service: Arc<dyn ReminderService>,
    integration_service: Arc<dyn EventIntegrationService>,
}

impl CalendarService {
    pub fn new(
        event_repository: Arc<dyn EventRepository>,
        attendee_repository: Arc<dyn EventAttendeeRepository>,
        view_service: Arc<dyn CalendarViewService>,
        conflict_service: Arc<dyn ConflictDetectionService>,
        recurrence_service: Arc<dyn RecurrenceService>,
        reminder_service: Arc<dyn ReminderService>,
        integration_service: Arc<dyn EventIntegrationService>,
    ) -> Self {
        Self {
            event_repository,
            attendee_repository,
            view_service,
            conflict_service,
            recurrence_service,
            reminder_service,
            integration_service,
        }
    }

    pub async fn create_event(&self, creation: EventCreation, created_by: Uuid) -> Result<EventInfo, Box<dyn Error>> {
        // Validate input
        if creation.title.trim().is_empty() {
            return Err("Event title cannot be empty".into());
        }
        if creation.start_time >= creation.end_time {
            return Err("Event end time must be after start time".into());
        }

        let now = Utc::now();
        let event_id = Uuid::new_v4();

        // Create event
        let event = Event {
            id: event_id,
            group_id: creation.group_id,
            title: creation.title.trim().to_string(),
            description: creation.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
            location: creation.location.map(|l| l.trim().to_string()).filter(|l| !l.is_empty()),
            start_time: creation.start_time,
            end_time: creation.end_time,
            is_all_day: creation.is_all_day,
            created_by,
            category: creation.category.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()),
            color: creation.color,
            recurrence: creation.recurrence.clone(),
            reminder_minutes: creation.reminder_minutes.clone(),
            visibility: creation.visibility,
            created_at: now,
            updated_at: now,
        };

        // Check for conflicts
        let conflicts = self.conflict_service.detect_conflicts(&event).await?;
        if !conflicts.is_empty() {
            // In a real app, you might want to return conflicts as warnings rather than errors
            return Err(format!("Event conflicts detected: {} conflicts", conflicts.len()).into());
        }

        self.event_repository.create_event(&event).await?;

        // Add attendees
        let mut attendees = Vec::new();
        
        // Add creator as organizer
        attendees.push(EventAttendee {
            event_id,
            user_id: created_by,
            status: AttendeeStatus::Accepted,
            is_organizer: true,
            invited_at: now,
            responded_at: Some(now),
        });

        // Add other attendees
        for user_id in creation.attendees {
            if user_id != created_by {
                attendees.push(EventAttendee {
                    event_id,
                    user_id,
                    status: AttendeeStatus::Pending,
                    is_organizer: false,
                    invited_at: now,
                    responded_at: None,
                });
            }
        }

        if !attendees.is_empty() {
            self.attendee_repository.add_attendees(&attendees).await?;
        }

        // Handle recurrence
        if event.recurrence.is_some() {
            let _recurring_events = self.recurrence_service.generate_recurring_events(&event, Some(100)).await?;
            // Note: In a full implementation, you'd save these instances
        }

        // Create reminders
        let _reminders = self.reminder_service.create_reminders(&event).await?;

        // Handle integrations
        if let Some(chore_id) = creation.linked_chore_id {
            self.integration_service.link_to_chore(&event_id, &chore_id).await?;
        }
        if let Some(expense_id) = creation.linked_expense_id {
            self.integration_service.link_to_expense(&event_id, &expense_id).await?;
        }

        // Return event info
        self.get_event(&event_id, &created_by).await?.ok_or("Failed to retrieve created event".into())
    }

    pub async fn get_event(&self, event_id: &Uuid, user_id: &Uuid) -> Result<Option<EventInfo>, Box<dyn Error>> {
        // Check if user has access to this event
        if !self.attendee_repository.is_user_invited(event_id, user_id).await? {
            return Ok(None);
        }

        let event = match self.event_repository.get_event_by_id(event_id).await? {
            Some(e) => e,
            None => return Ok(None),
        };

        let attendees = self.attendee_repository.get_event_attendees(event_id).await?;
        let user_status = attendees.iter()
            .find(|a| a.user_id == *user_id)
            .map(|a| a.status.clone());

        let can_edit = event.created_by == *user_id || 
                      attendees.iter().any(|a| a.user_id == *user_id && a.is_organizer);

        Ok(Some(EventInfo {
            id: event.id,
            group_id: event.group_id,
            group_name: "Group".to_string(), // TODO: Lookup group name
            title: event.title,
            description: event.description,
            location: event.location,
            start_time: event.start_time,
            end_time: event.end_time,
            is_all_day: event.is_all_day,
            created_by: event.created_by,
            created_by_name: "User".to_string(), // TODO: Lookup username
            category: event.category,
            color: event.color,
            recurrence: event.recurrence,
            reminder_minutes: event.reminder_minutes,
            visibility: event.visibility,
            attendees,
            user_status,
            can_edit,
            linked_chore_id: None, // TODO: Get from integration service
            linked_expense_id: None, // TODO: Get from integration service
            created_at: event.created_at,
            updated_at: event.updated_at,
        }))
    }

    pub async fn update_event(&self, event_id: &Uuid, user_id: &Uuid, update: EventUpdate) -> Result<(), Box<dyn Error>> {
        // Verify user has permission to update
        let event = self.event_repository.get_event_by_id(event_id).await?
            .ok_or("Event not found")?;
        
        let attendees = self.attendee_repository.get_event_attendees(event_id).await?;
        let can_edit = event.created_by == *user_id || 
                      attendees.iter().any(|a| a.user_id == *user_id && a.is_organizer);
        
        if !can_edit {
            return Err("Insufficient permissions to update event".into());
        }

        // Validate updates
        if let (Some(start), Some(end)) = (&update.start_time, &update.end_time) {
            if start >= end {
                return Err("Event end time must be after start time".into());
            }
        }

        // Handle recurring events
        if event.recurrence.is_some() {
            // In a real implementation, you'd ask the user what scope to update
            self.recurrence_service.update_recurring_series(event_id, &update, RecurrenceUpdateScope::ThisEvent).await?;
        } else {
            self.event_repository.update_event(event_id, &update).await?;
        }

        Ok(())
    }

    pub async fn delete_event(&self, event_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // Verify user has permission
        let event = self.event_repository.get_event_by_id(event_id).await?
            .ok_or("Event not found")?;
        
        if event.created_by != *user_id {
            return Err("Only the event creator can delete the event".into());
        }

        // Handle recurring events
        if event.recurrence.is_some() {
            self.recurrence_service.delete_recurring_series(event_id, RecurrenceDeleteScope::AllEvents).await?;
        } else {
            self.event_repository.delete_event(event_id).await?;
        }

        Ok(())
    }

    pub async fn invite_users(&self, event_id: &Uuid, inviter_id: &Uuid, invite: InviteUsers) -> Result<(), Box<dyn Error>> {
        // Verify user has permission to invite
        let attendees = self.attendee_repository.get_event_attendees(event_id).await?;
        let can_invite = attendees.iter().any(|a| a.user_id == *inviter_id && a.is_organizer);
        
        if !can_invite {
            return Err("Insufficient permissions to invite users".into());
        }

        let now = Utc::now();
        let new_attendees: Vec<EventAttendee> = invite.user_ids.into_iter()
            .filter(|user_id| !attendees.iter().any(|a| a.user_id == *user_id))
            .map(|user_id| EventAttendee {
                event_id: *event_id,
                user_id,
                status: AttendeeStatus::Pending,
                is_organizer: false,
                invited_at: now,
                responded_at: None,
            })
            .collect();

        if !new_attendees.is_empty() {
            self.attendee_repository.add_attendees(&new_attendees).await?;
        }

        Ok(())
    }

    pub async fn respond_to_event(&self, event_id: &Uuid, user_id: &Uuid, response: RespondToEvent) -> Result<(), Box<dyn Error>> {
        // Check if user is invited
        if !self.attendee_repository.is_user_invited(event_id, user_id).await? {
            return Err("User is not invited to this event".into());
        }

        self.attendee_repository.update_attendee_status(event_id, user_id, response.status).await
    }

    pub async fn get_day_view(&self, date: &DateTime<Utc>, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<CalendarView, Box<dyn Error>> {
        self.view_service.get_day_view(date, user_id, group_id).await
    }

    pub async fn get_week_view(&self, date: &DateTime<Utc>, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<CalendarView, Box<dyn Error>> {
        self.view_service.get_week_view(date, user_id, group_id).await
    }

    pub async fn get_month_view(&self, date: &DateTime<Utc>, user_id: &Uuid, group_id: Option<&Uuid>) -> Result<CalendarView, Box<dyn Error>> {
        self.view_service.get_month_view(date, user_id, group_id).await
    }

    pub async fn get_user_events(&self, user_id: &Uuid, start: Option<&DateTime<Utc>>, end: Option<&DateTime<Utc>>) -> Result<Vec<EventInfo>, Box<dyn Error>> {
        self.attendee_repository.get_user_events(user_id, start, end).await
    }

    pub async fn search_events(&self, query: &str, user_id: &Uuid) -> Result<Vec<EventInfo>, Box<dyn Error>> {
        if query.trim().is_empty() {
            return Err("Search query cannot be empty".into());
        }
        self.event_repository.search_events(query, user_id).await
    }

    pub async fn get_conflicts(&self, user_id: &Uuid, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Result<Vec<EventConflict>, Box<dyn Error>> {
        self.conflict_service.get_user_conflicts(user_id, start, end).await
    }

    pub async fn link_to_chore(&self, event_id: &Uuid, chore_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // TODO: Verify user has permission
        self.integration_service.link_to_chore(event_id, chore_id).await
    }

    pub async fn link_to_expense(&self, event_id: &Uuid, expense_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // TODO: Verify user has permission
        self.integration_service.link_to_expense(event_id, expense_id).await
    }

    pub async fn process_reminders(&self) -> Result<(), Box<dyn Error>> {
        // Background task to send pending reminders
        self.reminder_service.send_reminder_notifications().await
    }
}