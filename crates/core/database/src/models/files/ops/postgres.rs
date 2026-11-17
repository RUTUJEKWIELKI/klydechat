use revolt_config::report_internal_error;
use revolt_result::Result;

use crate::File;
use crate::FileHash;
use crate::FileUsedFor;
use crate::PostgresDb;

use super::AbstractAttachments;

static COL: &str = "attachments";

#[async_trait]
impl AbstractAttachments for PostgresDb {
    /// Insert attachment into database.
    async fn insert_attachment(&self, attachment: &File) -> Result<()> {
        query!(self, insert_one, COL, &attachment).map(|_| ())
    }

    /// Fetch an attachment by its id.
    async fn fetch_attachment(&self, tag: &str, file_id: &str) -> Result<File> {
        query!(
            self,
            find_one,
            COL,
            doc! {
                "_id": file_id,
                "tag": tag
            }
        )?
        .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch all deleted attachments.
    async fn fetch_deleted_attachments(&self) -> Result<Vec<File>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "deleted": true,
                "reported": {
                    "$ne": true
                }
            }
        )
    }

    /// Fetch all dangling attachments.
    async fn fetch_dangling_files(&self) -> Result<Vec<File>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "used_for.type": {
                    "$exists": 0
                },
                "deleted": {
                    "$ne": true
                }
            }
        )
    }

    /// Count references to a given hash, by either its original or processed hash.
    async fn count_file_hash_references(&self, hash: &FileHash) -> Result<usize> {
        query!(
            self,
            count_documents,
            COL,
            doc! {
                "hash": {
                    "$in": [&hash.id, &hash.processed_hash]
                }
            }
        )
        .map(|count| count as usize)
    }

    /// Find an attachment by its details and mark it as used by a given parent.
    async fn find_and_use_attachment(
        &self,
        id: &str,
        tag: &str,
        used_for: FileUsedFor,
        uploader_id: String,
    ) -> Result<File> {
        let file = query!(
            self,
            find_one,
            COL,
            doc! {
                "_id": id,
                "tag": tag,
                "used_for": {
                    "$exists": false
                }
            }
        )?
        .ok_or_else(|| create_error!(NotFound))?;

        query!(
            self,
            update_one_raw,
            COL,
            doc! {
                "_id": id
            },
            &doc! {
                "$set": {
                    "used_for": report_internal_error!(bson::to_bson(&used_for))?,
                    "uploader_id": uploader_id
                }
            }
        )
        .map(|_| ())?;

        Ok(file)
    }

    /// Mark an attachment as having been reported.
    async fn mark_attachment_as_reported(&self, id: &str) -> Result<()> {
        query!(
            self,
            update_one_raw,
            COL,
            doc! {
                "_id": id
            },
            &doc! {
                "$set": {
                    "reported": true
                }
            }
        )
        .map(|_| ())
    }

    /// Mark an attachment as having been deleted.
    async fn mark_attachment_as_deleted(&self, id: &str) -> Result<()> {
        query!(
            self,
            update_one_raw,
            COL,
            doc! {
                "_id": id
            },
            &doc! {
                "$set": {
                    "deleted": true
                }
            }
        )
        .map(|_| ())
    }

    /// Mark multiple attachments as having been deleted.
    async fn mark_attachments_as_deleted(&self, ids: &[String]) -> Result<()> {
        query!(
            self,
            update_many_raw,
            COL,
            doc! {
                "_id": {
                    "$in": ids
                }
            },
            &doc! {
                "$set": {
                    "deleted": true
                }
            }
        )
        .map(|_| ())
    }

    /// Delete the attachment entry.
    async fn delete_attachment(&self, id: &str) -> Result<()> {
        query!(self, delete_one_by_id, COL, id).map(|_| ())
    }
}

impl PostgresDb {
    pub async fn delete_many_attachments(&self, projection: bson::Document) -> Result<()> {
        query!(
            self,
            update_many_raw,
            COL,
            projection,
            &doc! {
                "$set": {
                    "deleted": true
                }
            }
        )
        .map(|_| ())
    }
}
