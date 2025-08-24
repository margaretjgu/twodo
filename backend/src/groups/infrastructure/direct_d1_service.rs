use worker::{D1Database, Error as WorkerError};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::groups::domain::group::{
    Group, GroupMember, GroupCreation, GroupUpdate, GroupInfo, GroupInvitation, 
    InviteUser, GroupMemberInfo, MemberRole,
};

pub struct DirectD1GroupService {
    db: D1Database,
}

impl DirectD1GroupService {
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

    pub async fn create_group_from_creation(&self, creation: GroupCreation, created_by: Uuid) -> Result<GroupInfo, WorkerError> {
        let group = Group {
            id: Uuid::new_v4(),
            name: creation.name.clone(),
            description: creation.description.clone(),
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create the group
        self.create_group(&group).await?;

        // Add the creator as owner
        let owner_member = GroupMember {
            group_id: group.id,
            user_id: created_by,
            role: MemberRole::Owner,
            joined_at: Utc::now(),
        };
        self.add_member(&owner_member).await?;

        // Return group info
        Ok(GroupInfo {
            id: group.id,
            name: group.name,
            description: group.description,
            created_by: group.created_by,
            member_count: 1,
            created_at: group.created_at,
            user_role: Some(MemberRole::Owner),
        })
    }

    pub async fn create_group(&self, group: &Group) -> Result<(), WorkerError> {
        let stmt = self.db.prepare("INSERT INTO groups (id, name, description, created_by, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)");
        
        stmt.bind(&[
            group.id.to_string().into(),
            group.name.clone().into(),
            group.description.clone().unwrap_or_default().into(),
            group.created_by.to_string().into(),
            group.created_at.to_rfc3339().into(),
            group.updated_at.to_rfc3339().into(),
        ])?
        .run()
        .await?;

        Ok(())
    }

    pub async fn add_member(&self, member: &GroupMember) -> Result<(), WorkerError> {
        let role_str = match member.role {
            MemberRole::Owner => "admin",  // Map owner to admin for DB constraint
            MemberRole::Admin => "admin", 
            MemberRole::Member => "member",
        };

        let stmt = self.db.prepare("INSERT INTO group_members (group_id, user_id, role, joined_at) VALUES (?1, ?2, ?3, ?4)");
        
        stmt.bind(&[
            member.group_id.to_string().into(),
            member.user_id.to_string().into(),
            role_str.into(),
            member.joined_at.to_rfc3339().into(),
        ])?
        .run()
        .await?;

        Ok(())
    }

    pub async fn get_group_by_id(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Option<GroupInfo>, WorkerError> {
        // Get group basic info
        let group_stmt = self.db.prepare("SELECT * FROM groups WHERE id = ?1");
        let group_result = group_stmt.bind(&[group_id.to_string().into()])?.first::<Value>(None).await?;

        if let Some(group_row) = group_result {
            // Get member count
            let count_stmt = self.db.prepare("SELECT COUNT(*) as count FROM group_members WHERE group_id = ?1");
            let count_result = count_stmt.bind(&[group_id.to_string().into()])?.first::<Value>(None).await?;
            let member_count = count_result
                .and_then(|row| row["count"].as_i64())
                .unwrap_or(0) as usize;

            // Get user's role in this group
            let role = self.get_user_role(group_id, user_id).await?;

            let group_info = GroupInfo {
                id: *group_id,
                name: group_row["name"].as_str().unwrap_or("").to_string(),
                description: Some(group_row["description"].as_str().unwrap_or("").to_string()),
                created_by: Uuid::parse_str(group_row["created_by"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?,
                member_count,
                created_at: DateTime::parse_from_rfc3339(group_row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                user_role: role,
            };

            Ok(Some(group_info))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_role(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Option<MemberRole>, WorkerError> {
        let stmt = self.db.prepare("SELECT role FROM group_members WHERE group_id = ?1 AND user_id = ?2");
        let result = stmt.bind(&[
            group_id.to_string().into(),
            user_id.to_string().into(),
        ])?.first::<Value>(None).await?;

        if let Some(row) = result {
            let role_str = row["role"].as_str().unwrap_or("member");
            let role = match role_str {
                "admin" => MemberRole::Admin,  // Treat admin as admin (could be owner)
                _ => MemberRole::Member,
            };
            Ok(Some(role))
        } else {
            Ok(None)
        }
    }

    pub async fn get_groups_for_user(&self, user_id: &Uuid) -> Result<Vec<GroupInfo>, WorkerError> {
        let stmt = self.db.prepare("
            SELECT g.*, gm.role, 
                   (SELECT COUNT(*) FROM group_members WHERE group_id = g.id) as member_count
            FROM groups g 
            JOIN group_members gm ON g.id = gm.group_id 
            WHERE gm.user_id = ?1
            ORDER BY g.created_at DESC
        ");
        let results = stmt.bind(&[user_id.to_string().into()])?.all().await?;

        let mut groups = Vec::new();
        for row in results.results::<Value>()? {
            let role_str = row["role"].as_str().unwrap_or("member");
            let role = match role_str {
                "admin" => MemberRole::Admin,  // Treat admin as admin (could be owner)
                _ => MemberRole::Member,
            };

            groups.push(GroupInfo {
                id: Uuid::parse_str(row["id"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?,
                name: row["name"].as_str().unwrap_or("").to_string(),
                description: Some(row["description"].as_str().unwrap_or("").to_string()),
                created_by: Uuid::parse_str(row["created_by"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?,
                member_count: row["member_count"].as_i64().unwrap_or(0) as usize,
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
                user_role: Some(role),
            });
        }

        Ok(groups)
    }

    pub async fn get_group_members(&self, group_id: &Uuid) -> Result<Vec<GroupMemberInfo>, WorkerError> {
        let stmt = self.db.prepare("SELECT user_id, role, joined_at FROM group_members WHERE group_id = ?1");
        let results = stmt.bind(&[group_id.to_string().into()])?.all().await?;

        let mut members = Vec::new();
        for row in results.results::<Value>()? {
            let user_id = Uuid::parse_str(row["user_id"].as_str().unwrap_or(""))
                .map_err(|e| WorkerError::RustError(format!("UUID parse error: {}", e)))?;
            let username = self.get_username(&user_id).await.unwrap_or_else(|_| "Unknown User".to_string());
            
            let role_str = row["role"].as_str().unwrap_or("member");
            let role = match role_str {
                "admin" => MemberRole::Admin,  // Treat admin as admin (could be owner)
                _ => MemberRole::Member,
            };

            members.push(GroupMemberInfo {
                user_id,
                username,
                role,
                joined_at: DateTime::parse_from_rfc3339(row["joined_at"].as_str().unwrap_or(""))
                    .map_err(|e| WorkerError::RustError(format!("Date parse error: {}", e)))?
                    .with_timezone(&Utc),
            });
        }

        Ok(members)
    }

    pub async fn invite_user(&self, group_id: &Uuid, invite: InviteUser, invited_by: Uuid) -> Result<(), WorkerError> {
        // For now, directly add the user as a member (simplified invitation system)
        let member = GroupMember {
            group_id: *group_id,
            user_id: invite.user_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
        };

        self.add_member(&member).await
    }

    pub async fn leave_group(&self, group_id: &Uuid, user_id: &Uuid) -> Result<(), WorkerError> {
        let stmt = self.db.prepare("DELETE FROM group_members WHERE group_id = ?1 AND user_id = ?2");
        stmt.bind(&[
            group_id.to_string().into(),
            user_id.to_string().into(),
        ])?.run().await?;

        Ok(())
    }

    pub async fn delete_group(&self, group_id: &Uuid, user_id: &Uuid) -> Result<(), WorkerError> {
        // Verify user is owner
        let role = self.get_user_role(group_id, user_id).await?;
        if !matches!(role, Some(MemberRole::Owner)) {
            return Err(WorkerError::RustError("Only owners can delete groups".to_string()));
        }

        // Delete all members first
        let delete_members_stmt = self.db.prepare("DELETE FROM group_members WHERE group_id = ?1");
        delete_members_stmt.bind(&[group_id.to_string().into()])?.run().await?;

        // Delete group
        let delete_group_stmt = self.db.prepare("DELETE FROM groups WHERE id = ?1");
        delete_group_stmt.bind(&[group_id.to_string().into()])?.run().await?;

        Ok(())
    }
}
