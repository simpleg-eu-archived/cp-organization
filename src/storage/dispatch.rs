use async_channel::Receiver;

use log::info;
use mongodb::Client;

use crate::storage::{
    actions::{
        invitation_code_action::InvitationCodeAction, member_action::MemberAction,
        organization_action::OrganizationAction, role_action::RoleAction,
    },
    executors::{
        invitation_code_executor::create_invitation_code, member_executor::create_member,
        organization_executor::create_organization, role_executor::get_admin_role_id,
    },
    storage_request::StorageRequest,
};

pub struct Dispatch {
    receiver: Receiver<StorageRequest>,
    client: Client,
}

impl Dispatch {
    pub fn new(receiver: Receiver<StorageRequest>, client: Client) -> Self {
        Self { receiver, client }
    }

    pub async fn run(self) {
        let client = self.client;

        loop {
            let storage_request = match self.receiver.recv().await {
                Ok(storage_request) => storage_request,
                Err(_) => {
                    info!("failed to receive storage request, stopping storage dispatch");
                    break;
                }
            };

            match storage_request {
                StorageRequest::Organization(action) => match action {
                    Some(action) => match action {
                        OrganizationAction::Create {
                            country,
                            name,
                            address,
                            replier,
                        } => {
                            create_organization(client.clone(), country, name, address, replier)
                                .await;
                        }
                    },
                    None => {
                        log::warn!("received empty organization action");
                    }
                },
                StorageRequest::Role(action) => match action {
                    Some(action) => match action {
                        RoleAction::GetAdminRoleId { replier } => {
                            get_admin_role_id(client.clone(), replier).await;
                        }
                    },
                    None => {
                        log::warn!("received empty role action");
                    }
                },
                StorageRequest::Member(action) => match action {
                    Some(action) => match action {
                        MemberAction::Create {
                            user_id,
                            admin_role_id,
                            organization_id,
                            replier,
                        } => {
                            create_member(
                                client.clone(),
                                user_id,
                                admin_role_id,
                                organization_id,
                                replier,
                            )
                            .await;
                        }
                    },
                    None => {
                        log::warn!("received empty member action");
                    }
                },
                StorageRequest::InvitationCode(action) => match action {
                    Some(action) => match action {
                        InvitationCodeAction::Create {
                            code,
                            org_id,
                            permissions,
                            roles,
                            replier,
                        } => {
                            create_invitation_code(
                                client.clone(),
                                code,
                                org_id,
                                permissions,
                                roles,
                                replier,
                            )
                            .await;
                        }
                    },
                    None => {
                        log::warn!("received empty invitation code action");
                    }
                },
            }
        }
    }
}
