use crate::{AbstractAccountInvites, AccountInvite, PostgresDb};
use revolt_result::Result;

const COL: &str = "account_invites";

#[async_trait]
impl AbstractAccountInvites for PostgresDb {
    /// Find invite by id
    async fn fetch_account_invite(&self, id: &str) -> Result<AccountInvite> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(InvalidInvite))
    }

    /// Save invite
    async fn save_account_invite(&self, invite: &AccountInvite) -> Result<()> {
        self.upsert_full(COL, invite)
            .await
            .map_err(|_: ::sqlx::Error| create_database_error!("upsert_one", COL))
    }
}
