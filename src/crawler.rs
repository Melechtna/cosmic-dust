use std::path::PathBuf;
use tokio::task::spawn_blocking;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
}

pub async fn crawl_files(mount_point: String, verbose: bool) -> Vec<FileEntry> {
    spawn_blocking(move || {
        let mut entries = Vec::new();

        for entry in WalkDir::new(&mount_point)
            .max_depth(1)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            if path == PathBuf::from(&mount_point) {
                continue; // Skip the mount point itself
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    if verbose {
                        println!("Error reading metadata for {:?}: {:?}", path, e);
                    }
                    continue;
                }
            };

            let size = if metadata.is_file() {
                metadata.len()
            } else if metadata.is_dir() {
                let dir_size = WalkDir::new(&path)
                    .follow_links(false)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
                    .sum::<u64>();
                dir_size
            } else {
                continue;
            };

            let entry = FileEntry {
                path,
                size,
                is_dir: metadata.is_dir(),
            };
            if verbose {
                println!("Crawled: {:?}", entry);
            }
            entries.push(entry);
        }

        if verbose {
            println!("Total entries crawled for {}: {}", mount_point, entries.len());
        }
        entries
    })
        .await
        .unwrap_or_else(|e| {
            if verbose {
                println!("Crawl failed: {:?}", e);
            }
            Vec::new()
        })
}