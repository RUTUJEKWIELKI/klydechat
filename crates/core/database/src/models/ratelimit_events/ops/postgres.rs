use std::time::{Duration, SystemTime};

use super::AbstractRatelimitEvents;
use crate::{PostgresDb, RatelimitEvent, RatelimitEventType};
use revolt_result::Result;
use ulid::Ulid;

static COL: &str = "ratelimit_events";

#[async_trait]
impl AbstractRatelimitEvents for PostgresDb {
    /// Insert a new ratelimit event
    async fn insert_ratelimit_event(&self, event: &RatelimitEvent) -> Result<()> {
        query!(self, insert_one, COL, &event).map(|_| ())
    }

    /// Count number of events in given duration and check if we've hit the limit
    async fn has_ratelimited(
        &self,
        target_id: &str,
        event_type: RatelimitEventType,
        period: Duration,
        count: usize,
    ) -> Result<bool> {
        query!(
            self,
            count_documents,
            COL,
            doc! {
                "_id": {
                    "$gte": Ulid::from_datetime(SystemTime::now() - period).to_string()
                },
                "target_id": target_id,
                "event_type": event_type.to_string()
            }
        )
        .map(|c| c as usize >= count)
    }
}
