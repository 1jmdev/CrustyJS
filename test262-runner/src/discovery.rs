use std::path::{Path, PathBuf};

use walkdir::WalkDir;

pub fn collect_test_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| is_test_file(entry.path()))
        .map(|entry| entry.into_path())
        .collect()
}

fn is_test_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "js") && !is_excluded(path)
}

fn is_excluded(path: &Path) -> bool {
    path.components().any(|part| {
        let text = part.as_os_str().to_string_lossy();
        text == "_FIXTURE" || text == "intl402"
    })
}
