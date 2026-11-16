use revolt_result::Result;

use crate::{AuditLogEntry, AuditLogQuery, PostgresDb};

use super::AbstractAuditLogs;

static COL: &str = "audit_logs";

#[async_trait]
impl AbstractAuditLogs for PostgresDb {
    /// Inserts an entry into the server's audit log
    async fn insert_audit_log_entry(&self, entry: &AuditLogEntry) -> Result<()> {
        query!(self, insert_one, COL, entry).map(|_| ())
    }

    /// Fetches a server's audit logs using the provided query options
    async fn get_server_audit_logs(
        &self,
        server: &str,
        query: AuditLogQuery,
    ) -> Result<Vec<AuditLogEntry>> {
        let mut filter = doc! {
            "server": server
        };

        if let Some(user) = query.user {
            filter.insert("user", user);
        };

        if let Some(target) = query.target {
            filter.insert("target", target);
        }

        if let Some(types) = query.r#type {
            filter.insert("action.type", doc! { "$in": types });
        };

        if let Some(doc) = match (query.before, query.after) {
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
        };

        query!(
            self,
            find_with_options,
            COL,
            filter,
            crate::PostgresFindOptions::new()
                .limit(query.limit)
                .sort(doc! { "_id": -1 })
        )
    }
}
