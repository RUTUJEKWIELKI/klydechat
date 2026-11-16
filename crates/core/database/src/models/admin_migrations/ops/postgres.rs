use crate::PostgresDb;

use super::AbstractMigrations;

#[async_trait]
impl AbstractMigrations for PostgresDb {
    #[cfg(test)]
    /// Drop the database
    async fn drop_database(&self) {
        PostgresDb::drop_database(self).await;
    }

    /// Migrate the database
    async fn migrate_database(&self) -> Result<(), ()> {
        info!("Migrating the database.");

        PostgresDb::create_database(self)
            .await
            .map_err(|_| ())?;

        ::sqlx::query(
            "INSERT INTO migrations (id, data) VALUES ('1', '{\"_id\": 1, \"revision\": 0}'::jsonb) \
             ON CONFLICT (id) DO NOTHING",
        )
        .execute(&self.0)
        .await
        .map_err(|_| ())?;

        Ok(())
    }
}
