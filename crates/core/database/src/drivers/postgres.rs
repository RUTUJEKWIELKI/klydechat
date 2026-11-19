use std::ops::Deref;

use base64::Engine;
use bson::{Bson, Document};
use revolt_result::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value as Json;

use crate::IntoDocumentPath;

use ::sqlx::Row;

database_derived!(
    /// PostgreSQL implementation
    #[derive(Debug)]
    pub struct PostgresDb(pub ::sqlx::PgPool);
);

impl Deref for PostgresDb {
    type Target = ::sqlx::PgPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Options for PostgreSQL find operations
#[derive(Debug, Default, Clone)]
pub struct PostgresFindOptions {
    /// Sort specification, a document mapping field names to 1 (asc) or -1 (desc)
    pub sort: Option<Document>,
    /// Maximum number of documents to return
    pub limit: Option<i64>,
    /// Number of documents to skip
    pub skip: Option<u64>,
}

impl PostgresFindOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sort(mut self, sort: Document) -> Self {
        self.sort = Some(sort);
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn skip(mut self, skip: u64) -> Self {
        self.skip = Some(skip);
        self
    }
}

/// A bind value for a parameterised PostgreSQL query
#[derive(Clone, Debug)]
pub enum Bind {
    Json(Json),
    Text(String),
    TextArray(Vec<String>),
    Int(i64),
}

#[allow(dead_code)]
impl PostgresDb {
    /// Run the schema definition and index creation statements
    pub async fn create_database(&self) -> Result<(), ::sqlx::Error> {
        let tables = ALL_COLLECTIONS;

        for table in tables {
            ::sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS \"{}\" (id TEXT PRIMARY KEY, data JSONB NOT NULL DEFAULT '{{}}'::jsonb)",
                table
            ))
            .execute(&self.0)
            .await?;
        }

        let indexes: &[(&str, &str)] = &[
            (
                "messages",
                "CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages ((data->>'channel'), id)",
            ),
            (
                "messages",
                "CREATE INDEX IF NOT EXISTS idx_messages_author ON messages ((data->>'author'))",
            ),
            (
                "messages",
                "CREATE INDEX IF NOT EXISTS idx_messages_channel_pinned ON messages ((data->>'channel'), (data->>'pinned'))",
            ),
            (
                "messages",
                "CREATE INDEX IF NOT EXISTS idx_messages_content_ts ON messages USING GIN (to_tsvector('simple', data->>'content'))",
            ),
            (
                "channel_unreads",
                "CREATE INDEX IF NOT EXISTS idx_channel_unreads_user ON channel_unreads ((data#>>'{_id,user}'))",
            ),
            (
                "channel_unreads",
                "CREATE INDEX IF NOT EXISTS idx_channel_unreads_compound ON channel_unreads ((data#>>'{_id,channel}'), (data#>>'{_id,user}'))",
            ),
            (
                "server_members",
                "CREATE INDEX IF NOT EXISTS idx_server_members_server ON server_members ((data#>>'{_id,server}'))",
            ),
            (
                "server_members",
                "CREATE INDEX IF NOT EXISTS idx_server_members_user ON server_members ((data#>>'{_id,user}'))",
            ),
            (
                "server_bans",
                "CREATE INDEX IF NOT EXISTS idx_server_bans_server ON server_bans ((data#>>'{_id,server}'))",
            ),
            (
                "attachments",
                "CREATE INDEX IF NOT EXISTS idx_attachments_hash ON attachments ((data->>'hash'))",
            ),
            (
                "attachments",
                "CREATE INDEX IF NOT EXISTS idx_attachments_used_for ON attachments ((data#>>'{used_for,id}'))",
            ),
            (
                "attachments",
                "CREATE INDEX IF NOT EXISTS idx_attachments_message ON attachments ((data->>'message_id'))",
            ),
            (
                "attachment_hashes",
                "CREATE INDEX IF NOT EXISTS idx_attachment_hashes_processed ON attachment_hashes ((data->>'processed_hash'))",
            ),
            (
                "users",
                "CREATE INDEX IF NOT EXISTS idx_users_username ON users (lower(data->>'username'))",
            ),
            (
                "sessions",
                "CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions ((data->>'user_id'))",
            ),
            (
                "sessions",
                "CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions ((data->>'token'))",
            ),
            (
                "accounts",
                "CREATE INDEX IF NOT EXISTS idx_accounts_email ON accounts (lower(data->>'email_normalised'))",
            ),
            (
                "emojis",
                "CREATE INDEX IF NOT EXISTS idx_emojis_parent ON emojis ((data#>>'{parent,id}'))",
            ),
            (
                "servers",
                "CREATE INDEX IF NOT EXISTS idx_servers_owner ON servers ((data->>'owner'))",
            ),
            (
                "bots",
                "CREATE INDEX IF NOT EXISTS idx_bots_owner ON bots ((data->>'owner'))",
            ),
            (
                "audit_logs",
                "CREATE INDEX IF NOT EXISTS idx_audit_logs_server ON audit_logs ((data->>'server'))",
            ),
            (
                "channel_invites",
                "CREATE INDEX IF NOT EXISTS idx_channel_invites_server ON channel_invites ((data->>'server'))",
            ),
            (
                "ratelimit_events",
                "CREATE INDEX IF NOT EXISTS idx_ratelimit_events_target ON ratelimit_events ((data->>'target_id'))",
            ),
            (
                "channel_webhooks",
                "CREATE INDEX IF NOT EXISTS idx_channel_webhooks_channel ON channel_webhooks ((data->>'channel_id'))",
            ),
        ];

