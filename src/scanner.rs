//! Discover image files in a directory.

use std::path::{Path, PathBuf};

const IMAGE_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "ico", "qoi",
];

pub fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTS.iter().any(|x| x.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

/// List image files directly in `dir` (non-recursive), sorted by name.
pub fn list_images(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut images = Vec::new();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && is_image_path(&path) {
            images.push(path);
        }
    }
    images.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(b.file_name().unwrap_or_default())
    });
    Ok(images)
}

/// Recursive scan — used when the cwd has no images at the top level.
pub fn list_images_recursive(dir: &Path, max: usize) -> anyhow::Result<Vec<PathBuf>> {
    let mut images = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if images.len() >= max {
            break;
        }
        let path = entry.path();
        if path.is_file() && is_image_path(path) {
            images.push(path.to_path_buf());
        }
    }
    images.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(b.file_name().unwrap_or_default())
    });
    Ok(images)
}
