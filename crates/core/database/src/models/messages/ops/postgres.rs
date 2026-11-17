use ::sqlx::Row;
use bson::{to_bson, Document};
use futures::try_join;
use revolt_models::v0::MessageSort;
use revolt_result::Result;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use ulid::Ulid;

use crate::{
    AppendMessage, DocumentId, FieldsMessage, IntoDocumentPath, Message, MessageQuery,
    MessageTimePeriod, PartialMessage, PostgresDb, PostgresFindOptions,
};

use super::AbstractMessages;

static COL: &str = "messages";

#[async_trait]
impl AbstractMessages for PostgresDb {
    /// Insert a new message into the database
    async fn insert_message(&self, message: &Message) -> Result<()> {
        query!(self, insert_one, COL, &message).map(|_| ())
    }

    /// Fetch a message by its id
    async fn fetch_message(&self, id: &str) -> Result<Message> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch multiple messages by given query
    async fn fetch_messages(&self, query: MessageQuery) -> Result<Vec<Message>> {
        let mut filter = doc! {};

        // 1. Apply message filters
        if let Some(channel) = query.filter.channel {
            filter.insert("channel", channel);
        }

        if let Some(author) = query.filter.author {
            filter.insert("author", author);
        }

        let is_search_query = if let Some(query) = query.filter.query {
            filter.insert(
                "$text",
                doc! {
                    "$search": query
                },
            );

            true
        } else {
            false
        };

        if let Some(pinned) = query.filter.pinned {
            filter.insert("pinned", pinned);
        };

        // 2. Find query limit
        let limit = query.limit.unwrap_or(50);

        // 3. Apply message time period
        match query.time_period {
            MessageTimePeriod::Relative { nearby } => {
                // 3.1. Prepare filters
                let mut older_message_filter = filter.clone();
                let mut newer_message_filter = filter;

                older_message_filter.insert(
                    "_id",
                    doc! {
                        "$lt": &nearby
                    },
                );

                newer_message_filter.insert(
                    "_id",
                    doc! {
                        "$gte": &nearby
                    },
                );

                // 3.2. Execute in both directions
                let (a, b) = try_join!(
                    self.find_with_options::<_, Message>(
                        COL,
                        newer_message_filter,
                        PostgresFindOptions::new()
                            .limit(limit / 2 + 1)
                            .sort(doc! {
                                "_id": 1_i32
                            }),
                    ),
                    self.find_with_options::<_, Message>(
                        COL,
                        older_message_filter,
                        PostgresFindOptions::new()
                            .limit(limit / 2 + 1)
                            .sort(doc! {
                                "_id": -1_i32
                            }),
                    )
                )
                .map_err(|_: ::sqlx::Error| create_database_error!("find", COL))?;

                Ok([a, b].concat())
            }
            MessageTimePeriod::Absolute {
                before,
                after,
                sort,
            } => {
                // 3.1. Apply message ID filter
                if let Some(doc) = match (before, after) {
                    (Some(before), Some(after)) => Some(doc! {
                        "$lt": before,
                        "$gt": after
                    }),
                    (Some(before), _) => Some(doc! {
                        "$lt": before
                    }),
                    (_, Some(after)) => Some(doc! {
                        "$gt": after
                    }),
                    _ => None,
                } {
                    filter.insert("_id", doc);
                }

                // 3.2. Execute with given message sort
                self.find_with_options(
                    COL,
                    filter,
                    PostgresFindOptions::new()
                        .limit(limit)
                        .sort(match sort.unwrap_or(MessageSort::Latest) {
                            // Sort by relevance, fallback to latest
                            MessageSort::Relevance => {
                                if is_search_query {
                                    doc! {
                                        "_id": -1_i32
                                    }
                                } else {
                                    doc! {
                                        "_id": -1_i32
                                    }
                                }
                            }
                            // Sort by latest first
                            MessageSort::Latest => doc! {
                                "_id": -1_i32
                            },
                            // Sort by oldest first
                            MessageSort::Oldest => doc! {
                                "_id": 1_i32
                            },
                        }),
                )
                .await
                .map_err(|_: ::sqlx::Error| create_database_error!("find", COL))
            }
        }
    }