        for (table, statement) in indexes {
            if !tables.contains(table) {
                continue;
            }

            ::sqlx::query(statement).execute(&self.0).await?;
        }

        Ok(())
    }

    /// Drop all tables (used for testing)
    pub async fn drop_database(&self) {
        for table in ALL_COLLECTIONS {
            ::sqlx::query(&format!("DROP TABLE IF EXISTS \"{}\" CASCADE", table))
                .execute(&self.0)
                .await
                .ok();
        }
    }

    /// Insert one document into a collection
    pub async fn insert_one<T: Serialize + Send + Sync>(
        &self,
        collection: &'static str,
        document: T,
    ) -> Result<(), ::sqlx::Error> {
        let value = to_json_value(&document)?;
        let id = extract_id(&value)?;

        ::sqlx::query(&format!(
            "INSERT INTO \"{}\" (id, data) VALUES ($1, $2)",
            collection
        ))
        .bind(id)
        .bind(value)
        .execute(&self.0)
        .await
        .map(|_| ())
    }

    /// Insert many documents into a collection
    pub async fn insert_many<T: Serialize + Send + Sync>(
        &self,
        collection: &'static str,
        documents: Vec<T>,
    ) -> Result<(), ::sqlx::Error> {
        for document in documents {
            self.insert_one(collection, document).await?;
        }

        Ok(())
    }

    /// Count documents by projection
    pub async fn count_documents(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<u64, ::sqlx::Error> {
        let mut binds = Vec::new();
        let condition = translate_filter(&projection, "data", &mut binds)?;

        let sql = format!("SELECT COUNT(*)::bigint FROM \"{}\" WHERE {}", collection, condition);
        let row = bind_query(::sqlx::query(&sql), &binds)
            .fetch_one(&self.0)
            .await?;

        Ok(row.get::<i64, _>(0) as u64)
    }

    /// Find multiple documents in a collection with options
    pub async fn find_with_options<O, T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
        options: O,
    ) -> Result<Vec<T>, ::sqlx::Error>
    where
        O: Into<Option<PostgresFindOptions>>,
    {
        let options = options.into().unwrap_or_default();
        let mut binds = Vec::new();
        let condition = translate_filter(&projection, "data", &mut binds)?;

        let mut sql = format!("SELECT data FROM \"{}\" WHERE {}", collection, condition);
        sql.push_str(&build_order_by(options.sort.as_ref()));
        if let Some(limit) = options.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(skip) = options.skip {
            sql.push_str(&format!(" OFFSET {}", skip));
        }

        let rows = bind_query(::sqlx::query(&sql), &binds)
            .fetch_all(&self.0)
            .await?;

        rows.into_iter().map(decode_row).collect()
    }

    /// Find multiple documents in a collection
    pub async fn find<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<Vec<T>, ::sqlx::Error> {
        self.find_with_options(collection, projection, None).await
    }

    /// Find documents with a raw SQL condition and binds
    pub async fn find_where<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        condition: &str,
        binds: &[Bind],
    ) -> Result<Vec<T>, ::sqlx::Error> {
        let sql = format!("SELECT data FROM \"{}\" WHERE {}", collection, condition);
        let rows = bind_query(::sqlx::query(&sql), binds)
            .fetch_all(&self.0)
            .await?;

        rows.into_iter().map(decode_row).collect()
    }

    /// Find one document with a raw SQL condition and binds
    pub async fn find_one_where<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        condition: &str,
        binds: &[Bind],
    ) -> Result<Option<T>, ::sqlx::Error> {
        let sql = format!(
            "SELECT data FROM \"{}\" WHERE {} LIMIT 1",
            collection, condition
        );
        let row = bind_query(::sqlx::query(&sql), binds)
            .fetch_optional(&self.0)
            .await?;

        row.map(decode_row).transpose()
    }

    /// Find one document with case-insensitive text matching on the given field
    pub async fn find_one_ci<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        field: &str,
        value: &str,
    ) -> Result<Option<T>, ::sqlx::Error> {
        self.find_one_where(
            collection,
            &format!("lower(data#>>'{{{}}}') = lower($1)", field),
            &[Bind::Text(value.to_string())],
        )
        .await
    }

    /// Find one document with options
    pub async fn find_one_with_options<O, T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
        options: O,
    ) -> Result<Option<T>, ::sqlx::Error>
    where
        O: Into<Option<PostgresFindOptions>>,
    {
        let options = options.into().unwrap_or_default();
        let mut binds = Vec::new();
        let condition = translate_filter(&projection, "data", &mut binds)?;

        let mut sql = format!("SELECT data FROM \"{}\" WHERE {}", collection, condition);
        sql.push_str(&build_order_by(options.sort.as_ref()));
        if let Some(skip) = options.skip {
            sql.push_str(&format!(" OFFSET {}", skip));
        }
        sql.push_str(" LIMIT 1");

        let row = bind_query(::sqlx::query(&sql), &binds)
            .fetch_optional(&self.0)
            .await?;

        row.map(decode_row).transpose()
    }

    /// Find one document
    pub async fn find_one<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<Option<T>, ::sqlx::Error> {
        self.find_one_with_options(collection, projection, None)
            .await
    }

    /// Find one document by its ID
    pub async fn find_one_by_id<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        id: &str,
    ) -> Result<Option<T>, ::sqlx::Error> {
        let row = ::sqlx::query(&format!(
            "SELECT data FROM \"{}\" WHERE id = $1",
            collection
        ))
        .bind(id)
        .fetch_optional(&self.0)
        .await?;

        row.map(decode_row).transpose()
    }

    /// Update documents matching the given filter with a MongoDB-style update document
    pub async fn update_one_raw(
        &self,
        collection: &'static str,
        projection: Document,
        update: &Document,
    ) -> Result<(), ::sqlx::Error> {
        self.update_many_raw(collection, projection, update).await
    }

    /// Update all documents matching the given filter with a MongoDB-style update document
    pub async fn update_many_raw(
        &self,
        collection: &'static str,
        projection: Document,
        update: &Document,
    ) -> Result<(), ::sqlx::Error> {
        let (set, mut binds) = translate_update(update, "data")?;

        if set.is_none() {
            return Ok(());
        }

        let condition = translate_filter(&projection, "data", &mut binds)?;

        let sql = format!(
            "UPDATE \"{}\" SET data = {} WHERE {}",
            collection,
            set.unwrap(),
            condition
        );

        bind_query(::sqlx::query(&sql), &binds)
        .execute(&self.0)
        .await
        .map(|_| ())
    }

    /// Update one document given a projection, partial document, and list of paths to unset
    pub async fn update_one<P, T: Serialize>(
        &self,
        collection: &'static str,
        projection: Document,
        partial: T,
        remove: Vec<&dyn IntoDocumentPath>,
        prefix: P,
    ) -> Result<(), ::sqlx::Error>
    where
        P: Into<Option<String>>,
    {
        let prefix = prefix.into();

        let mut paths = Vec::new();
        for field in remove {
            if let Some(path) = field.as_path() {
                if let Some(prefix) = &prefix {
                    paths.push(prefix.to_owned() + path);
                } else {
                    paths.push(path.to_string());
                }
            }
        }

        let value = to_json_value(&partial)?;
        let set: Vec<(String, Json)> = if let Some(prefix) = &prefix {
            match value {
                Json::Object(map) => map
                    .into_iter()
                    .map(|(k, v)| (format!("{}{}", prefix, k), v))
                    .collect(),
                _ => return Err(db_err("partial update must be an object")),
            }
        } else {
            match value {
                Json::Object(map) => map.into_iter().collect(),
                _ => return Err(db_err("partial update must be an object")),
            }
        };

        let (expression, mut binds) = build_partial_update(&set, &paths);
        if expression.is_none() {
            return Ok(());
        }

        let condition = translate_filter(&projection, "data", &mut binds)?;

        bind_query(
            ::sqlx::query(&format!(
                "UPDATE \"{}\" SET data = {} WHERE {}",
                collection,
                expression.unwrap(),
                condition
            )),
            &binds,
        )
        .execute(&self.0)
        .await
        .map(|_| ())
    }

    /// Update one document given an ID, partial document, and list of paths to unset
    pub async fn update_one_by_id<P, T: Serialize>(
        &self,
        collection: &'static str,
        id: &str,
        partial: T,
        remove: Vec<&dyn IntoDocumentPath>,
        prefix: P,
    ) -> Result<(), ::sqlx::Error>
    where
        P: Into<Option<String>>,
    {
        self.update_one(
            collection,
            bson::doc! {
                "_id": id
            },
            partial,
            remove,
            prefix,
        )
        .await
    }

    /// Upsert a full document by its ID
    pub async fn upsert_full<T: Serialize + Send + Sync>(
        &self,
        collection: &'static str,
        document: T,
    ) -> Result<(), ::sqlx::Error> {
        let value = to_json_value(&document)?;
        let id = extract_id(&value)?;

        ::sqlx::query(&format!(
            "INSERT INTO \"{}\" (id, data) VALUES ($1, $2) \
             ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data",
            collection
        ))
        .bind(id)
        .bind(value)
        .execute(&self.0)
        .await
        .map(|_| ())
    }

    /// Upsert a document given its ID value, base document, and a MongoDB-style update document.
    /// Returns the document state after the update.
    pub async fn upsert_raw<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        id: &Json,
        base: Json,
        update: &Document,
    ) -> Result<Option<T>, ::sqlx::Error> {
        let id_text = serde_json::to_string(id).map_err(|_| db_err("failed to serialise id"))?;
        let (set, binds) =
            translate_update(update, &format!("\"{}\".data", collection))?;
        let set = set
            .map(|set| renumber_placeholders(&set, 2))
            .unwrap_or_else(|| "EXCLUDED.data".to_string());

        let sql = format!(
            "INSERT INTO \"{}\" (id, data) VALUES ($1, $2) \
             ON CONFLICT (id) DO UPDATE SET data = {} \
             RETURNING data",
            collection, set
        );

        let row = bind_query(::sqlx::query(&sql).bind(id_text).bind(base), &binds)
            .fetch_optional(&self.0)
            .await?;

        row.map(decode_row).transpose()
    }

    /// Find one document by the given filter and update it with a MongoDB-style update document.
    /// Returns the document state after the update.
    pub async fn find_one_and_update<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
        update: &Document,
    ) -> Result<Option<T>, ::sqlx::Error> {
        let (set, mut binds) = translate_update(update, "data")?;

        if set.is_none() {
            // No-op update, just fetch the document
            return self.find_one(collection, projection).await;
        }

        let condition = translate_filter(&projection, "data", &mut binds)?;

        let row = bind_query(
            ::sqlx::query(&format!(
                "UPDATE \"{}\" SET data = {} WHERE {} RETURNING data",
                collection,
                set.unwrap(),
                condition
            )),
            &binds,
        )
        .fetch_optional(&self.0)
        .await?;

        row.map(decode_row).transpose()
    }

    /// Delete one document by the given projection, returning the number of deleted documents
    pub async fn delete_one(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<u64, ::sqlx::Error> {
        let mut binds = Vec::new();
        let condition = translate_filter(&projection, "data", &mut binds)?;

        bind_query(
            ::sqlx::query(&format!(
                "DELETE FROM \"{}\" WHERE {}",
                collection, condition
            )),
            &binds,
        )
        .execute(&self.0)
        .await
        .map(|result| result.rows_affected())
    }

    /// Delete one document by the given ID
    pub async fn delete_one_by_id(
        &self,
        collection: &'static str,
        id: &str,
    ) -> Result<u64, ::sqlx::Error> {
        ::sqlx::query(&format!("DELETE FROM \"{}\" WHERE id = $1", collection))
            .bind(id)
            .execute(&self.0)
            .await
            .map(|result| result.rows_affected())
    }

    /// Delete all documents matching the given projection, returning the number of deleted documents
    pub async fn delete_many(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<u64, ::sqlx::Error> {
        self.delete_one(collection, projection).await
    }
}

/// All collections used across the codebase
pub const ALL_COLLECTIONS: &[&str] = &[
    "account_invites",
    "accounts",
    "attachment_hashes",
    "attachments",
    "audit_logs",
    "bots",
    "channel_invites",
    "channel_unreads",
    "channel_webhooks",
    "channels",
    "discover_bans",
    "discover_requests",
    "emojis",
    "messages",
    "mfa_tickets",
    "migrations",
    "policy_changes",
    "pubsub",
    "ratelimit_events",
    "safety_reports",
    "safety_snapshots",
    "safety_strikes",
    "server_bans",
    "server_members",
    "servers",
    "sessions",
    "user_settings",
    "users",
    "webhooks",
];

/// Convert an arbitrary serialisable value into JSON
pub fn to_json_value<T: Serialize>(value: &T) -> Result<Json, ::sqlx::Error> {
    serde_json::to_value(value).map_err(|_| db_err("failed to serialise document"))
}

/// Extract the document ID from a serialised document
pub fn extract_id(value: &Json) -> Result<String, ::sqlx::Error> {
    match value.get("_id") {
        Some(Json::String(id)) => Ok(id.clone()),
        Some(other) => serde_json::to_string(other).map_err(|_| db_err("failed to serialise id")),
        None => Err(db_err("document is missing an `_id` field")),
    }
}

/// Build a MongoDB-style partial update expression from set values and unset paths
pub fn build_partial_update(
    set: &[(String, Json)],
    unset: &[String],
) -> (Option<String>, Vec<Bind>) {
    let mut binds = Vec::new();
    let mut expression = "data".to_string();

    for path in unset {
        expression = format!("{} #- '{}'", expression, to_path_array(path));
    }

    for (path, value) in set {
        binds.push(Bind::Json(value.clone()));
        expression = format!(
            "jsonb_set({}, '{}', ${})",
            ensure_parents(&expression, path),
            to_path_array(path),
            binds.len()
        );
    }

    if binds.is_empty() && unset.is_empty() {
        (None, binds)
    } else {
        (Some(expression), binds)
    }
}

/// Translate a MongoDB update document into a PostgreSQL SET expression
pub fn translate_update(
    update: &Document,
    root: &str,
) -> Result<(Option<String>, Vec<Bind>), ::sqlx::Error> {
    let mut binds = Vec::new();
    let mut expression = root.to_string();

    // $set
    if let Some(Bson::Document(doc)) = update.get("$set") {
        for (path, value) in doc {
            binds.push(Bind::Json(bson_to_json(value)));
            expression = format!(
                "jsonb_set({}, '{}', ${})",
                ensure_parents(&expression, path),
                to_path_array(path),
                binds.len()
            );
        }
    }

    // $unset
    if let Some(Bson::Document(doc)) = update.get("$unset") {
        for (path, _) in doc {
            expression = format!("{} #- '{}'", expression, to_path_array(path));
        }
    }

    // $push
    if let Some(Bson::Document(doc)) = update.get("$push") {
        for (path, value) in doc {
            match value {
                Bson::Document(inner) if inner.contains_key("$each") => {
                    let each = inner.get("$each").expect("checked above");
                    binds.push(Bind::Json(bson_to_json(each)));
                    let base = ensure_parents(&expression, path);
                    expression = format!(
                        "jsonb_set({}, '{}', COALESCE(NULLIF({} #> '{}', 'null'::jsonb), '[]'::jsonb) || ${})",
                        base,
                        to_path_array(path),
                        base,
                        to_path_array(path),
                        binds.len()
                    );
                }
                _ => {
                    binds.push(Bind::Json(bson_to_json(value)));
                    let base = ensure_parents(&expression, path);
                    expression = format!(
                        "jsonb_set({}, '{}', COALESCE(NULLIF({} #> '{}', 'null'::jsonb), '[]'::jsonb) || jsonb_build_array(${}))",
                        base,
                        to_path_array(path),
                        base,
                        to_path_array(path),
                        binds.len()
                    );
                }
            }
        }
    }

    // $addToSet
    if let Some(Bson::Document(doc)) = update.get("$addToSet") {
        for (path, value) in doc {
            match value {
                Bson::Document(inner) if inner.contains_key("$each") => {
                    let each = inner.get("$each").expect("checked above");
                    binds.push(Bind::Json(bson_to_json(each)));
                    let base = ensure_parents(&expression, path);
                    expression = format!(
                        "jsonb_set({}, '{}', \
                         CASE WHEN COALESCE(NULLIF({} #> '{}', 'null'::jsonb), '[]'::jsonb) @> ${} \
                         THEN {} #> '{}' \
                         ELSE COALESCE(NULLIF({} #> '{}', 'null'::jsonb), '[]'::jsonb) || ${} END)",
                        base,
                        to_path_array(path),
                        base,
                        to_path_array(path),
                        binds.len(),
                        base,
                        to_path_array(path),
                        base,
                        to_path_array(path),
                        binds.len()
                    );
                }
                _ => {
                    binds.push(Bind::Json(bson_to_json(value)));
                    let base = ensure_parents(&expression, path);
                    expression = format!(
                        "jsonb_set({}, '{}', \
                         CASE WHEN COALESCE(NULLIF({} #> '{}', 'null'::jsonb), '[]'::jsonb) @> jsonb_build_array(${}) \
                         THEN {} #> '{}' \
                         ELSE COALESCE(NULLIF({} #> '{}', 'null'::jsonb), '[]'::jsonb) || jsonb_build_array(${}) END)",
                        base,
                        to_path_array(path),
                        base,
                        to_path_array(path),
                        binds.len(),
                        base,
                        to_path_array(path),
                        base,
                        to_path_array(path),
                        binds.len()
                    );
                }
            }
        }
    }

    // $pull
    if let Some(Bson::Document(doc)) = update.get("$pull") {
        for (path, condition) in doc {
            let predicate = translate_pull_condition(condition, &mut binds)?;
            let base = ensure_parents(&expression, path);
            expression = format!(
                "jsonb_set({}, '{}', \
                 COALESCE((SELECT jsonb_agg(e) FROM jsonb_array_elements( \
                 CASE WHEN jsonb_typeof({} #> '{}') = 'array' THEN {} #> '{}' ELSE '[]'::jsonb END) e \
                 WHERE NOT ({})), '[]'::jsonb))",
                base,
                to_path_array(path),
                expression,
                to_path_array(path),
                expression,
                to_path_array(path),
                predicate
            );
        }
    }

    // $inc
    if let Some(Bson::Document(doc)) = update.get("$inc") {
        for (path, value) in doc {
            binds.push(Bind::Json(bson_to_json(value)));
            let base = ensure_parents(&expression, path);
            expression = format!(
                "jsonb_set({}, '{}', to_jsonb( \
                 COALESCE(CASE WHEN jsonb_typeof({} #> '{}') = 'number' \
                 THEN ({} #> '{}')::numeric END, 0) + (${}::jsonb)::numeric))",
                base,
                to_path_array(path),
                expression,
                to_path_array(path),
                expression,
                to_path_array(path),
                binds.len()
            );
        }
    }

    if binds.is_empty() && expression == root {
        return Ok((None, binds));
    }

    Ok((Some(expression), binds))
}

/// Translate a `$pull` condition into a predicate over array elements aliased as `e`
fn translate_pull_condition(condition: &Bson, binds: &mut Vec<Bind>) -> Result<String, ::sqlx::Error> {
    match condition {
        Bson::Document(doc) if doc.keys().any(|k| k.starts_with('$')) => {
            let mut clauses = Vec::new();
            for (op, value) in doc {
                clauses.push(translate_operator_condition("e", "e", false, op, value, binds)?);
            }
            Ok(clauses.join(" AND "))
        }
        Bson::Document(doc) => {
            let mut clauses = Vec::new();
            for (key, value) in doc {
                let (expression, text_expression, is_column) = path_expressions("e", key);

                if let Bson::Document(operators) = value {
                    if operators.keys().any(|k| k.starts_with('$')) {
                        for (op, operand) in operators {
                            clauses.push(translate_operator_condition(
                                &expression,
                                &text_expression,
                                is_column,
                                op,
                                operand,
                                binds,
                            )?);
                        }

                        continue;
                    }
                }

                clauses.push(translate_equality(&expression, is_column, value, binds)?);
            }

            if clauses.is_empty() {
                Ok("TRUE".to_string())
            } else {
                Ok(clauses.join(" AND "))
            }
        }
        _ => {
            binds.push(Bind::Json(bson_to_json(condition)));
            Ok(format!("e = ${}", binds.len()))
        }
    }
}

/// Wrap a base expression so that every parent object along `path` exists
///
/// `jsonb_set` does not create intermediate objects, so parents missing from
/// the stored document must be initialised before setting a nested value.
fn ensure_parents(base: &str, path: &str) -> String {
    let segments: Vec<&str> = path.split('.').collect();
    let mut expr = base.to_string();

    for i in 1..segments.len() {
        let parent = to_path_array(&segments[..i].join("."));
        expr = format!(
            "jsonb_set({}, '{}', COALESCE(NULLIF(({}) #> '{}', 'null'::jsonb), '{{}}'::jsonb))",
            expr, parent, base, parent
        );
    }

    expr
}

/// Shift `$N` placeholders in a SQL fragment by the given offset
fn renumber_placeholders(sql: &str, offset: usize) -> String {
    let mut out = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            let digits: String = chars.clone().take_while(|c| c.is_ascii_digit()).collect();

            if !digits.is_empty() {
                for _ in 0..digits.len() {
                    chars.next();
                }

                let n: usize = digits.parse().unwrap();
                out.push_str(&format!("${}", n + offset));
                continue;
            }
        }

        out.push(c);
    }

    out
}

