use std::io::Write;
use std::path::Path;

pub fn write_atomic(path: &str, data: &[u8], mode: u32) -> anyhow::Result<()> {
    let target = Path::new(path);
    let parent = target
        .parent()
        .ok_or_else(|| anyhow::anyhow!("path has no parent directory: {path}"))?;
    std::fs::create_dir_all(parent)?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(data)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tmp.as_file_mut()
            .set_permissions(std::fs::Permissions::from_mode(mode))?;
    }

    tmp.persist(target)?;
    Ok(())
}
