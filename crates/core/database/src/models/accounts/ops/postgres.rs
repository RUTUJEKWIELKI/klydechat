use ::sqlx::Row;
use crate::{AbstractAccounts, Account, PostgresDb};
use iso8601_timestamp::{Duration, Timestamp};
use revolt_result::Result;

const COL: &str = "accounts";

#[async_trait]
impl AbstractAccounts for PostgresDb {
    /// Find account by id
    async fn fetch_account(&self, id: &str) -> Result<Account> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(UnknownUser))
    }

    /// Find account by normalised email
    async fn fetch_account_by_normalised_email(
        &self,
        normalised_email: &str,
    ) -> Result<Option<Account>> {
        self.find_one_ci(COL, "email_normalised", normalised_email)
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("find_one", COL))
    }

    /// Find account with active pending email verification
    async fn fetch_account_with_email_verification(&self, token: &str) -> Result<Account> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "verification.token": token,
                "verification.expiry": {
                    "$gte": ts_bson(Timestamp::now_utc())
                }
            }
        )?
        .ok_or_else(|| create_error!(InvalidToken))
    }

    /// Find account with active password reset
    async fn fetch_account_with_password_reset(&self, token: &str) -> Result<Account> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "password_reset.token": token,
                "password_reset.expiry": {
                    "$gte": ts_bson(Timestamp::now_utc())
                }
            }
        )?
        .ok_or_else(|| create_error!(InvalidToken))
    }

    /// Find account with active deletion token
    async fn fetch_account_with_deletion_token(&self, token: &str) -> Result<Account> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "deletion.token": token,
                "deletion.expiry": {
                    "$gte": ts_bson(Timestamp::now_utc())
                }
            }
        )?
        .ok_or_else(|| create_error!(InvalidToken))
    }

    /// Find accounts which are due to be deleted
    async fn fetch_accounts_due_for_deletion(&self) -> Result<Vec<Account>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "deletion.status": "Scheduled",
                "deletion.after": {
                    "$lte": ts_bson(Timestamp::now_utc())
                }
            }
        )
    }

    // Save account
    async fn save_account(&self, account: &Account) -> Result<()> {
        self.upsert_full(COL, account)
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("find_one", COL))
    }

    async fn bump_lockout_count(&self, id: &str) -> Result<Account> {
        let now = Timestamp::now_utc();

        let expiry_5 = ts_json(now + Duration::hours(1));
        let expiry_4 = ts_json(now + Duration::minutes(5));
        let expiry_3 = ts_json(now + Duration::minutes(1));

        let sql = "UPDATE accounts SET data = jsonb_set( \
                   jsonb_set(data, '{lockout,attempts}', \
                   to_jsonb(COALESCE(CASE WHEN jsonb_typeof(data #> '{lockout,attempts}') = 'number' \
                   THEN (data #> '{lockout,attempts}')::numeric END, 0) + 1)), \
                   '{lockout,expiry}', \
                   CASE \
                   WHEN COALESCE(CASE WHEN jsonb_typeof(data #> '{lockout,attempts}') = 'number' \
                   THEN (data #> '{lockout,attempts}')::numeric END, 0) + 1 >= 5 THEN $1::jsonb \
                   WHEN COALESCE(CASE WHEN jsonb_typeof(data #> '{lockout,attempts}') = 'number' \
                   THEN (data #> '{lockout,attempts}')::numeric END, 0) + 1 = 4 THEN $2::jsonb \
                   WHEN COALESCE(CASE WHEN jsonb_typeof(data #> '{lockout,attempts}') = 'number' \
                   THEN (data #> '{lockout,attempts}')::numeric END, 0) + 1 = 3 THEN $3::jsonb \
                   ELSE data #> '{lockout,expiry}' END) \
                   WHERE id = $4 RETURNING data";

        let row = ::sqlx::query(sql)
            .bind(expiry_5)
            .bind(expiry_4)
            .bind(expiry_3)
            .bind(id)
            .fetch_optional(&self.0)
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("find_one_and_update", COL))?;

        let value: serde_json::Value = row
            .and_then(|row| row.try_get("data").ok())
            .ok_or_else(|| create_error!(UnknownUser))?;

        serde_json::from_value(value)
            .map_err(|_: serde_json::Error| create_database_error!("find_one_and_update", COL))
    }
}

/// Convert a timestamp into a BSON string value for use in filters
fn ts_bson(ts: Timestamp) -> bson::Bson {
    bson::Bson::String(ts.format().to_string())
}

/// Convert a timestamp into a JSON value for use as a query bind
fn ts_json(ts: Timestamp) -> serde_json::Value {
    serde_json::Value::String(ts.format().to_string())
}
