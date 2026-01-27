use axum_extra::headers::Mime;
use bytes::Bytes;
use object_store::{ObjectStore, PutOptions, local::LocalFileSystem, path::Path};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    Gpx,
    Other,
}
impl From<Mime> for FileType {
    fn from(mime: Mime) -> Self {
        match mime.type_().as_str() {
            "application" => match mime.subtype().as_str() {
                "gpx" | "gpx+xml" => FileType::Gpx,
                // Browsers often send GPX files as octet-stream
                "octet-stream" => FileType::Gpx,
                s => {
                    tracing::warn!("Unknown mime subtype: {s}");
                    FileType::Other
                }
            },
            s => {
                tracing::warn!("Unknown mime type: {s}");
                FileType::Other
            }
        }
    }
}

impl FileType {
    pub fn as_mime_str(self) -> &'static str {
        match self {
            FileType::Gpx => "application/gpx+xml",
            FileType::Other => "application/octet-stream",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjectStoreService {
    store: Arc<dyn ObjectStore>,
    _base_path: String,
}

impl ObjectStoreService {
    pub fn new_local(base_path: String) -> Self {
        // Ensure the directory exists before creating the LocalFileSystem
        std::fs::create_dir_all(&base_path).expect("Failed to create uploads directory");
        let store = Arc::new(LocalFileSystem::new_with_prefix(&base_path).unwrap());
        Self {
            store,
            _base_path: base_path,
        }
    }

    pub async fn store_file(
        &self,
        user_id: Uuid,
        activity_id: Uuid,
        file_type: FileType,
        content: Bytes,
    ) -> Result<String, AppError> {
        if !matches!(file_type, FileType::Gpx) {
            return Err(AppError::InvalidInput(format!(
                "Unsupported file type: {file_type:?}. Only GPX files are supported."
            )));
        }

        let object_path = format!("activities/{user_id}/{activity_id}",);

        let path = Path::from(object_path.clone());

        let opts = PutOptions::default();

        // todo re-enable when using proper blob storage
        // opts.attributes.insert(
        //     object_store::Attribute::ContentType,
        //     file_type.as_mime_str().into(),
        // );

        self.store
            .put_opts(&path, content.into(), opts)
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
