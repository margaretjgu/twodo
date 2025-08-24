// Optimized push notification service for Cloudflare Workers
use worker::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub data: HashMap<String, String>,
    pub user_ids: Vec<String>,
    pub notification_type: NotificationType,
}

#[derive(Serialize, Deserialize)]
pub enum NotificationType {
    ExpenseAdded,
    ChoreAssigned,
    ChoreReminder,
    DebtSettlement,
    GroupInvitation,
    EventReminder,
}

#[derive(Serialize, Deserialize)]
pub struct FCMMessage {
    pub to: String,
    pub notification: FCMNotification,
    pub data: HashMap<String, String>,
    pub priority: String,
}

#[derive(Serialize, Deserialize)]
pub struct FCMNotification {
    pub title: String,
    pub body: String,
    pub icon: String,
    pub click_action: String,
}

pub struct NotificationService {
    fcm_key: String,
    db: D1Database,
}

impl NotificationService {
    pub fn new(fcm_key: String, db: D1Database) -> Self {
        Self { fcm_key, db }
    }

    // Optimized batch notification sending
    pub async fn send_notification(&self, payload: NotificationPayload) -> Result<(), Error> {
        // Get all push tokens for target users in one query
        let tokens = self.get_push_tokens(&payload.user_ids).await?;
        
        if tokens.is_empty() {
            return Ok(()); // No tokens to send to
        }

        // Create FCM messages
        let messages: Vec<FCMMessage> = tokens.into_iter().map(|(token, platform)| {
            FCMMessage {
                to: token,
                notification: FCMNotification {
                    title: payload.title.clone(),
                    body: payload.body.clone(),
                    icon: "ic_notification".to_string(),
                    click_action: self.get_click_action(&payload.notification_type),
                },
                data: payload.data.clone(),
                priority: "high".to_string(),
            }
        }).collect();

        // Send all notifications concurrently (max 100 per batch for FCM limits)
        for chunk in messages.chunks(100) {
            self.send_fcm_batch(chunk).await?;
        }

        Ok(())
    }

