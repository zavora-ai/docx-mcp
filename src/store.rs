use std::collections::HashMap;
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::error::DocxMcpError;

/// A single entry in the DocumentStore.
pub struct DocumentEntry {
    pub id: String,
    pub data: docx_rs::Docx,
    pub file_path: Option<String>,
    pub last_access: Instant,
}

/// In-memory store for open documents, keyed by UUID handles.
/// Supports TTL-based eviction and LRU eviction at capacity.
pub struct DocumentStore {
    documents: HashMap<String, DocumentEntry>,
    max_capacity: usize,
    ttl: Duration,
}

impl DocumentStore {
    /// Create a new DocumentStore with the given capacity limit and TTL duration.
    pub fn new(max_capacity: usize, ttl: Duration) -> Self {
        Self {
            documents: HashMap::new(),
            max_capacity,
            ttl,
        }
    }

    /// Insert a new document. Runs TTL eviction, then LRU eviction if at capacity,
    /// generates a UUID v4 handle, inserts the entry, and returns the handle.
    pub fn insert(&mut self, data: docx_rs::Docx, file_path: Option<String>) -> String {
        // Step 1: evict expired entries
        self.evict_expired();

        // Step 2: if still at capacity, evict LRU
        if self.documents.len() >= self.max_capacity {
            self.evict_lru();
        }

        // Step 3: generate UUID v4
        let id = Uuid::new_v4().to_string();

        // Step 4: insert
        let entry = DocumentEntry {
            id: id.clone(),
            data,
            file_path,
            last_access: Instant::now(),
        };
        self.documents.insert(id.clone(), entry);

        id
    }

    /// Get a mutable reference to a document entry, updating last_access.
    /// Returns DocumentNotFound if the handle doesn't exist.
    pub fn get_mut(&mut self, handle: &str) -> Result<&mut DocumentEntry, DocxMcpError> {
        let entry = self.documents.get_mut(handle).ok_or_else(|| {
            DocxMcpError::DocumentNotFound {
                handle: handle.to_string(),
            }
        })?;
        entry.last_access = Instant::now();
        Ok(entry)
    }

    /// Get a reference to a document entry, updating last_access.
    /// Takes `&mut self` because it updates the last_access timestamp.
    /// Returns DocumentNotFound if the handle doesn't exist.
    pub fn get(&mut self, handle: &str) -> Result<&DocumentEntry, DocxMcpError> {
        let entry = self.documents.get_mut(handle).ok_or_else(|| {
            DocxMcpError::DocumentNotFound {
                handle: handle.to_string(),
            }
        })?;
        entry.last_access = Instant::now();
        Ok(entry)
    }

    /// Remove a document entry by handle.
    /// Returns DocumentNotFound if the handle doesn't exist.
    pub fn remove(&mut self, handle: &str) -> Result<(), DocxMcpError> {
        if self.documents.remove(handle).is_none() {
            return Err(DocxMcpError::DocumentNotFound {
                handle: handle.to_string(),
            });
        }
        Ok(())
    }

    /// Evict all entries whose last_access exceeds the TTL.
    pub fn evict_expired(&mut self) {
        let now = Instant::now();
        let ttl = self.ttl;
        self.documents
            .retain(|_, entry| now.duration_since(entry.last_access) <= ttl);
    }

    /// Evict the least-recently-accessed entry (oldest last_access).
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self
            .documents
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(key, _)| key.clone())
        {
            self.documents.remove(&lru_key);
        }
    }
}
