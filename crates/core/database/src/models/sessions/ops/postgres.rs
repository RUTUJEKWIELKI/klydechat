use crate::{AbstractSessions, PostgresDb, Session};
use bson::to_bson;
use iso8601_timestamp::Timestamp;
use revolt_result::Result;

const COL: &str = "sessions";

#[async_trait]
impl AbstractSessions for PostgresDb {
    /// Find session by id
    async fn fetch_session(&self, id: &str) -> Result<Session> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(UnknownUser))
    }

    /// Find sessions by user id
    async fn fetch_sessions(&self, user_id: &str) -> Result<Vec<Session>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "user_id": user_id
            }
        )
    }

    /// Find sessions by user ids
    async fn fetch_sessions_with_subscription(&self, user_ids: &[String]) -> Result<Vec<Session>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "user_id": {
                    "$in": user_ids
                },
                "subscription": {
                    "$exists": true
                }
            }
        )
    }

    /// Fetch a session from the database by token
    async fn fetch_session_by_token(&self, token: &str) -> Result<Session> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "token": token
            }
        )?
        .ok_or_else(|| create_error!(InvalidSession))
    }

    /// Save session
    async fn save_session(&self, session: &Session) -> Result<()> {
        self.upsert_full(COL, session)
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("upsert_one", COL))
    }

    /// Delete session
    async fn delete_session(&self, id: &str) -> Result<()> {
        query!(
            self,
            delete_one_by_id,
            COL,
            id
        )
        .map(|_| ())
    }

    /// Delete session
    async fn delete_all_sessions(&self, user_id: &str, ignore: Option<String>) -> Result<()> {
        let mut query_doc = doc! {
            "user_id": user_id
        };

        if let Some(id) = ignore {
            query_doc.insert(
                "_id",
                doc! {
                    "$ne": id
                },
            );
        }

        query!(self, delete_many, COL, query_doc).map(|_| ())
    }

    /// Remove push subscription for a session by session id
    async fn remove_push_subscription_by_session_id(&self, session_id: &str) -> Result<()> {
        query!(
            self,
            update_one_raw,
            COL,
            doc! {
                "_id": session_id
            },
            &doc! {
                "$unset": {
                    "subscription": 1
                }
            }
        )
        .map(|_| ())
    }

    async fn update_session_last_seen(&self, session_id: &str, when: Timestamp) -> Result<()> {
        query!(
            self,
            update_one_raw,
            COL,
            doc! {
                "_id": session_id
            },
            &doc! {
                "$set": {
                    "last_seen": to_bson(&when).unwrap()
                }
            }
        )
        .map(|_| ())
    }
}