/// Convert a dotted MongoDB path into a PostgreSQL text array literal for path operators
fn to_path_array(path: &str) -> String {
    let segments: Vec<String> = path
        .split('.')
        .map(|segment| segment.replace(",", "\\,"))
        .collect();

    format!("{{{}}}", segments.join(","))
}

/// Build an ORDER BY clause from a MongoDB-style sort document
fn build_order_by(sort: Option<&Document>) -> String {
    let Some(sort) = sort else {
        return String::new();
    };

    let mut clauses = Vec::new();
    for (field, direction) in sort {
        if field == "score" {
            continue;
        }

        let direction = if direction.as_i32().unwrap_or(1) < 0 || direction.as_i64().unwrap_or(1) < 0
        {
            "DESC"
        } else {
            "ASC"
        };

        if field == "_id" {
            clauses.push(format!("id {}", direction));
        } else {
            let segments: Vec<String> = field.split('.').map(|s| s.to_string()).collect();
            clauses.push(format!(
                "data#>>'{{{}}}' {}",
                segments.join(","),
                direction
            ));
        }
    }

    if clauses.is_empty() {
        String::new()
    } else {
        format!(" ORDER BY {}", clauses.join(", "))
    }
}

/// Translate a MongoDB filter document into a SQL condition and binds
pub fn translate_filter(
    filter: &Document,
    root: &str,
    binds: &mut Vec<Bind>,
) -> Result<String, ::sqlx::Error> {
    translate_filter_inner(filter, root, binds)
}

