use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use thiserror::Error;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Document not found")]
    NotFound,
    #[error("Collection not found")]
    CollectionNotFound,
    #[error("Lock poisoned")]
    LockPoisoned,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Collection {
    name: String,
    documents: Arc<RwLock<HashMap<String, Document>>>,
    path: PathBuf,
}

impl Collection {
    pub fn new(name: &str, db_path: &Path) -> Result<Self, DbError> {
        let path = db_path.join(format!("{}.json", name));
        debug!("Initializing collection at: {}", path.display());

        let documents = if path.exists() {
            info!("Loading existing collection: {}", name);
            let raw = fs::read_to_string(&path)?;
            serde_json::from_str(&raw)?
        } else {
            fs::create_dir_all(db_path)?;
            HashMap::new()
        };

        Ok(Self {
            name: name.to_string(),
            documents: Arc::new(RwLock::new(documents)),
            path,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn insert(&self, data: serde_json::Value, ttl: Option<i64>) -> Result<Document, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = ttl.map(|secs| now + chrono::Duration::seconds(secs));

        let doc = Document {
            id: id.clone(),
            data,
            created_at: now,
            updated_at: now,
            expires_at,
        };

        let mut docs = self.documents.write().map_err(|_| DbError::LockPoisoned)?;
        docs.insert(id.clone(), doc.clone());
        self.persist()?;
        info!("Inserted document with ID: {}", id);
        Ok(doc)
    }

    pub fn find(&self, id: &str) -> Result<Option<Document>, DbError> {
        let docs = self.documents.read().map_err(|_| DbError::LockPoisoned)?;
        Ok(docs.get(id).cloned())
    }

    pub fn find_all(&self) -> Result<Vec<Document>, DbError> {
        let docs = self.documents.read().map_err(|_| DbError::LockPoisoned)?;
        Ok(docs.values().cloned().collect())
    }

    pub fn update(&self, id: &str, data: serde_json::Value) -> Result<Document, DbError> {
        let mut docs = self.documents.write().map_err(|_| DbError::LockPoisoned)?;
        let doc = docs.get_mut(id).ok_or(DbError::NotFound)?;

        doc.data = data;
        doc.updated_at = Utc::now();
        let doc = doc.clone();

        self.persist()?;
        info!("Updated document with ID: {}", id);
        Ok(doc)
    }

    pub fn delete(&self, id: &str) -> Result<(), DbError> {
        let mut docs = self.documents.write().map_err(|_| DbError::LockPoisoned)?;
        if docs.remove(id).is_some() {
            self.persist()?;
            info!("Deleted document with ID: {}", id);
            Ok(())
        } else {
            Err(DbError::NotFound)
        }
    }

    fn persist(&self) -> Result<(), DbError> {
        let docs = self.documents.read().map_err(|_| DbError::LockPoisoned)?;
        let data = serde_json::to_string_pretty(&*docs)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        file.write_all(data.as_bytes())?;
        debug!("Persisted collection: {}", self.name);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Database {
    path: PathBuf,
    collections: Arc<RwLock<HashMap<String, Collection>>>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, DbError> {
        let path = path.as_ref().to_path_buf();
        info!("Initializing database at: {}", path.display());

        Ok(Self {
            path,
            collections: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn collection(&self, name: &str) -> Result<Collection, DbError> {
        let mut collections = self
            .collections
            .write()
            .map_err(|_| DbError::LockPoisoned)?;

        if let Some(col) = collections.get(name) {
            Ok(col.clone())
        } else {
            let col = Collection::new(name, &self.path)?;
            collections.insert(name.to_string(), col.clone());
            Ok(col)
        }
    }

    pub fn drop_collection(&self, name: &str) -> Result<(), DbError> {
        let mut collections = self
            .collections
            .write()
            .map_err(|_| DbError::LockPoisoned)?;

        if collections.remove(name).is_some() {
            let path = self.path.join(format!("{}.json", name));
            if path.exists() {
                fs::remove_file(path)?;
            }
            info!("Dropped collection: {}", name);
            Ok(())
        } else {
            Err(DbError::CollectionNotFound)
        }
    }

    pub fn start_ttl_cleaner(&self, interval_secs: u64) {
        let db = self.clone();
        std::thread::spawn(move || {
            let interval = std::time::Duration::from_secs(interval_secs);
            loop {
                std::thread::sleep(interval);
                if let Ok(collections) = db.collections.read() {
                    for col in collections.values() {
                        if let Ok(mut docs) = col.documents.write() {
                            let now = Utc::now();
                            let before = docs.len();
                            docs.retain(|_, doc| doc.expires_at.map_or(true, |exp| exp > now));
                            let after = docs.len();

                            if before != after {
                                let _ = col.persist();
                                debug!(
                                    "Cleaned {} expired documents from {}",
                                    before - after,
                                    col.name()
                                );
                            }
                        }
                    }
                }
            }
        });
    }
}
