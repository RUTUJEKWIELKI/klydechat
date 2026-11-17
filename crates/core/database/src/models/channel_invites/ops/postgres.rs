use crate::Invite;
use crate::PostgresDb;
use bson::Document;
use revolt_result::Result;

use super::AbstractChannelInvites;

static COL: &str = "channel_invites";

/// Helper function for valid invite filtering
fn valid_invite_filter() -> Result<Document> {
    let now = bson::DateTime::now().timestamp_millis();

    Ok(doc! {
        "$and": [
            {
                "$or": [
                    { "expires": null },
                    { "expires": { "$gt": now } },
                ]
            },
            {
                "$or": [
                    { "max_uses": null },
                    { "$expr": { "$lt": ["$uses", "$max_uses"] } },
                ]
            },
        ]
    })
}
#[async_trait]
impl AbstractChannelInvites for PostgresDb {
    /// Insert a new invite into the database
    async fn insert_invite(&self, invite: &Invite) -> Result<()> {
        query!(self, insert_one, COL, &invite).map(|_| ())
    }

    /// Fetch an invite by the code
    async fn fetch_invite(&self, code: &str) -> Result<Invite> {
        let mut filter = doc! { "_id": code };
        filter.extend(valid_invite_filter()?);

        query!(self, find_one, COL, filter)?
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch all invites for a server
    async fn fetch_invites_for_server(&self, server_id: &str) -> Result<Vec<Invite>> {
        let mut filter = doc! { "server": server_id };
        filter.extend(valid_invite_filter()?);

        query!(self, find, COL, filter)
    }

    /// Delete an invite by its code
    async fn delete_invite(&self, code: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, code).map(|_| ())
    }

    /// Atomically consume one use of an invite, returning the invite's state
    /// *after* the increment — or `None` if it was expired, exhausted, or
    /// didn't exist.
    async fn consume_invite_use(&self, code: &str) -> Result<Option<Invite>> {
        let mut filter = doc! { "_id": code };
        filter.extend(valid_invite_filter()?);

        self.find_one_and_update(COL, filter, &doc! { "$inc": { "uses": 1 } })
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("find_one_and_update", COL))
    }

    async fn delete_expired_invites(&self) -> Result<u64> {
        let now = bson::DateTime::now().timestamp_millis();

        query!(
            self,
            delete_many,
            COL,
            doc! {
                "$or": [
                    { "expires": { "$lte": &now } },
                    {
                        "$and": [
                            { "max_uses": { "$ne": null } },
                            { "$expr": { "$gte": ["$uses", "$max_uses"] } },
                        ]
                    },
                ]
            }
        )
    }
}
