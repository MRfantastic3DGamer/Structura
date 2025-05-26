use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::path::Path;

pub async fn write_text_to_file<P: AsRef<Path>>(file_path: P, text: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path).await?;
    file.write_all(text.as_bytes()).await?;
    Ok(())
}
