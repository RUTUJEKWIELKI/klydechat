use bson::to_bson;
use iso8601_timestamp::Timestamp;
use revolt_result::Result;

use crate::PostgresDb;
use crate::PolicyChange;

use super::AbstractPolicyChange;

static COL: &str = "policy_changes";

#[async_trait]
impl AbstractPolicyChange for PostgresDb {
    /// Fetch all policy changes
    async fn fetch_policy_changes(&self) -> Result<Vec<PolicyChange>> {
        query!(self, find, COL, doc! {})
    }

    /// Acknowledge policy changes
    async fn acknowledge_policy_changes(&self, user_id: &str) -> Result<()> {
        let latest_policy = self
            .fetch_policy_changes()
            .await?
            .into_iter()
            .map(|policy| policy.created_time)
            .max()
            .unwrap_or(Timestamp::UNIX_EPOCH);

        query!(
            self,
            update_one_raw,
            "users",
            doc! {
                "_id": user_id
            },
            &doc! {
                "$set": {
                    "last_acknowledged_policy_change": to_bson(&latest_policy)
                        .map_err(|_: bson::ser::Error| create_database_error!("to_bson", "timestamp"))?
                }
            }
        )
        .map(|_| ())
    }
}