fn translate_filter_inner(
    filter: &Document,
    root: &str,
    binds: &mut Vec<Bind>,
) -> Result<String, ::sqlx::Error> {
    let mut clauses = Vec::new();

    for (key, value) in filter {
        match key.as_str() {
            "$or" => {
                let Bson::Array(branches) = value else {
                    return Err(db_err("$or must be an array"));
                };

                let mut parts = Vec::new();
                for branch in branches {
                    let Bson::Document(doc) = branch else {
                        return Err(db_err("$or branches must be documents"));
                    };

                    parts.push(format!("({})", translate_filter_inner(doc, root, binds)?));
                }

                clauses.push(format!("({})", parts.join(" OR ")));
            }
            "$and" => {
                let Bson::Array(branches) = value else {
                    return Err(db_err("$and must be an array"));
                };

                let mut parts = Vec::new();
                for branch in branches {
                    let Bson::Document(doc) = branch else {
                        return Err(db_err("$and branches must be documents"));
                    };

                    parts.push(format!("({})", translate_filter_inner(doc, root, binds)?));
                }

                clauses.push(format!("({})", parts.join(" AND ")));
            }
            "$text" => {
                let Bson::Document(doc) = value else {
                    return Err(db_err("$text must be a document"));
                };

                let search = doc
                    .get("$search")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| db_err("$text requires a $search string"))?;

                binds.push(Bind::Text(search.to_string()));
                clauses.push(format!(
                    "to_tsvector('simple', {} #>> '{{content}}') @@ plainto_tsquery('simple', ${})",
                    root,
                    binds.len()
                ));
            }
            "$expr" => {
                clauses.push(translate_expr(value, root, binds)?);
            }
            _ => {
                clauses.push(translate_key_condition(root, key, value, binds)?);
            }
        }
    }

    if clauses.is_empty() {
        Ok("TRUE".to_string())
    } else {
        Ok(clauses.join(" AND "))
    }
}

