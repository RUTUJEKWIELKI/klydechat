use revolt_result::Result;

use crate::PostgresDb;
use crate::Snapshot;

use super::AbstractSnapshot;

static COL: &str = "safety_snapshots";

#[async_trait]
impl AbstractSnapshot for PostgresDb {
    /// Insert a new snapshot into the database
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        query!(self, insert_one, COL, &snapshot).map(|_| ())
    }
}
