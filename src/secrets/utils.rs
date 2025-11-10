use std::fs;
use std::path::Path;
use tracing::debug;

pub fn create_secrets_dir() -> std::io::Result<()> {
    fs::create_dir_all("secrets")?;
    debug!("Created/verified secrets directory");
    Ok(())
}

pub fn write_json_pretty<P: AsRef<Path>, T: serde::Serialize>(
    path: P,
    data: &T,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(&path, json)?;
    debug!("Wrote pretty JSON to {:?}", path.as_ref());
    Ok(())
}

pub fn write_json<P: AsRef<Path>, T: serde::Serialize>(path: P, data: &T) -> std::io::Result<()> {
    let json = serde_json::to_string(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(&path, json)?;
    debug!("Wrote compact JSON to {:?}", path.as_ref());
    Ok(())
}
