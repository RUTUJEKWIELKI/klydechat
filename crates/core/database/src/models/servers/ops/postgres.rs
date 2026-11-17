use bson::{to_document, Bson, Document};
use revolt_result::Result;

use crate::{FieldsRole, FieldsServer, PartialRole, PartialServer, PostgresDb, Role, Server};
use crate::IntoDocumentPath;

use super::AbstractServers;

static COL: &str = "servers";

#[async_trait]
impl AbstractServers for PostgresDb {
    /// Insert a new server into database
    async fn insert_server(&self, server: &Server) -> Result<()> {
        query!(self, insert_one, COL, &server).map(|_| ())
    }

    /// Fetch a server by its id
    async fn fetch_server(&self, id: &str) -> Result<Server> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch a servers by their ids
    async fn fetch_servers<'a>(&self, ids: &'a [String]) -> Result<Vec<Server>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "_id": {
                    "$in": ids
                }
            }
        )
    }

    async fn fetch_owned_servers(&self, user_id: &str) -> Result<Vec<Server>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "owner": user_id
            }
        )
    }

    /// Update a server with new information
    async fn update_server(
        &self,
        id: &str,
        partial: &PartialServer,
        remove: Vec<FieldsServer>,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            id,
            partial,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None
        )
        .map(|_| ())
    }

    /// Delete a server by its id
    async fn delete_server(&self, id: &str) -> Result<()> {
        self.delete_associated_server_objects(id).await?;
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }

    /// Insert a new role into server object
    async fn insert_role(&self, server_id: &str, role: &Role) -> Result<()> {
        query!(
            self,
            update_one_raw,
            COL,
            doc! {
                "_id": server_id
            },
            &doc! {
                "$set": {
                    "roles.".to_owned() + role.id.as_str(): to_document(role)
                        .map_err(|_: bson::ser::Error| create_database_error!("to_document", "role"))?
                }
            }
        )
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", "server"))
    }

    /// Update an existing role on a server
    async fn update_role(
        &self,
        server_id: &str,
        role_id: &str,
        partial: &PartialRole,
        remove: Vec<FieldsRole>,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            server_id,
            partial,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            "roles.".to_owned() + role_id + "."
        )
        .map(|_| ())
    }

    /// Delete a role from a server
    ///
    /// Also updates channels and members.
    async fn delete_role(&self, server_id: &str, role_id: &str) -> Result<()> {
        query!(
            self,
            update_many_raw,
            "server_members",
            doc! {
                "_id.server": server_id
            },
            &doc! {
                "$pull": {
                    "roles": &role_id
                }
            }
        )
        .map_err(|_: ::sqlx::Error| create_database_error!("update_many", "server_members"))?;

        query!(
            self,
            update_many_raw,
            "channels",
            doc! {
                "server": server_id
            },
            &doc! {
                "$unset": {
                    "role_permissions.".to_owned() + role_id: 1_i32
                }
            }
        )
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", "channels"))?;

        query!(
            self,
            update_one_raw,
            COL,
            doc! {
                "_id": server_id
            },
            &doc! {
                "$unset": {
                    "roles.".to_owned() + role_id: 1_i32
                }
            }
        )
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", "servers"))
    }
}

impl PostgresDb {
    pub async fn delete_associated_server_objects(&self, server_id: &str) -> Result<()> {
        // Find all channels
        let channels: Vec<String> = query!(
            self,
            find,
            "channels",
            doc! {
                "server": server_id
            }
        )?
        .into_iter()
        .map(|d: Document| {
            d.get_str("_id")
                .map(|s| s.to_string())
                .ok()
                .unwrap_or_default()
        })
        .collect();

        // Check if there are any attachments we need to delete.
        self.delete_bulk_messages(doc! {
            "channel": {
                "$in": &channels
            }
        })
        .await?;

        // Delete all emoji.
        query!(
            self,
            update_many_raw,
            "emojis",
            doc! {
                "parent.id": &server_id
            },
            &doc! {
                "$set": {
                    "parent": {
                        "type": "Detached"
                    }
                }
            }
        )
        .map_err(|_: ::sqlx::Error| create_database_error!("update_many", "emojis"))?;

        // Delete all channels.
        query!(
            self,
            delete_many,
            "channels",
            doc! {
                "server": &server_id
            }
        )
        .map_err(|_: ::sqlx::Error| create_database_error!("delete_many", "channels"))?;

        // Delete any associated objects, e.g. unreads and invites.
        self.delete_associated_channel_objects(Bson::Document(doc! { "$in": &channels }))
            .await?;

        // Delete members and bans.
        for with in ["server_members", "server_bans"] {
            query!(
                self,
                delete_many,
                with,
                doc! {
                    "_id.server": &server_id
                }
            )
            .map_err(|_: ::sqlx::Error| create_database_error!("delete_many", with))?;
        }

        // Update many attachments with parent id.
        self.delete_many_attachments(doc! {
            "used_for.id": &server_id
        })
        .await?;

        query!(
            self,
            delete_many,
            "audit_logs",
            doc! {
                "server": &server_id
            }
        )
        .map_err(|_: ::sqlx::Error| create_database_error!("delete_many", "audit_logs"))?;

        Ok(())
    }
}
