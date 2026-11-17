use crate::ChannelUnread;
use crate::PostgresDb;
use revolt_result::Result;
use ulid::Ulid;

use super::AbstractChannelUnreads;

static COL: &str = "channel_unreads";

#[async_trait]
impl AbstractChannelUnreads for PostgresDb {
    /// Acknowledge a message, and returns updated channel unread.
    async fn acknowledge_message(
        &self,
        channel_id: &str,
        user_id: &str,
        message_id: &str,
    ) -> Result<Option<ChannelUnread>> {
        let id = serde_json::json!({
            "channel": channel_id,
            "user": user_id
        });

        let base = serde_json::json!({
            "_id": id,
            "last_id": message_id
        });

        self.upsert_raw::<ChannelUnread>(
            COL,
            &id,
            base,
            &doc! {
                "$pull": {
                    "mentions": {
                        "$lte": message_id
                    }
                },
                "$set": {
                    "last_id": message_id
                }
            },
        )
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", COL))
    }

    /// Acknowledge many channels.
    async fn acknowledge_channels(&self, user_id: &str, channel_ids: &[String]) -> Result<()> {
        let current_time = Ulid::new().to_string();

        query!(
            self,
            delete_many,
            COL,
            doc! {
                "_id.channel": {
                    "$in": channel_ids
                },
                "_id.user": user_id
            }
        )?;

        for channel_id in channel_ids {
            let id = serde_json::json!({
                "channel": channel_id,
                "user": user_id
            });

            let base = serde_json::json!({
                "_id": id,
                "last_id": current_time
            });

            self.upsert_raw::<ChannelUnread>(COL, &id, base, &doc! {}).await.map_err(
                |_: ::sqlx::Error| create_database_error!("insert_one", COL),
            )?;
        }

        Ok(())
    }

    /// Add a mention.
    async fn add_mention_to_unread<'a>(
        &self,
        channel_id: &str,
        user_id: &str,
        message_ids: &[String],
    ) -> Result<()> {
        let id = serde_json::json!({
            "channel": channel_id,
            "user": user_id
        });

        let base = serde_json::json!({
            "_id": id,
            "mentions": message_ids
        });

        self.upsert_raw::<ChannelUnread>(
            COL,
            &id,
            base,
            &doc! {
                "$push": {
                    "mentions": {
                        "$each": message_ids
                    }
                }
            },
        )
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", COL))
        .map(|_| ())
    }

    /// Add a mention to multiple users.
    async fn add_mention_to_many_unreads<'a>(
        &self,
        channel_id: &str,
        user_ids: &[String],
        message_ids: &[String],
    ) -> Result<()> {
        for user_id in user_ids {
            self.add_mention_to_unread(channel_id, user_id, message_ids)
                .await?;
        }

        Ok(())
    }

    /// Fetch all channel unreads for a user.
    async fn fetch_unreads(&self, user_id: &str) -> Result<Vec<ChannelUnread>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "_id.user": user_id
            }
        )
    }

    async fn fetch_unread_mentions(&self, user_id: &str) -> Result<Vec<ChannelUnread>> {
        query! {
            self,
            find,
            COL,
            doc! {
                "_id.user": user_id,
                "mentions": {"$ne": null}
            }
        }
    }

    /// Fetch unread for a specific user in a channel.
    async fn fetch_unread(&self, user_id: &str, channel_id: &str) -> Result<Option<ChannelUnread>> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "_id.user": user_id,
                "_id.channel": channel_id
            }
        )
    }
}
