use revolt_result::Result;

use crate::PostgresDb;
use crate::Report;

use super::AbstractReport;

static COL: &str = "safety_reports";

#[async_trait]
impl AbstractReport for PostgresDb {
    /// Insert a new report into the database
    async fn insert_report(&self, report: &Report) -> Result<()> {
        query!(self, insert_one, COL, &report).map(|_| ())
    }
}