/// Translate a condition on a single document key
fn translate_key_condition(
    root: &str,
    key: &str,
    value: &Bson,
    binds: &mut Vec<Bind>,
) -> Result<String, ::sqlx::Error> {
    let (expression, text_expression, is_column) = path_expressions(root, key);

    match value {
        Bson::Document(doc) if doc.keys().any(|k| k.starts_with('$')) => {
            let mut clauses = Vec::new();
            for (op, operand) in doc {
                clauses.push(translate_operator_condition(
                    &expression,
                    &text_expression,
                    is_column,
                    op,
                    operand,
                    binds,
                )?);
            }
            Ok(clauses.join(" AND "))
        }
        _ => translate_equality(&expression, is_column, value, binds),
    }
}

/// Build the JSONB and text extraction expressions for a dotted path
fn path_expressions(root: &str, path: &str) -> (String, String, bool) {
    if root == "data" && path == "_id" {
        return ("id".to_string(), "id".to_string(), true);
    }

    let segments: Vec<String> = path.split('.').map(|s| s.to_string()).collect();
    let joined = segments.join(",");

    (
        format!("{} #> '{{{}}}'", root, joined),
        format!("{} #>> '{{{}}}'", root, joined),
        false,
    )
}

/// Translate a MongoDB equality match
fn translate_equality(
    expression: &str,
    is_column: bool,
    value: &Bson,
    binds: &mut Vec<Bind>,
) -> Result<String, ::sqlx::Error> {
    if is_column {
        match value {
            Bson::String(id) => {
                binds.push(Bind::Text(id.clone()));
                Ok(format!("{} = ${}", expression, binds.len()))
            }
            Bson::Document(_) => {
                // Composite ID match against the embedded document
                let json = bson_to_json(value);
                binds.push(Bind::Json(json));
                Ok(format!("data #> '{{_id}}' = ${}::jsonb", binds.len()))
            }
            _ => Err(db_err("unsupported _id equality value")),
        }
    } else if matches!(value, Bson::Null | Bson::Undefined) {
        Ok(format!(
            "({} IS NULL OR {} = 'null'::jsonb)",
            expression, expression
        ))
    } else {
        binds.push(Bind::Json(bson_to_json(value)));
        Ok(format!("{} = ${}::jsonb", expression, binds.len()))
    }
}

