use std::io::Read;
use std::io::Write;
use std::path::Path;

use super::check::{self, VersionProvider};

const DEFAULT_RELEASE_BASE_URL: &str = "https://github.com/epicsagas/claudy/releases/download";

pub struct DownloadResult {
    pub path: std::path::PathBuf,
    pub version: String,
    pub cleanup_dir: std::path::PathBuf,
}

pub fn download_latest_if_newer(current: &str) -> anyhow::Result<Option<DownloadResult>> {
    if std::env::var("CLAUDY_SKIP_SELF_UPDATE").as_deref() == Ok("1") {
        return Ok(None);
    }

    let url = std::env::var("CLAUDY_UPDATE_URL").unwrap_or_else(|_| {
        "https://github.com/epicsagas/claudy/releases/latest/download/latest.json".to_string()
    });
    let provider = super::check::GitHubProvider::new(url);
    let meta = provider.fetch_latest()?;

    if !super::check::is_newer(&meta.version, current) {
        return Ok(None);
    }

    let version = check::display_version(&meta.version);
    let (binary_path, cleanup_dir) = download_release_binary(&version)?;

    Ok(Some(DownloadResult {
        path: binary_path,
        version,
        cleanup_dir,
    }))
}

fn download_release_binary(
    version: &str,
) -> anyhow::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_path = tmp_dir.path().to_path_buf();
    let tmp_path_clone = tmp_path.clone();
    std::mem::forget(tmp_dir);

    let asset_name = release_asset_name()?;
    let asset_path = tmp_path.join(&asset_name);
    let checksums_path = tmp_path.join("checksums.txt");
    let binary_path = tmp_path.join("claudy");

    let asset_url = release_asset_url(version, &asset_name);
    let checksums_url = release_asset_url(version, "checksums.txt");

    download_file(&asset_url, &asset_path)?;
    download_file(&checksums_url, &checksums_path)?;
    verify_checksum(&asset_path, &checksums_path, &asset_name)?;
    extract_binary(&asset_path, &binary_path)?;

    Ok((binary_path, tmp_path_clone))
}

fn release_asset_name() -> anyhow::Result<String> {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err(anyhow::anyhow!("unsupported operating system"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err(anyhow::anyhow!("unsupported architecture"));
    };

    Ok(format!("claudy_{}_{}.tar.gz", os, arch))
}

fn release_asset_url(version: &str, asset: &str) -> String {
    if let Ok(base) = std::env::var("CLAUDY_RELEASE_BASE_URL") {
        let base = base.trim().trim_end_matches('/');
        if !base.is_empty() {
            return format!("{}/{}", base, asset);
        }
    }
    format!(
        "{}/{}/{}",
        DEFAULT_RELEASE_BASE_URL.trim_end_matches('/'),
        version,
        asset
    )
}

fn download_file(url: &str, path: &Path) -> anyhow::Result<()> {
    let agent = ureq::Agent::new_with_defaults();
    let mut resp = agent
        .get(url)
        .header("User-Agent", "claudy-install")
        .call()?;

    let mut data = Vec::new();
    resp.body_mut().as_reader().read_to_end(&mut data)?;
    std::fs::write(path, &data)?;
    Ok(())
}

fn verify_checksum(
    asset_path: &Path,
    checksums_path: &Path,
    asset_name: &str,
) -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};

    let data = std::fs::read_to_string(checksums_path)?;
    let mut expected = "";
    for line in data.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 2 {
            continue;
        }
        if Path::new(fields[fields.len() - 1])
            .file_name()
            .map(|n| n.to_string_lossy() == asset_name)
            .unwrap_or(false)
        {
            expected = fields[0];
            break;
        }
    }
    if expected.is_empty() {
        anyhow::bail!("checksum for {} not found", asset_name);
    }

    let mut file = std::fs::File::open(asset_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let result = hasher.finalize();
    let got = hex::encode(result);

    if !got.eq_ignore_ascii_case(expected) {
        anyhow::bail!("checksum mismatch for {}", asset_name);
    }
    Ok(())
}

fn extract_binary(asset_path: &Path, binary_path: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::open(asset_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.file_name().map(|n| n == "claudy").unwrap_or(false) {
            let mut tmp =
                tempfile::NamedTempFile::new_in(binary_path.parent().unwrap_or(Path::new(".")))?;
            std::io::copy(&mut entry, &mut tmp)?;
            tmp.as_file_mut().flush()?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                tmp.as_file_mut()
                    .set_permissions(std::fs::Permissions::from_mode(0o755))?;
            }
            tmp.persist(binary_path)?;
            return Ok(());
        }
    }
    Err(anyhow::anyhow!(
        "claudy binary not found in {}",
        asset_path.display()
    ))
}
