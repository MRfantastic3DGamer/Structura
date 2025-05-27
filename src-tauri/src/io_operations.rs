use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::path::Path;
use tokio::io::AsyncReadExt;

pub async fn write_text_to_file<P: AsRef<Path>>(file_path: P, text: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path).await?;
    file.write_all(text.as_bytes()).await?;
    Ok(())
}

pub async fn read_text_from_file<P: AsRef<Path>>(file_path: P) -> std::io::Result<String> {
    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}