/// Translate a single MongoDB operator condition
fn translate_operator_condition(
    expression: &str,
    text_expression: &str,
    is_column: bool,
    op: &str,
    value: &Bson,
    binds: &mut Vec<Bind>,
) -> Result<String, ::sqlx::Error> {
    match op {
        "$eq" => translate_equality(expression, is_column, value, binds),
        "$ne" => {
            if !is_column && matches!(value, Bson::Null | Bson::Undefined) {
                Ok(format!(
                    "({} IS NOT NULL AND {} <> 'null'::jsonb)",
                    expression, expression
                ))
            } else if is_column {
                let Bson::String(id) = value else {
                    return Err(db_err("unsupported _id $ne value"));
                };

                binds.push(Bind::Text(id.clone()));
                Ok(format!("{} <> ${}", expression, binds.len()))
            } else {
                binds.push(Bind::Json(bson_to_json(value)));
                Ok(format!("{} <> ${}::jsonb", expression, binds.len()))
            }
        }
        "$in" | "$nin" => {
            let negation = if op == "$nin" { "NOT " } else { "" };

            if is_column {
                let Bson::Array(items) = value else {
                    return Err(db_err("$in must be an array"));
                };

                let ids: Vec<String> = items
                    .iter()
                    .map(|item| match item {
                        Bson::String(id) => Ok(id.clone()),
                        _ => Err(db_err("unsupported _id $in value")),
                    })
                    .collect::<std::result::Result<_, _>>()?;

                binds.push(Bind::TextArray(ids));
                Ok(format!("{} {}= ANY(${})", negation, expression, binds.len()))
            } else {
                binds.push(Bind::Json(bson_to_json(value)));
                Ok(format!(
                    "{}(EXISTS (SELECT 1 FROM jsonb_array_elements(${}::jsonb) v \
                    WHERE {} = v OR (jsonb_typeof({}) = 'array' AND {} @> jsonb_build_array(v))))",
                    negation,
                    binds.len(),
                    expression,
                    expression,
                    expression
                ))
            }
        }
        "$gt" | "$gte" | "$lt" | "$lte" => {
            let sql_op = match op {
                "$gt" => ">",
                "$gte" => ">=",
                "$lt" => "<",
                "$lte" => "<=",
                _ => unreachable!(),
            };

            if is_column {
                let Bson::String(id) = value else {
                    return Err(db_err("unsupported _id comparison value"));
                };

                binds.push(Bind::Text(id.clone()));
                Ok(format!("{} {} ${}", expression, sql_op, binds.len()))
            } else {
                binds.push(Bind::Json(bson_to_json(value)));
                Ok(format!(
                    "{} {} ${}::jsonb",
                    expression, sql_op, binds.len()
                ))
            }
        }
        "$exists" => {
            let exists = match value {
                Bson::Boolean(v) => *v,
                Bson::Int32(v) => *v != 0,
                Bson::Int64(v) => *v != 0,
                _ => return Err(db_err("unsupported $exists value")),
            };

            if exists {
                Ok(format!("{} IS NOT NULL", expression))
            } else {
                Ok(format!("{} IS NULL", expression))
            }
        }
        "$all" => {
            binds.push(Bind::Json(bson_to_json(value)));
            Ok(format!("{} @> ${}::jsonb", expression, binds.len()))
        }
        "$elemMatch" => {
            let Bson::Document(doc) = value else {
                return Err(db_err("$elemMatch must be a document"));
            };

            let inner = translate_filter_inner(doc, "e", binds)?;
            Ok(format!(
                "EXISTS (SELECT 1 FROM jsonb_array_elements( \
                 CASE WHEN jsonb_typeof({}) = 'array' THEN {} ELSE '[]'::jsonb END) e WHERE {})",
                expression, expression, inner
            ))
        }
        "$size" => {
            let Bson::Int32(size) = value else {
                return Err(db_err("unsupported $size value"));
            };

            Ok(format!(
                "jsonb_array_length(CASE WHEN jsonb_typeof({}) = 'array' THEN {} ELSE '[]'::jsonb END) = {}",
                expression, expression, size
            ))
        }
        "$regex" => {
            let pattern = value
                .as_str()
                .ok_or_else(|| db_err("unsupported $regex value"))?;

            binds.push(Bind::Text(pattern.to_string()));
            Ok(format!("{} ~ ${}", text_expression, binds.len()))
        }
        _ => Err(db_err(&format!("unsupported MongoDB operator: {}", op))),
    }
}

