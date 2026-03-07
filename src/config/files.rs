use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

pub fn validate_file(path: &PathBuf) -> Result<(), String> {
    let metadata = match std::fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) => return Err(format!("cannot read file metadata: {}", e)),
    };
    if !metadata.is_file() {
        return Err("not a regular file".to_string());
    }
    if metadata.len() == 0 {
        return Err("file is empty".to_string());
    }
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("cannot open file: {}", e)),
    };
    let mut buffer = [0u8; 4096];
    let mut handle = file;
    match handle.read(&mut buffer) {
        Ok(n) => {
            if n == 0 {
                return Err("cannot read from file".to_string());
            }
        }
        Err(e) => return Err(format!("file read error: {}", e)),
    }
    Ok(())
}

pub fn scan_folder_for_files(
    folder: &str,
    validate: bool,
    allowed_extensions: Option<&[String]>,
) -> Result<Vec<PathBuf>, String> {
    let folder_path = PathBuf::from(folder);
    if !folder_path.exists() || !folder_path.is_dir() {
        return Err(format!("Scan folder does not exist or is not a directory: {}", folder));
    }
    let mut found_files = Vec::new();
    let mut invalid_count = 0;
    let mut skipped_count = 0;
    let entries = match std::fs::read_dir(&folder_path) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Failed to read directory {}: {}", folder, e)),
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }
        if let Some(allowed) = allowed_extensions
            && let Some(ext) = path.extension().and_then(|e| e.to_str())
        {
            let ext_lower = ext.to_lowercase();
            if !allowed.iter().any(|a| a.to_lowercase() == ext_lower) {
                skipped_count += 1;
                continue;
            }
        }
        if path.is_file() {
            if validate {
                match validate_file(&path) {
                    Ok(_) => found_files.push(path),
                    Err(e) => {
                        invalid_count += 1;
                        log::warn!("[Config] Skipping invalid file {}: {}", path.display(), e);
                    }
                }
            } else {
                found_files.push(path);
            }
        } else if path.is_dir() {
            match scan_folder_for_files(path.to_str().unwrap(), validate, allowed_extensions) {
                Ok(sub_files) => found_files.extend(sub_files),
                Err(e) => {
                    log::warn!("[Config] Failed to scan subdirectory {}: {}", path.display(), e);
                }
            }
        }
    }
    if found_files.is_empty() {
        let mut error_msg = format!("No valid files found in folder: {}", folder);
        if invalid_count > 0 {
            error_msg.push_str(&format!(" ({} invalid files skipped)", invalid_count));
        }
        if skipped_count > 0 {
            error_msg.push_str(&format!(" ({} files skipped by filters)", skipped_count));
        }
        return Err(error_msg);
    }
    log::info!(
        "[Config] Scanned folder '{}': found {} valid files, {} invalid, {} skipped",
        folder,
        found_files.len(),
        invalid_count,
        skipped_count
    );
    Ok(found_files)
}