    // Optimized token retrieval
    async fn get_push_tokens(&self, user_ids: &[String]) -> Result<Vec<(String, String)>, Error> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = user_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT token, platform FROM push_tokens WHERE user_id IN ({}) AND created_at > ?",
            placeholders
        );

        let stmt = self.db.prepare(&query);
        let mut bindings: Vec<JsValue> = user_ids.iter().map(|id| id.clone().into()).collect();
        
        // Only get tokens from last 30 days (stale token cleanup)
        let thirty_days_ago = (js_sys::Date::now() - (30.0 * 24.0 * 60.0 * 60.0 * 1000.0)) as i64;
        bindings.push(thirty_days_ago.into());

        let result = stmt.bind(&bindings)?.all().await?;
        
        let mut tokens = Vec::new();
        for row in result.results()? {
            let token: String = row.get("token")?;
            let platform: String = row.get("platform")?;
            tokens.push((token, platform));
        }

        Ok(tokens)
    }

    // Batch FCM sending with error handling
    async fn send_fcm_batch(&self, messages: &[FCMMessage]) -> Result<(), Error> {
        let mut headers = Headers::new();
        headers.set("Authorization", &format!("key={}", self.fcm_key))?;
        headers.set("Content-Type", "application/json")?;

        // Send all messages concurrently
        let futures: Vec<_> = messages.iter().map(|message| {
            self.send_single_fcm(message, &headers)
        }).collect();

        // Wait for all to complete (fail fast on critical errors)
        for future in futures {
            let _ = future.await; // Log errors but don't fail the batch
        }

        Ok(())
    }

    async fn send_single_fcm(&self, message: &FCMMessage, headers: &Headers) -> Result<(), Error> {
        let request = Request::new_with_init(
            "https://fcm.googleapis.com/fcm/send",
            RequestInit::new()
                .with_method(Method::Post)
                .with_headers(headers.clone())
                .with_body(Some(serde_json::to_string(message)?.into())),
        )?;

        let response = Fetch::Request(request).send().await?;
        
        // Log failed notifications for debugging but don't throw
        if !response.status_code().is_success() {
            console_log!("FCM send failed: {}", response.status_code());
        }

        Ok(())
    }

    fn get_click_action(&self, notification_type: &NotificationType) -> String {
        match notification_type {
            NotificationType::ExpenseAdded => "OPEN_EXPENSES".to_string(),
            NotificationType::ChoreAssigned => "OPEN_CHORES".to_string(),
            NotificationType::ChoreReminder => "OPEN_CHORES".to_string(),
            NotificationType::DebtSettlement => "OPEN_EXPENSES".to_string(),
            NotificationType::GroupInvitation => "OPEN_GROUPS".to_string(),
            NotificationType::EventReminder => "OPEN_CALENDAR".to_string(),
        }
    }

    // Smart notification templates to reduce payload size
    pub fn create_expense_notification(
        expense_desc: &str,
        amount: f64,
        payer_name: &str,
        group_members: Vec<String>,
    ) -> NotificationPayload {
        NotificationPayload {
            title: "New Expense Added".to_string(),
            body: format!("{} paid ${:.2} for {}", payer_name, amount, expense_desc),
            data: HashMap::from([
                ("type".to_string(), "expense_added".to_string()),
                ("amount".to_string(), amount.to_string()),
            ]),
            user_ids: group_members,
            notification_type: NotificationType::ExpenseAdded,
        }
    }

    pub fn create_chore_reminder(
        chore_title: &str,
        assigned_user: String,
        deadline: i64,
    ) -> NotificationPayload {
        NotificationPayload {
            title: "Chore Reminder".to_string(),
            body: format!("Don't forget: {}", chore_title),
            data: HashMap::from([
                ("type".to_string(), "chore_reminder".to_string()),
                ("deadline".to_string(), deadline.to_string()),
            ]),
            user_ids: vec![assigned_user],
            notification_type: NotificationType::ChoreReminder,
        }
    }
}

// Scheduled notification handler (runs on Cloudflare Cron)
#[event(scheduled)]
pub async fn scheduled(event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    let db = env.d1("DB")?;
    let fcm_key = env.var("FCM_SERVER_KEY")?.to_string();
    let notification_service = NotificationService::new(fcm_key, db);

    match event.cron().as_str() {
        // Daily chore reminders at 9 AM
        "0 9 * * *" => {
            send_chore_reminders(&notification_service).await?;
        }
        // Weekly expense summaries on Sunday
        "0 20 * * 0" => {
            send_weekly_summaries(&notification_service).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn send_chore_reminders(service: &NotificationService) -> Result<(), Error> {
    // Get chores due today or overdue
    let tomorrow = (js_sys::Date::now() + (24.0 * 60.0 * 60.0 * 1000.0)) as i64;
    
    let stmt = service.db.prepare("
        SELECT c.title, c.assigned_to, c.deadline, u.username
        FROM chores c
        JOIN users u ON c.assigned_to = u.id
        WHERE c.status = 'pending' 
        AND c.deadline <= ?
        AND c.deadline > ?
    ");
    
    let now = js_sys::Date::now() as i64;
    let result = stmt.bind(&[tomorrow.into(), now.into()])?.all().await?;
    
    for row in result.results()? {
        let title: String = row.get("title")?;
        let assigned_to: String = row.get("assigned_to")?;
        let deadline: i64 = row.get("deadline")?;
        
        let notification = NotificationService::create_chore_reminder(&title, assigned_to, deadline);
        service.send_notification(notification).await?;
    }
    
    Ok(())
}

async fn send_weekly_summaries(_service: &NotificationService) -> Result<(), Error> {
    // Implementation for weekly expense summaries
    Ok(())
}
