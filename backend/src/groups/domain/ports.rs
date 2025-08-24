use async_trait::async_trait;
use uuid::Uuid;
use super::group::{Group, GroupMember, GroupCreation, GroupUpdate, GroupInfo, GroupInvitation, GroupMemberInfo};
use std::error::Error;

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn create_group(&self, group: &Group) -> Result<(), Box<dyn Error>>;
    async fn get_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>, Box<dyn Error>>;
    async fn update_group(&self, group_id: &Uuid, update: &GroupUpdate) -> Result<(), Box<dyn Error>>;
    async fn delete_group(&self, group_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn get_groups_for_user(&self, user_id: &Uuid) -> Result<Vec<GroupInfo>, Box<dyn Error>>;
}

#[async_trait]
pub trait GroupMemberRepository: Send + Sync {
    async fn add_member(&self, member: &GroupMember) -> Result<(), Box<dyn Error>>;
    async fn remove_member(&self, group_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn get_members(&self, group_id: &Uuid) -> Result<Vec<GroupMemberInfo>, Box<dyn Error>>;
    async fn is_member(&self, group_id: &Uuid, user_id: &Uuid) -> Result<bool, Box<dyn Error>>;
    async fn get_user_role(&self, group_id: &Uuid, user_id: &Uuid) -> Result<Option<super::group::MemberRole>, Box<dyn Error>>;
}

#[async_trait]
pub trait GroupInvitationRepository: Send + Sync {
    async fn create_invitation(&self, invitation: &GroupInvitation) -> Result<(), Box<dyn Error>>;
    async fn get_pending_invitations(&self, user_id: &Uuid) -> Result<Vec<GroupInvitation>, Box<dyn Error>>;
    async fn accept_invitation(&self, group_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>>;
    async fn decline_invitation(&self, group_id: &Uuid, user_id: &Uuid) -> Result<(), Box<dyn Error>>;
}