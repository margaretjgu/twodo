use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::groups::domain::group::{Group, GroupMember, GroupCreation, GroupUpdate, GroupInfo, GroupInvitation, InviteUser, MemberRole, GroupMemberInfo};
use crate::groups::domain::ports::{GroupRepository, GroupMemberRepository, GroupInvitationRepository};
use std::error::Error;

pub struct GroupService {
    group_repository: Arc<dyn GroupRepository>,
    member_repository: Arc<dyn GroupMemberRepository>,
    invitation_repository: Arc<dyn GroupInvitationRepository>,
}

impl GroupService {
    pub fn new(
        group_repository: Arc<dyn GroupRepository>,
        member_repository: Arc<dyn GroupMemberRepository>,
        invitation_repository: Arc<dyn GroupInvitationRepository>,
    ) -> Self {
        Self {
            group_repository,
            member_repository,
            invitation_repository,
        }
    }

    pub async fn create_group(&self, creation: GroupCreation) -> Result<GroupInfo, Box<dyn Error>> {
        // Validate input
        if creation.name.trim().is_empty() {
            return Err("Group name cannot be empty".into());
        }
        if creation.name.len() > 100 {
            return Err("Group name cannot exceed 100 characters".into());
        }

        let now = Utc::now();
        let group_id = Uuid::new_v4();

        // Create group
        let group = Group {
            id: group_id,
            name: creation.name.trim().to_string(),
            description: creation.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
            created_by: creation.created_by,
            created_at: now,
            updated_at: now,
        };

        self.group_repository.create_group(&group).await?;

        // Add creator as owner
        let owner_member = GroupMember {
            group_id,
            user_id: creation.created_by,
            role: MemberRole::Owner,
            joined_at: now,
        };

        self.member_repository.add_member(&owner_member).await?;

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

    pub async fn get_group(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Option<GroupInfo>, Box<dyn Error>> {
        // Check if user is a member
        if !self.member_repository.is_member(group_id, user_id).await? {
            return Ok(None);
        }

        let group = match self.group_repository.get_group_by_id(group_id).await? {
            Some(g) => g,
            None => return Ok(None),
        };

        let members = self.member_repository.get_members(group_id).await?;
        let user_role = self.member_repository.get_user_role(group_id, user_id).await?;

        Ok(Some(GroupInfo {
            id: group.id,
            name: group.name,
            description: group.description,
            created_by: group.created_by,
            member_count: members.len(),
            created_at: group.created_at,
            user_role,
        }))
    }

    pub async fn get_user_groups(&self, user_id: &Uuid) -> Result<Vec<GroupInfo>, Box<dyn Error>> {
        self.group_repository.get_groups_for_user(user_id).await
    }

    pub async fn update_group(&self, group_id: &Uuid, user_id: &Uuid, update: GroupUpdate) -> Result<(), Box<dyn Error>> {
        // Check if user has permission (owner or admin)
        let user_role = self.member_repository.get_user_role(group_id, user_id).await?;
        match user_role {
            Some(MemberRole::Owner) | Some(MemberRole::Admin) => {},
            _ => return Err("Insufficient permissions to update group".into()),
        }

        // Validate updates
        if let Some(ref name) = update.name {
            if name.trim().is_empty() {
                return Err("Group name cannot be empty".into());
            }
            if name.len() > 100 {
                return Err("Group name cannot exceed 100 characters".into());
            }
        }

        self.group_repository.update_group(group_id, &update).await
    }

    pub async fn invite_user(&self, group_id: &Uuid, inviter_id: &Uuid, invite: InviteUser) -> Result<(), Box<dyn Error>> {
        // Check if inviter has permission (owner or admin)
        let user_role = self.member_repository.get_user_role(group_id, inviter_id).await?;
        match user_role {
            Some(MemberRole::Owner) | Some(MemberRole::Admin) => {},
            _ => return Err("Insufficient permissions to invite users".into()),
        }

        // Check if user is already a member
        if self.member_repository.is_member(group_id, &invite.user_id).await? {
            return Err("User is already a member of this group".into());
        }

        let invitation = GroupInvitation {
            group_id: *group_id,
            invited_user_id: invite.user_id,
            invited_by: *inviter_id,
            created_at: Utc::now(),
        };

        self.invitation_repository.create_invitation(&invitation).await
    }

    pub async fn accept_invitation(&self, group_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // Remove invitation and add as member
        self.invitation_repository.accept_invitation(group_id, user_id).await?;

        let member = GroupMember {
            group_id: *group_id,
            user_id: *user_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
        };

        self.member_repository.add_member(&member).await
    }

    pub async fn decline_invitation(&self, group_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>> {
        self.invitation_repository.decline_invitation(group_id, user_id).await
    }

    pub async fn remove_member(&self, group_id: &Uuid, remover_id: &Uuid, member_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // Check permissions
        let remover_role = self.member_repository.get_user_role(group_id, remover_id).await?;
        let member_role = self.member_repository.get_user_role(group_id, member_id).await?;

        // Can't remove yourself if you're the owner
        if remover_id == member_id && matches!(remover_role, Some(MemberRole::Owner)) {
            return Err("Owner cannot remove themselves from the group".into());
        }

        // Only owners can remove admins, owners and admins can remove members
        match (remover_role, member_role) {
            (Some(MemberRole::Owner), _) => {},
            (Some(MemberRole::Admin), Some(MemberRole::Member)) => {},
            (Some(MemberRole::Member), _) if remover_id == member_id => {}, // Members can remove themselves
            _ => return Err("Insufficient permissions to remove this member".into()),
        }

        self.member_repository.remove_member(group_id, member_id).await
    }

    pub async fn get_group_members(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Vec<GroupMemberInfo>, Box<dyn Error>> {
        // Check if user is a member
        if !self.member_repository.is_member(group_id, user_id).await? {
            return Err("Not authorized to view group members".into());
        }

        self.member_repository.get_members(group_id).await
    }

    pub async fn get_pending_invitations(&self, user_id: &Uuid) -> Result<Vec<GroupInvitation>, Box<dyn Error>> {
        self.invitation_repository.get_pending_invitations(user_id).await
    }
}