    /// Fetch multiple messages by given IDs
    async fn fetch_messages_by_id(&self, ids: &[String]) -> Result<Vec<Message>> {
        self.find_with_options(
            COL,
            doc! {
                "_id": {
                    "$in": ids
                }
            },
            None,
        )
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("find", COL))
    }

    /// Update a given message with new information
    async fn update_message(
        &self,
        id: &str,
        message: &PartialMessage,
        remove: Vec<FieldsMessage>,
    ) -> Result<()> {
        query!(
            self,
            update_one_by_id,
            COL,
            id,
            message,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None
        )
        .map(|_| ())
    }

    /// Append information to a given message
    async fn append_message(&self, id: &str, append: &AppendMessage) -> Result<()> {
        let mut query_doc = Document::new();

        if let Some(embeds) = &append.embeds {
            if !embeds.is_empty() {
                query_doc.insert(
                    "$push",
                    doc! {
                        "embeds": {
                            "$each": to_bson(embeds)
                                .map_err(|_: bson::ser::Error| create_database_error!("to_bson", "embeds"))?
                        }
                    },
                );
            }
        }

        if query_doc.is_empty() {
            return Ok(());
        }

        self.update_one_raw(
            COL,
            doc! {
                "_id": id
            },
            &query_doc,
        )
        .await
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", COL))
    }

    /// Add a new reaction to a message
    async fn add_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()> {
        self.update_one_raw(
            COL,
            doc! {
                "_id": id
            },
            &doc! {
                "$addToSet": {
                    format!("reactions.{emoji}"): user
                }
            },
        )
        .await
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", COL))
    }

    /// Remove a reaction from a message
    async fn remove_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()> {
        self.update_one_raw(
            COL,
            doc! {
                "_id": id
            },
            &doc! {
                "$pull": {
                    format!("reactions.{emoji}"): user
                }
            },
        )
        .await
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", COL))
    }

    /// Remove reaction from a message
    async fn clear_reaction(&self, id: &str, emoji: &str) -> Result<()> {
        self.update_one_raw(
            COL,
            doc! {
                "_id": id
            },
            &doc! {
                "$unset": {
                    format!("reactions.{emoji}"): 1
                }
            },
        )
        .await
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", COL))
    }

    /// Delete a message from the database by its id
    async fn delete_message(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }

    /// Delete messages from a channel by their ids and corresponding channel id
    async fn delete_messages(&self, channel: &str, ids: &[String]) -> Result<()> {
        query!(
            self,
            delete_many,
            COL,
            doc! {
                "channel": channel,
                "_id": {
                    "$in": ids
                }
            }
        )
        .map(|_| ())
        .map_err(|_: ::sqlx::Error| create_database_error!("delete_many", COL))
    }

    /// Delete all messages from a specific author in a server from a certain ULID onwards
    async fn delete_messages_by_author_since(
        &self,
        channels: &[String],
        author: &str,
        since: SystemTime,
    ) -> Result<HashMap<String, Vec<String>>> {
        let threshold_ulid = Ulid::from_datetime(since).to_string();

        let rows = ::sqlx::query(
            "SELECT data->>'channel' AS channel, \
             jsonb_agg(data->>'_id') AS message_ids, \
             COALESCE(jsonb_agg(COALESCE((SELECT jsonb_agg(a->>'_id') \
             FROM jsonb_array_elements(CASE WHEN jsonb_typeof(data->'attachments') = 'array' \
             THEN data->'attachments' ELSE '[]'::jsonb END) a), '[]'::jsonb)), '[]'::jsonb) AS attachment_ids \
             FROM messages \
             WHERE data->>'author' = $1 AND data->>'channel' = ANY($2) AND id >= $3 \
             GROUP BY data->>'channel'",
        )
        .bind(author)
        .bind(channels.to_vec())
        .bind(threshold_ulid.clone())
        .fetch_all(&self.0)
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("aggregate", COL))?;

        let mut deleted_messages: HashMap<String, Vec<String>> = HashMap::new();
        let mut attachment_ids: HashSet<String> = HashSet::new();

        for row in rows {
            let channel: String = row.try_get("channel").unwrap_or_default();
            let message_ids: serde_json::Value = row.try_get("message_ids").unwrap_or_default();
            let attachments: serde_json::Value =
                row.try_get("attachment_ids").unwrap_or_default();

            deleted_messages.insert(
                channel,
                serde_json::from_value(message_ids).unwrap_or_default(),
            );

            if let serde_json::Value::Array(nested) = attachments {
                for group in nested {
                    if let serde_json::Value::Array(ids) = group {
                        for id in ids {
                            if let serde_json::Value::String(id) = id {
                                attachment_ids.insert(id);
                            }
                        }
                    }
                }
            }
        }

        // Mark attachments as deleted before deleting messages
        if !attachment_ids.is_empty() {
            self.update_many_raw(
                "attachments",
                doc! {
                    "_id": {
                        "$in": attachment_ids.into_iter().collect::<Vec<String>>()
                    }
                },
                &doc! {
                    "$set": {
                        "deleted": true
                    }
                },
            )
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("update_many", "attachments"))?;
        }

        query!(
            self,
            delete_many,
            COL,
            doc! {
                "author": author,
                "channel": { "$in": channels },
                "_id": { "$gte": &threshold_ulid }
            }
        )
        .map_err(|_: ::sqlx::Error| create_database_error!("delete_many", COL))?;

        Ok(deleted_messages)
    }

    async fn delete_messages_by_user(&self, user_id: &str) -> Result<()> {
        self.delete_bulk_messages(doc! {
            "author": user_id,
        })
        .await
    }
}

impl PostgresDb {
    pub async fn delete_bulk_messages(&self, projection: Document) -> Result<()> {
        let mut for_attachments = projection.clone();
        for_attachments.insert(
            "attachments",
            doc! {
                "$exists": 1_i32
            },
        );

        // Check if there are any attachments we need to delete.
        let message_ids_with_attachments = self
            .find_with_options::<_, DocumentId>(
                COL,
                for_attachments,
                None,
            )
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("find_many", "attachments"))?
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<String>>();

        // If we found any, mark them as deleted.
        if !message_ids_with_attachments.is_empty() {
            self.update_many_raw(
                "attachments",
                doc! {
                    "message_id": {
                        "$in": message_ids_with_attachments
                    }
                },
                &doc! {
                    "$set": {
                        "deleted": true
                    }
                },
            )
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("update_many", "attachments"))?;
        }

        // And then delete said messages.
        self.delete_many(COL, projection)
            .await
            .map(|_| ())
            .map_err(|_: ::sqlx::Error| create_database_error!("delete_many", COL))
    }
}
