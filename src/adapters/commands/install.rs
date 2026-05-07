use std::path::Path;

use crate::adapters::update;
use crate::config;
use crate::launcher as runtime;

use crate::domain::context::Context;

pub fn run_install(ctx: &mut Context) -> anyhow::Result<i32> {
    let is_homebrew = runtime::envkit::is_homebrew();

    let (exec_path, installed_version, cleanup) = if is_homebrew {
        let exec_path = std::env::current_exe()?;
        let version = update::check::display_version(crate::adapters::version::VALUE);
        (exec_path, version, None)
    } else {
        match resolve_install_binary() {
            Ok(result) => result,
            Err(e) => {
                ctx.output.warn(&format!(
                    "could not fetch latest release; installing current binary instead: {}",
                    e
                ));
                let exec_path = std::env::current_exe()?;
                let version = update::check::display_version(crate::adapters::version::VALUE);
                (exec_path, version, None)
            }
        }
    };

    ctx.paths.ensure_base_dirs()?;

    // Install the claudy binary to bin_dir
    let dest_binary = Path::new(&ctx.paths.bin_dir).join("claudy");
    if !is_homebrew {
        copy_executable(&exec_path.to_string_lossy(), &dest_binary.to_string_lossy())?;
    }

    config::vault::prune_outdated_entries(&mut ctx.secrets, &ctx.catalog);
    config::registry::write_registry(&ctx.paths.config_file, &ctx.config)?;
    config::vault::persist_vault(&ctx.paths.secrets_file, &ctx.secrets)?;

    // Ensure MCP server is registered in Claude Code settings
    crate::adapters::mcp::server::ensure_registered_global();

    // Ensure MCP server is registered in all existing modes
    sync_mode_registrations(&ctx.paths.modes_dir);

    // Clean up legacy files
    let legacy1 = std::path::Path::new(&ctx.paths.data_dir).join("claudy-full.sh");
    let legacy2 = std::path::Path::new(&ctx.paths.data_dir).join("banner");
    let _ = std::fs::remove_file(&legacy1);
    let _ = std::fs::remove_file(&legacy2);

    ctx.output.success(&format!(
        "installed Claudy {} to {}",
        installed_version, ctx.paths.bin_dir
    ));

    // Check PATH
    let path_env = std::env::var("PATH").unwrap_or_default();
    if !path_contains_dir(&path_env, &ctx.paths.bin_dir) {
        ctx.output.warn(&format!(
            "{} is not on PATH; add `export PATH=\"{}:$PATH\"` to your shell profile and restart your shell",
            ctx.paths.bin_dir, ctx.paths.bin_dir
        ));
    }

    drop(cleanup);
    Ok(0)
}

fn sync_mode_registrations(modes_dir: &str) {
    let modes_path = Path::new(modes_dir);
    if !modes_path.exists() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(modes_path) else {
        return;
    };
    for entry in entries.flatten() {
        // Skip symlinks to prevent writes outside the modes tree
        if entry.file_type().is_ok_and(|ft| ft.is_symlink()) {
            continue;
        }
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.starts_with('.') {
            continue;
        }
        if super::mode_cmd::validate_mode_name(name).is_err() {
            continue;
        }
        crate::adapters::mcp::server::ensure_registered_mode(modes_dir, name);
    }
}

fn copy_executable(src: &str, dst: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(Path::new(dst).parent().unwrap_or(Path::new(".")))?;
    let data = std::fs::read(src)?;
    crate::config::atomic::write_atomic(dst, &data, 0o755)?;
    Ok(())
}

fn resolve_install_binary() -> anyhow::Result<(std::path::PathBuf, String, Option<TempFileCleanup>)>
{
    match update::install::download_latest_if_newer(crate::adapters::version::VALUE) {
        Ok(Some(result)) => {
            let cleanup = TempFileCleanup(result.cleanup_dir);
            Ok((result.path, result.version, Some(cleanup)))
        }
        Ok(None) => {
            let exec_path = std::env::current_exe()?;
            let version = update::check::display_version(crate::adapters::version::VALUE);
            Ok((exec_path, version, None))
        }
        Err(_) => {
            let exec_path = std::env::current_exe()?;
            let version = update::check::display_version(crate::adapters::version::VALUE);
            Ok((exec_path, version, None))
        }
    }
}

struct TempFileCleanup(std::path::PathBuf);

impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn path_contains_dir(path_env: &str, dir: &str) -> bool {
    let target = normalize_path_dir(dir);
    if target.is_empty() {
        return false;
    }
    for entry in std::env::split_paths(path_env) {
        if normalize_path_dir(&entry.to_string_lossy()) == target {
            return true;
        }
    }
    false
}

fn normalize_path_dir(dir: &str) -> String {
    if dir.is_empty() {
        return String::new();
    }
    let path = Path::new(dir);
    let resolved = std::fs::canonicalize(dir).unwrap_or_else(|_| path.to_path_buf());
    if resolved.is_absolute() {
        resolved
    } else {
        std::env::current_dir().unwrap_or_default().join(resolved)
    }
    .to_string_lossy()
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_contains_dir_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_str = dir.path().to_string_lossy().to_string();
        let path_env = format!("{}:/usr/bin", dir_str);
        assert!(path_contains_dir(&path_env, &dir_str));
    }

    #[test]
    fn test_path_contains_dir_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_str = dir.path().to_string_lossy().to_string();
        let path_env = "/usr/bin:/usr/local/bin".to_string();
        assert!(!path_contains_dir(&path_env, &dir_str));
    }

    #[test]
    fn test_path_contains_dir_empty() {
        assert!(!path_contains_dir("/usr/bin", ""));
    }
}
