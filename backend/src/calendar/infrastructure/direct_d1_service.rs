use worker::{D1Database, Error as WorkerError};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::calendar::domain::event::{
    Event, EventInfo, EventCreation, EventVisibility, AttendeeStatus, EventAttendeeInfo,
};

pub struct DirectD1CalendarService {
    db: D1Database,
}

impl DirectD1CalendarService {
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

    pub async fn create_event_from_creation(&self, creation: EventCreation, created_by: Uuid) -> Result<EventInfo, WorkerError> {
        let event = Event {
            id: Uuid::new_v4(),
            group_id: creation.group_id,
            title: creation.title.clone(),
            description: creation.description.clone(),
            location: creation.location.clone(),
            start_time: creation.start_time,
            end_time: creation.end_time,
            is_all_day: creation.is_all_day,
            created_by,
            category: creation.category.clone(),
            color: creation.color.clone(),
            recurrence: None, // Simplified - no recurrence for now
            reminder_minutes: creation.reminder_minutes,
            visibility: creation.visibility,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the event
        self.create_event(&event).await?;

        // Add attendees if provided
        for attendee_id in &creation.attendees {
            self.add_attendee(&event.id, attendee_id).await?;
        }

        // Return event info
        let created_by_name = self.get_username(&created_by).await.unwrap_or_else(|_| "Unknown User".to_string());
        let group_name = self.get_group_name(&event.group_id).await.unwrap_or_else(|_| "Unknown Group".to_string());
        
        Ok(EventInfo {
            id: event.id,
            group_id: event.group_id,
            group_name,
            title: event.title,
            description: event.description,
            location: event.location,
            start_time: event.start_time,
            end_time: event.end_time,
            is_all_day: event.is_all_day,
            created_by,
            created_by_name,
            category: event.category,
            color: event.color,
            recurrence: event.recurrence,
            reminder_minutes: event.reminder_minutes,
            visibility: event.visibility,
            attendees: vec![], // Simplified for now
            user_status: None, // Simplified for now
            can_edit: true, // Simplified for now
            linked_chore_id: None, // Simplified for now
            linked_expense_id: None, // Simplified for now
            created_at: event.created_at,
            updated_at: event.updated_at,
        })
    }

    pub async fn create_event(&self, event: &Event) -> Result<(), WorkerError> {
        let visibility_str = match event.visibility {
            EventVisibility::Public => "public",
            EventVisibility::Private => "private",
            EventVisibility::Confidential => "confidential",
        };

        let stmt = self.db.prepare("INSERT INTO events (id, group_id, title, description, start_time, end_time, location, created_by, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)");
        
        stmt.bind(&[
            event.id.to_string().into(),
            event.group_id.to_string().into(),
            event.title.clone().into(),
            event.description.clone().unwrap_or_default().into(),
            event.start_time.to_rfc3339().into(),
            event.end_time.to_rfc3339().into(),
            event.location.clone().unwrap_or_default().into(),
            event.created_by.to_string().into(),
            event.created_at.to_rfc3339().into(),
            event.updated_at.to_rfc3339().into(),
        ])?
        .run()
        .await?;

        Ok(())
    }

    pub async fn add_attendee(&self, event_id: &Uuid, user_id: &Uuid) -> Result<(), WorkerError> {
        let stmt = self.db.prepare("INSERT INTO event_attendees (event_id, user_id, status, responded_at) VALUES (?1, ?2, ?3, ?4)");
        
        stmt.bind(&[
            event_id.to_string().into(),
            user_id.to_string().into(),
            "pending".into(),
            "".into(), // Empty string for NULL
        ])?
        .run()
        .await?;

        Ok(())
    }

    pub async fn get_event_by_id(&self, event_id: &Uuid, user_id: &Uuid) -> Result<Option<EventInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT * FROM events WHERE id = ?1");
        let result = stmt.bind(&[event_id.to_string().into()])?.first::<Value>(None).await?;

        if let Some(row) = result {
            let created_by = Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let group_id = Uuid::parse_str(row["group_id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;

            let created_by_name = self.get_username(&created_by).await.unwrap_or_else(|_| "Unknown User".to_string());
            let group_name = self.get_group_name(&group_id).await.unwrap_or_else(|_| "Unknown Group".to_string());
            let attendees = self.get_event_attendee_info(event_id).await?;
            let user_status = self.get_user_event_status(event_id, user_id).await?;
            let can_edit = created_by == *user_id;

            let event_info = EventInfo {
                id: *event_id,
                group_id,
                group_name,
                title: row["title"].as_str().unwrap_or("").to_string(),
                description: Some(row["description"].as_str().unwrap_or("").to_string()),
                location: Some(row["location"].as_str().unwrap_or("").to_string()),
                start_time: DateTime::parse_from_rfc3339(row["start_time"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                end_time: DateTime::parse_from_rfc3339(row["end_time"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                is_all_day: false, // Simplified
                created_by,
                created_by_name,
                category: None, // Simplified
                color: None, // Simplified
                recurrence: None,
                reminder_minutes: vec![],
                visibility: EventVisibility::Public,
                attendees,
                user_status,
                can_edit,
                linked_chore_id: None,
                linked_expense_id: None,
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row["updated_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
            };

            Ok(Some(event_info))
        } else {
            Ok(None)
        }
    }

    pub async fn get_event_attendees(&self, event_id: &Uuid) -> Result<Vec<Uuid>, WorkerError> {
        let stmt = self.db.prepare("SELECT user_id FROM event_attendees WHERE event_id = ?1");
        let results = stmt.bind(&[event_id.to_string().into()])?.all().await?;

        let mut attendees = Vec::new();
        for row in results.results::<Value>()? {
            if let Ok(user_id) = Uuid::parse_str(row["user_id"].as_str().unwrap_or("")) {
                attendees.push(user_id);
            }
        }

        Ok(attendees)
    }

    pub async fn get_event_attendee_info(&self, event_id: &Uuid) -> Result<Vec<EventAttendeeInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT ea.user_id, ea.status, ea.responded_at FROM event_attendees ea WHERE ea.event_id = ?1");
        let results = stmt.bind(&[event_id.to_string().into()])?.all().await?;

        let mut attendees = Vec::new();
        for row in results.results::<Value>()? {
            if let Ok(user_id) = Uuid::parse_str(row["user_id"].as_str().unwrap_or("")) {
                let username = self.get_username(&user_id).await.unwrap_or_else(|_| "Unknown User".to_string());
                let status = match row["status"].as_str().unwrap_or("pending") {
                    "accepted" => AttendeeStatus::Accepted,
                    "declined" => AttendeeStatus::Declined,
                    "tentative" => AttendeeStatus::Tentative,
                    _ => AttendeeStatus::Pending,
                };
                let responded_at = if let Some(time_str) = row["responded_at"].as_str() {
                    if !time_str.is_empty() {
                        Some(DateTime::parse_from_rfc3339(time_str)
                            .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                            .with_timezone(&Utc))
                    } else {
                        None
                    }
                } else {
                    None
                };

                attendees.push(EventAttendeeInfo {
                    user_id,
                    username,
                    status,
                    is_organizer: false, // We'll determine this separately
                    responded_at,
                });
            }
        }

        Ok(attendees)
    }

    pub async fn get_user_event_status(&self, event_id: &Uuid, user_id: &Uuid) -> Result<Option<AttendeeStatus>, WorkerError> {
        let stmt = self.db.prepare("SELECT status FROM event_attendees WHERE event_id = ?1 AND user_id = ?2");
        let results = stmt.bind(&[event_id.to_string().into(), user_id.to_string().into()])?.all().await?;

        if let Some(row) = results.results::<Value>()?.into_iter().next() {
            let status = match row["status"].as_str().unwrap_or("pending") {
                "accepted" => AttendeeStatus::Accepted,
                "declined" => AttendeeStatus::Declined,
                "tentative" => AttendeeStatus::Tentative,
                _ => AttendeeStatus::Pending,
            };
            Ok(Some(status))
        } else {
            Ok(None)
        }
    }

    pub async fn get_group_events(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Vec<EventInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT * FROM events WHERE group_id = ?1 ORDER BY start_time ASC");
        let results = stmt.bind(&[group_id.to_string().into()])?.all().await?;

        let mut events = Vec::new();
        for row in results.results::<Value>()? {
            let event_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let created_by = Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;

            let created_by_name = self.get_username(&created_by).await.unwrap_or_else(|_| "Unknown User".to_string());
            let group_name = self.get_group_name(group_id).await.unwrap_or_else(|_| "Unknown Group".to_string());
            let attendees = self.get_event_attendees(&event_id).await?;

            events.push(EventInfo {
                id: event_id,
                group_id: *group_id,
                group_name,
                title: row["title"].as_str().unwrap_or("").to_string(),
                description: Some(row["description"].as_str().unwrap_or("").to_string()),
                location: Some(row["location"].as_str().unwrap_or("").to_string()),
                start_time: DateTime::parse_from_rfc3339(row["start_time"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                end_time: DateTime::parse_from_rfc3339(row["end_time"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                is_all_day: false,
                created_by,
                created_by_name,
                category: None,
                color: None,
                recurrence: None,
                reminder_minutes: vec![],
                visibility: EventVisibility::Public,
                attendees: vec![], // Simplified for now
                user_status: None,
                can_edit: true,
                linked_chore_id: None,
                linked_expense_id: None,
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row["updated_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
            });
        }

        Ok(events)
    }

    pub async fn delete_event(&self, event_id: &Uuid, user_id: &Uuid) -> Result<(), WorkerError> {
        // For now, allow anyone to delete (in production, should check permissions)
        
        // Delete attendees first
        let delete_attendees_stmt = self.db.prepare("DELETE FROM event_attendees WHERE event_id = ?1");
        delete_attendees_stmt.bind(&[event_id.to_string().into()])?.run().await?;

        // Delete event
        let delete_event_stmt = self.db.prepare("DELETE FROM events WHERE id = ?1");
        delete_event_stmt.bind(&[event_id.to_string().into()])?.run().await?;

        Ok(())
    }

    pub async fn get_events_in_date_range(&self, group_id: &Uuid, start_date: &DateTime<Utc>, end_date: &DateTime<Utc>, _user_id: &Uuid) -> Result<Vec<EventInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT * FROM events WHERE group_id = ?1 AND start_time >= ?2 AND start_time <= ?3 ORDER BY start_time ASC");
        let results = stmt.bind(&[
            group_id.to_string().into(),
            start_date.to_rfc3339().into(),
            end_date.to_rfc3339().into(),
        ])?.all().await?;

        let mut events = Vec::new();
        for row in results.results::<Value>()? {
            let event_id = Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let created_by = Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;

            let created_by_name = self.get_username(&created_by).await.unwrap_or_else(|_| "Unknown User".to_string());
            let group_name = self.get_group_name(group_id).await.unwrap_or_else(|_| "Unknown Group".to_string());
            let attendees = self.get_event_attendees(&event_id).await?;

            events.push(EventInfo {
                id: event_id,
                group_id: *group_id,
                group_name,
                title: row["title"].as_str().unwrap_or("").to_string(),
                description: Some(row["description"].as_str().unwrap_or("").to_string()),
                location: Some(row["location"].as_str().unwrap_or("").to_string()),
                start_time: DateTime::parse_from_rfc3339(row["start_time"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                end_time: DateTime::parse_from_rfc3339(row["end_time"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                is_all_day: false,
                created_by,
                created_by_name,
                category: None,
                color: None,
                recurrence: None,
                reminder_minutes: vec![],
                visibility: EventVisibility::Public,
                attendees: vec![], // Simplified for now
                user_status: None,
                can_edit: true,
                linked_chore_id: None,
                linked_expense_id: None,
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(row["updated_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
            });
        }

        Ok(events)
    }
}
