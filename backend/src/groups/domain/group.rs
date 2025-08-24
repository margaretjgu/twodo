use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid, // User ID who created the group
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupCreation {
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub member_count: usize,
    pub created_at: DateTime<Utc>,
    pub user_role: Option<MemberRole>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupInvitation {
    pub group_id: Uuid,
    pub invited_user_id: Uuid,
    pub invited_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteUser {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupMemberInfo {
    pub user_id: Uuid,
    pub username: String,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}