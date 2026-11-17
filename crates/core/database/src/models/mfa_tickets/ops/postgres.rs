use std::time::{Duration, SystemTime};

use crate::{AbstractMFATickets, MFATicket, PostgresDb};
use iso8601_timestamp::Timestamp;
use revolt_result::Result;
use ulid::Ulid;

const COL: &str = "mfa_tickets";

#[async_trait]
impl AbstractMFATickets for PostgresDb {
    /// Find ticket by token
    ///
    /// Ticket is only valid for 5 minute
    async fn fetch_ticket_by_token(&self, token: &str) -> Result<MFATicket> {
        let ticket: MFATicket = query!(self, find_one, COL, doc! { "token": token })?
            .ok_or_else(|| create_error!(InvalidToken))?;

        if let Ok(ulid) = Ulid::from_string(&ticket.id) {
            if Timestamp::from(ulid.datetime() + Duration::from_mins(5)) > Timestamp::now_utc() {
                Ok(ticket)
            } else {
                Err(create_error!(InvalidToken))
            }
        } else {
            Err(create_error!(InvalidToken))
        }
    }

    /// Save ticket
    async fn save_ticket(&self, ticket: &MFATicket) -> Result<()> {
        self.upsert_full(COL, ticket)
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("upsert_one", COL))
    }

    /// Delete ticket
    async fn delete_ticket(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).and_then(|count| {
            if count == 0 {
                Err(create_error!(InvalidToken))
            } else {
                Ok(())
            }
        })
    }

    /// Delete all expired tickets
    async fn delete_expired_tickets(&self) -> Result<usize> {
        let threshhold =
            Ulid::from_datetime(SystemTime::now() - Duration::from_mins(5)).to_string();

        query!(
            self,
            delete_many,
            COL,
            doc! {
                "_id": { "$lt": threshhold }
            }
        )
        .map(|count| count as usize)
    }
}
