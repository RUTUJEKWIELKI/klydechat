use revolt_result::{Result, ToRevoltError};
use serde::Deserialize;

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ChunkedDatabaseGenerator<T> {
    Reference {
        offset: usize,
        data: Vec<T>,
    },
}

impl<T: for<'d> Deserialize<'d> + Clone> ChunkedDatabaseGenerator<T> {
    pub fn new_reference(data: Vec<T>) -> Self {
        Self::Reference { offset: 0, data }
    }

    pub async fn next(&mut self) -> Result<Option<T>> {
        match self {
            Self::Reference { offset, data } => {
                if let Some(value) = data.get(*offset) {
                    *offset += 1;
                    Ok(Some(value.clone()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn next_n(&mut self, n: usize) -> Result<Option<Vec<T>>> {
        let mut docs = Vec::new();

        while docs.len() < n {
            if let Some(doc) = self.next().await? {
                docs.push(doc);
            } else if docs.is_empty() {
                return Ok(None);
            } else {
                break;
            }
        }

        Ok(Some(docs))
    }
}
