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
    Tcx,
    Fit,
    Other,
}
impl From<Mime> for FileType {
    fn from(mime: Mime) -> Self {
        match mime.type_().as_str() {
            "application" => match mime.subtype().as_str() {
                "gpx" | "gpx+xml" => FileType::Gpx,
                "vnd.garmin.tcx+xml" | "tcx+xml" | "tcx" => FileType::Tcx,
                "vnd.ant.fit" | "fit" => FileType::Fit,
                // Browsers often send activity files as octet-stream - caller must
                // use detect_from_bytes() to determine actual type
                "octet-stream" => FileType::Other,
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
            FileType::Tcx => "application/vnd.garmin.tcx+xml",
            FileType::Fit => "application/vnd.ant.fit",
            FileType::Other => "application/octet-stream",
        }
    }

    /// Detect file type from raw bytes by checking magic bytes/signatures.
    /// Used when MIME type is octet-stream and we need to determine actual format.
    pub fn detect_from_bytes(bytes: &[u8]) -> Self {
        // FIT files start with header size byte, then ".FIT" signature at offset 8-11
        if bytes.len() >= 12 {
            let header_size = bytes[0];
            // FIT header is either 12 or 14 bytes
            if (header_size == 12 || header_size == 14)
                && bytes.len() >= header_size as usize
                && &bytes[8..12] == b".FIT"
            {
                return FileType::Fit;
            }
        }

        // Check for XML-based formats (GPX and TCX)
        // Skip BOM if present and look for XML declaration or root element
        let text = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return FileType::Other,
        };

        let trimmed = text.trim_start_matches('\u{feff}').trim();

        // Look for TCX root element
        if trimmed.contains("<TrainingCenterDatabase") {
            return FileType::Tcx;
        }

        // Look for GPX root element
        if trimmed.contains("<gpx") {
            return FileType::Gpx;
        }

        FileType::Other
    }

    /// Returns true if this file type is a supported activity format
    pub fn is_supported_activity_format(self) -> bool {
        matches!(self, FileType::Gpx | FileType::Tcx | FileType::Fit)
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
        if !file_type.is_supported_activity_format() {
            return Err(AppError::InvalidInput(format!(
                "Unsupported file type: {file_type:?}. Only GPX, TCX, and FIT files are supported."
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
