use ::sqlx::Row;
use bson::to_bson;
use revolt_result::Result;

use crate::bson_to_json;
use crate::PostgresDb;
use crate::UserSettings;

use super::AbstractUserSettings;

static COL: &str = "user_settings";

#[async_trait]
impl AbstractUserSettings for PostgresDb {
    /// Fetch a subset of user settings
    async fn fetch_user_settings(&'_ self, id: &str, filter: &'_ [String]) -> Result<UserSettings> {
        let value: serde_json::Value = ::sqlx::query(
            "SELECT data - '_id' AS data FROM user_settings WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.0)
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("find_one", COL))?
        .and_then(|row| row.try_get("data").ok())
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()));

        let mut settings = serde_json::Map::new();
        if let serde_json::Value::Object(map) = value {
            for (key, value) in map {
                if filter.is_empty() || filter.contains(&key) {
                    settings.insert(key, value);
                }
            }
        }

        serde_json::from_value(serde_json::Value::Object(settings))
            .map_err(|_: serde_json::Error| create_database_error!("find_one", COL))
    }

    /// Update a subset of user settings
    async fn set_user_settings(&self, id: &str, settings: &UserSettings) -> Result<()> {
        let mut set = doc! {};
        for (key, data) in settings {
            set.insert(
                key.clone(),
                vec![to_bson(&data.0).unwrap(), to_bson(&data.1).unwrap()],
            );
        }

        let value = bson_to_json(&bson::Bson::Document(set));

        ::sqlx::query(
            "INSERT INTO user_settings (id, data) VALUES ($1, $2) \
             ON CONFLICT (id) DO UPDATE SET data = data || $2::jsonb",
        )
        .bind(id)
        .bind(value)
        .execute(&self.0)
        .await
        .map_err(|_: ::sqlx::Error| create_database_error!("update_one", "user_settings"))?;

        Ok(())
    }

    /// Delete all user settings
    async fn delete_user_settings(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }
}
