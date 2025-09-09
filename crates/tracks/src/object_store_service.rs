use bytes::Bytes;
use object_store::{local::LocalFileSystem, path::Path, ObjectStore};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Clone)]
pub struct ObjectStoreService {
    store: Arc<dyn ObjectStore>,
    base_path: String,
}

impl ObjectStoreService {
    pub fn new_local(base_path: String) -> Self {
        let store = Arc::new(LocalFileSystem::new_with_prefix(&base_path).unwrap());
        Self { store, base_path }
    }

    pub async fn store_file(
        &self,
        content: Bytes,
        user_id: Uuid,
        filename: &str,
        activity_id: Uuid,
    ) -> Result<String, AppError> {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("gpx");

        let object_path = format!("activities/{}/{}.{}", user_id, activity_id, extension);

        let path = Path::from(object_path.clone());

        self.store
            .put(&path, content.into())
            .await
            .map_err(|e| AppError::InvalidInput(format!("Failed to store file: {}", e)))?;

        Ok(object_path)
    }

    pub async fn get_file(&self, object_path: &str) -> Result<Bytes, AppError> {
        let path = Path::from(object_path);

        let result = self
            .store
            .get(&path)
            .await
            .map_err(|_| AppError::NotFound)?;

        let bytes = result
            .bytes()
            .await
            .map_err(|e| AppError::InvalidInput(format!("Failed to read file: {}", e)))?;

        Ok(bytes)
    }

    pub async fn delete_file(&self, object_path: &str) -> Result<(), AppError> {
        let path = Path::from(object_path);

        self.store
            .delete(&path)
            .await
            .map_err(|e| AppError::InvalidInput(format!("Failed to delete file: {}", e)))?;

        Ok(())
    }
}
