use std::future::Future;
use std::pin::Pin;
use tokio::fs;

#[tokio::main]
async fn main() {
    if let Err(e) = find_target_dirs(".").await {
        eprintln!("Error: {}", e);
    }
}

fn find_target_dirs(path: &str) -> Pin<Box<dyn Future<Output = std::io::Result<()>> + '_>> {
    Box::pin(async move {
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_dir() {
                    if let Some(name) = path.file_name() {
                        let dir_name = name.to_string_lossy();
                        if dir_name == "target" {
                            println!("{}", path.display());
                        }
                    }
                    if let Err(e) = find_target_dirs(path.to_str().unwrap_or("")).await {
                        eprintln!("Warning: Could not read directory {:?}: {}", path, e);
                    }
                }
            }
        }
        Ok(())
    })
}
