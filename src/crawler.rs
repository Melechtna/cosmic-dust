use jwalk::{Parallelism, WalkDir};
use std::path::PathBuf;
use std::time::Duration;
use std::fs;
use tokio::task::spawn_blocking;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

pub async fn crawl_files(mount_point: String, verbose: bool) -> Vec<FileEntry> {
    let root = PathBuf::from(&mount_point);

    spawn_blocking(move || {
        let mut dir_sizes: std::collections::HashMap<PathBuf, u64> = std::collections::HashMap::new();
        let mut top_level_entries: Vec<FileEntry> = Vec::new();

        for entry in WalkDir::new(&root)
            .follow_links(false)
            .process_read_dir(|_depth, path, _read_dir_state, children| {
                // Iterate over tree results and skip any folder we can't access
                children.retain(|child_result| {
                    if let Ok(dir_entry) = child_result {
                        // If metadata fails (permission denied, etc.), skip descending into this folder
                        if dir_entry.file_type.is_dir() && fs::metadata(dir_entry.path()).is_err() {
                            return false;
                        }
                    }
                    true
                });

                // Skip common virtual/no-space root folders entirely
                let skip_prefixes = ["/proc", "/sys", "/dev"];
                if skip_prefixes.iter().any(|&prefix| path.starts_with(prefix)) {
                    children.clear();  // Skip this dir and all folders
                }
            })
            .parallelism(Parallelism::RayonDefaultPool { busy_timeout: Duration::from_secs(5) })
            .skip_hidden(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let depth = entry.depth();

            if depth == 0 {
                continue; // Skip root
            }

            // Fetch metadata once
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    if verbose {
                        println!("Error reading metadata for {:?}: {:?}", path, e);
                    }
                    continue;
                }
            };

            let size = metadata.len();
            let is_dir = metadata.is_dir();

            if is_dir {
                dir_sizes.entry(path.clone()).or_insert(0);
            } else {
                let mut current = path.clone();
                loop {
                    *dir_sizes.entry(current.clone()).or_insert(0) += size;
                    if let Some(parent) = current.parent() {
                        if parent == root {
                            break;
                        }
                        current = parent.to_path_buf();
                    } else {
                        break;
                    }
                }
            }

            // Collect top-level items on the fly
            if depth == 1 {
                let entry = FileEntry {
                    path,
                    size: if is_dir { 0 } else { size },
                    is_dir,
                };
                if verbose {
                    println!("Top-level discovered: {:?}", entry);
                }
                top_level_entries.push(entry);
            }
        }

        // Finalize top-level folder sizes from aggregation
        for entry in &mut top_level_entries {
            if entry.is_dir {
                entry.size = *dir_sizes.get(&entry.path).unwrap_or(&0);
            }
        }

        if verbose {
            println!("Total top-level entries for {}: {}", mount_point, top_level_entries.len());
        }

        top_level_entries
    })
        .await
        .unwrap_or_else(|e| {
            if verbose {
                println!("Crawl failed: {:?}", e);
            }
            Vec::new()
        })
}