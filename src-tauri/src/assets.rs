//! Detection of supported 3D asset format and scanning of folder contents

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Supported asset extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &["vrm", "vrma", "pmx", "pmd", "fbx", "glb", "gltf", "x"];

#[derive(Debug, Clone, Serialize)]
pub struct AssetInfo {
    pub path: String,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectResult {
    /// true if the path is an asset or a folder containing assets
    pub is_asset: bool,
    /// The original path that was inspected
    pub source: String,
    /// List of assets (1 if file, N if folder)
    pub assets: Vec<AssetInfo>,
}

/// Returns the lowercase file extension
fn extension_of(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

/// Returns `true` if the file is a supported asset
pub fn is_supported_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match extension_of(path) {
        Some(ext) => SUPPORTED_EXTENSIONS.contains(&ext.as_str()),
        None => false,
    }
}

/// Returns the asset info for a supported file
fn asset_info(path: &Path) -> Option<AssetInfo> {
    if !is_supported_file(path) {
        return None;
    }

    let kind = extension_of(path).unwrap_or_default();
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();
    Some(AssetInfo {
        path: path.to_string_lossy().into_owned(),
        name,
        kind,
    })
}

/// Scan the contents of a folder for supported assets
fn scan_folder(dir: &Path) -> Vec<AssetInfo> {
    let mut assets = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(info) = asset_info(&path) {
                assets.push(info);
            }
        }
    }

    assets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    assets
}

/// Inspects a path as a supported file or a folder containing supported assets
pub fn inspect(path: &str) -> InspectResult {
    let p = PathBuf::from(path);

    if p.is_dir() {
        let assets = scan_folder(&p);
        return InspectResult {
            is_asset: !assets.is_empty(),
            source: path.to_string(),
            assets,
        };
    }

    if let Some(info) = asset_info(&p) {
        return InspectResult {
            is_asset: true,
            source: path.to_string(),
            assets: vec![info],
        };
    }

    InspectResult {
        is_asset: false,
        source: path.to_string(),
        assets: Vec::new(),
    }
}

/// Tests for the `assets` module
#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    /// Creates a unique temporary directory for the test
    fn temp_dir() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let dir = env::temp_dir().join(format!("moew-assets-test-{id}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Writes a dummy file and returns its path
    fn write_file(dir: &Path, name: &str) -> PathBuf {
        let file = dir.join(name);
        fs::write(&file, b"dummy").unwrap();
        file
    }

    /// Returns the extension of the file in lowercase
    #[test]
    fn extension_of_is_lowercased() {
        assert_eq!(
            extension_of(&PathBuf::from("model.VRM")),
            Some("vrm".to_string())
        );

        assert_eq!(
            extension_of(&PathBuf::from("model.pmx")),
            Some("pmx".to_string())
        );
    }

    /// Returns `None` when the file has no extension
    #[test]
    fn extension_of_is_none_without_extension() {
        assert_eq!(extension_of(&PathBuf::from("model")), None);
    }

    /// Accepts every supported file extension
    #[test]
    fn is_supported_file_accepts_every_supported_extension() {
        let dir = temp_dir();
        for ext in SUPPORTED_EXTENSIONS {
            let file = write_file(&dir, &format!("model.{}", ext));
            assert!(is_supported_file(&file));
        }
    }

    /// Rejects unsupported file extensions
    #[test]
    fn is_supported_file_rejects_unknown_extension() {
        let dir = temp_dir();
        let file = write_file(&dir, "notes.txt");
        assert!(!is_supported_file(&file));
    }

    /// Rejects directories
    #[test]
    fn is_supported_file_rejects_directories() {
        let dir = temp_dir();
        assert!(!is_supported_file(&dir));
    }

    /// Returns an asset for a supported file
    #[test]
    fn inspect_single_file_returns_one_asset() {
        let dir = temp_dir();
        let file = write_file(&dir, "miku.pmx");
        let result = inspect(file.to_string_lossy().as_ref());
        assert!(result.is_asset);
        assert_eq!(result.assets.len(), 1);
        assert_eq!(result.assets[0].name, "miku.pmx");
        assert_eq!(result.assets[0].kind, "pmx");
    }

    /// Returns sorted assets from a folder and ignores unsupported files
    #[test]
    fn inspect_folder_returns_sorted_assets_only() {
        let dir = temp_dir();
        write_file(&dir, "b.vrm");
        write_file(&dir, "a.glb");
        write_file(&dir, "readme.txt"); // must be ignored
        let result = inspect(dir.to_string_lossy().as_ref());
        assert!(result.is_asset);
        assert_eq!(result.assets.len(), 2);
        assert_eq!(result.assets[0].name, "a.glb");
        assert_eq!(result.assets[1].name, "b.vrm");
    }

    /// Returns `false` for unsupported files
    #[test]
    fn inspect_unsupported_file_is_not_asset() {
        let dir = temp_dir();
        let file = write_file(&dir, "readme.txt");
        let result = inspect(file.to_string_lossy().as_ref());
        assert!(!result.is_asset);
        assert!(result.assets.is_empty());
    }

    /// Returns `false` for empty folders
    #[test]
    fn inspect_empty_folder_is_not_asset() {
        let dir = temp_dir();
        let result = inspect(dir.to_string_lossy().as_ref());
        assert!(!result.is_asset);
        assert!(result.assets.is_empty());
    }
}