/// Translate a MongoDB `$expr` expression into SQL
fn translate_expr(value: &Bson, root: &str, binds: &mut Vec<Bind>) -> Result<String, ::sqlx::Error> {
    let Bson::Document(doc) = value else {
        return Err(db_err("$expr must be a document"));
    };

    let (op, operands) = doc
        .iter()
        .next()
        .ok_or_else(|| db_err("$expr requires an operator"))?;

    let Bson::Array(operands) = operands else {
        return Err(db_err("$expr operator requires operands"));
    };

    if operands.len() != 2 {
        return Err(db_err("$expr requires exactly two operands"));
    }

    let sql_op = match op.as_str() {
        "$lt" => "<",
        "$lte" => "<=",
        "$gt" => ">",
        "$gte" => ">=",
        "$eq" => "=",
        "$ne" => "<>",
        _ => return Err(db_err(&format!("unsupported $expr operator: {}", op))),
    };

    let mut parts = Vec::new();
    for operand in operands {
        parts.push(match operand {
            Bson::String(path) if path.starts_with('$') => {
                let field = &path[1..];
                let (expression, _, _) = path_expressions(root, field);
                format!(
                    "(CASE WHEN jsonb_typeof({}) = 'number' THEN ({})::numeric END)",
                    expression, expression
                )
            }
            other => {
                binds.push(Bind::Json(bson_to_json(other)));
                format!("(${}::jsonb)::numeric", binds.len())
            }
        });
    }

    Ok(format!("{} {} {}", parts[0], sql_op, parts[1]))
}

