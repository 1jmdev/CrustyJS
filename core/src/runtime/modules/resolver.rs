use std::path::{Path, PathBuf};

pub fn resolve(specifier: &str, from_file: &Path) -> PathBuf {
    let base = if from_file.is_dir() {
        from_file.to_path_buf()
    } else {
        from_file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    let mut candidate = if specifier.starts_with("./") || specifier.starts_with("../") {
        base.join(specifier)
    } else {
        PathBuf::from(specifier)
    };

    if candidate.extension().is_none() {
        candidate.set_extension("js");
    }

    std::fs::canonicalize(&candidate).unwrap_or(candidate)
}
