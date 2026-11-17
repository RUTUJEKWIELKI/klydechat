use revolt_result::Result;

use crate::{
    FieldsUser, IntoDocumentPath, PartialUser, PostgresDb, RelationshipStatus, User,
};

use super::AbstractUsers;

static COL: &str = "users";

#[async_trait]
impl AbstractUsers for PostgresDb {
    /// Insert a new user into the database
    async fn insert_user(&self, user: &User) -> Result<()> {
        query!(self, insert_one, COL, &user).map(|_| ())
    }

    /// Fetch a user from the database
    async fn fetch_user(&self, id: &str) -> Result<User> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch a user from the database by their username
    async fn fetch_user_by_username(&self, username: &str, discriminator: &str) -> Result<User> {
        self.find_one_where(
            COL,
            "lower(data->>'username') = lower($1) AND data->'discriminator' = $2::jsonb",
            &[
                crate::Bind::Text(username.to_string()),
                crate::Bind::Json(serde_json::Value::String(discriminator.to_string())),
            ],
        )
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("find_one", COL))?
        .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch multiple users by their ids
    async fn fetch_users<'a>(&self, ids: &'a [String]) -> Result<Vec<User>> {
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

    /// Fetch all discriminators in use for a username
    async fn fetch_discriminators_in_use(&self, username: &str) -> Result<Vec<String>> {
        ::sqlx::query_scalar(
            "SELECT data->>'discriminator' FROM users WHERE lower(data->>'username') = lower($1)",
        )
        .bind(username)
        .fetch_all(&self.0)
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("find", COL))
    }

    /// Fetch ids of users that both users are friends with
    async fn fetch_mutual_user_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        Ok(query!(
            self,
            find,
            COL,
            doc! {
                "$and": [
                    { "relations": { "$elemMatch": { "_id": &user_a, "status": "Friend" } } },
                    { "relations": { "$elemMatch": { "_id": &user_b, "status": "Friend" } } }
                ]
            }
        )?
        .into_iter()
        .map(|user: crate::DocumentId| user.id)
        .collect())
    }

    /// Fetch ids of channels that both users are in
    async fn fetch_mutual_channel_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        Ok(query!(
            self,
            find,
            "channels",
            doc! {
                "channel_type": {
                    "$in": ["Group", "DirectMessage"]
                },
                "recipients": {
                    "$all": [ user_a, user_b ]
                }
            }
        )?
        .into_iter()
        .map(|channel: crate::DocumentId| channel.id)
        .collect())
    }

    /// Fetch ids of servers that both users share
    async fn fetch_mutual_server_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>> {
        ::sqlx::query_scalar(
            "SELECT DISTINCT a.data#>>'{_id,server}' \
             FROM server_members a \
             INNER JOIN server_members b \
             ON b.data#>>'{_id,server}' = a.data#>>'{_id,server}' \
             WHERE a.data#>>'{_id,user}' = $1 AND b.data#>>'{_id,user}' = $2",
        )
        .bind(user_a)
        .bind(user_b)
        .fetch_all(&self.0)
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("aggregate", "server_members"))
    }

    /// Update a user by their id given some data
    async fn update_user(
        &self,
        id: &str,
        partial: &PartialUser,
        remove: Vec<FieldsUser>,
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

    /// Set relationship with another user
    ///
    /// This should use pull_relationship if relationship is None.
    async fn set_relationship(
        &self,
        user_id: &str,
        target_id: &str,
        relationship: &RelationshipStatus,
    ) -> Result<()> {
        if let RelationshipStatus::None = relationship {
            return self.pull_relationship(user_id, target_id).await;
        }

        ::sqlx::query(
            "UPDATE users SET data = jsonb_set(data, '{relations}', \
             COALESCE((SELECT jsonb_agg(e) FROM jsonb_array_elements( \
             CASE WHEN jsonb_typeof(data->'relations') = 'array' THEN data->'relations' \
             ELSE '[]'::jsonb END) e WHERE e->'_id' <> to_jsonb($2::text)), '[]'::jsonb) \
             || jsonb_build_array(jsonb_build_object('_id', $2::text, 'status', $3::text))) \
             WHERE id = $1",
        )
        .bind(user_id)
        .bind(target_id)
        .bind(format!("{relationship:?}"))
        .execute(&self.0)
        .await
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", "user"))
    }

    /// Remove relationship with another user
    async fn pull_relationship(&self, user_id: &str, target_id: &str) -> Result<()> {
        query!(
            self,
            update_one_raw,
            COL,
            doc! {
                "_id": user_id
            },
            &doc! {
                "$pull": {
                    "relations": {
                        "_id": target_id
                    }
                }
            }
        )
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", COL))
    }

    /// Delete a user by their id
    async fn delete_user(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }

    /// Removes all relationships with the user from the list of users
    async fn clear_user_relationships(&self, target_id: &str, user_ids: Vec<String>) -> Result<()> {
        query!(
            self,
            update_many_raw,
            COL,
            doc! { "_id": { "$in": user_ids } },
            &doc! {
                "$pull": {
                    "relations": {
                        "_id": target_id.to_string()
                    }
                }
            }
        )
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("bulk_write", COL))
    }
}