/// Convert a BSON value into a JSON value
pub fn bson_to_json(value: &Bson) -> Json {
    match value {
        Bson::Double(v) => Json::from(*v),
        Bson::String(v) => Json::from(v.clone()),
        Bson::Boolean(v) => Json::from(*v),
        Bson::Null | Bson::Undefined => Json::Null,
        Bson::Int32(v) => Json::from(*v),
        Bson::Int64(v) => Json::from(*v),
        Bson::DateTime(v) => Json::from(
            iso8601_timestamp::Timestamp::from(v.to_system_time())
                .format()
                .to_string(),
        ),
        Bson::ObjectId(v) => Json::from(v.to_hex()),
        Bson::Document(doc) => Json::Object(
            doc.iter()
                .map(|(k, v)| (k.clone(), bson_to_json(v)))
                .collect(),
        ),
        Bson::Array(items) => Json::Array(items.iter().map(bson_to_json).collect()),
        Bson::Binary(binary) => Json::from(
            base64::engine::general_purpose::STANDARD.encode(&binary.bytes),
        ),
        Bson::Timestamp(ts) => Json::from(ts.time as i64),
        _ => Json::Null,
    }
}

/// Bind a list of values to a query
fn bind_query<'q>(
    mut query: ::sqlx::query::Query<'q, ::sqlx::Postgres, ::sqlx::postgres::PgArguments>,
    binds: &'q [Bind],
) -> ::sqlx::query::Query<'q, ::sqlx::Postgres, ::sqlx::postgres::PgArguments> {
    for bind in binds {
        query = match bind {
            Bind::Json(value) => query.bind(value.clone()),
            Bind::Text(value) => query.bind(value.clone()),
            Bind::TextArray(value) => query.bind(value.clone()),
            Bind::Int(value) => query.bind(*value),
        };
    }

    query
}

/// Decode a data column into a typed document
fn decode_row<T: DeserializeOwned>(
    row: ::sqlx::postgres::PgRow,
) -> std::result::Result<T, ::sqlx::Error> {
    let value: Json = row
        .try_get("data")
        .map_err(|_| db_err("missing data column"))?;

    serde_json::from_value(value)
        .map_err(|err| ::sqlx::Error::ColumnDecode {
            index: "data".to_string(),
            source: Box::new(err),
        })
}

/// Create a generic sqlx error
fn db_err(message: &str) -> ::sqlx::Error {
    ::sqlx::Error::Configuration(message.to_string().into())
}
