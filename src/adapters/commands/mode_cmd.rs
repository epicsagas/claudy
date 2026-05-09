use std::path::Path;

use crate::domain::context::Context;

fn warn_deprecated(old: &str, new: &str) {
    eprintln!(
        "[deprecated] '{}' is deprecated, use '{}' instead.",
        old, new
    );
}

pub fn run_mode(ctx: &mut Context, action: &str, name: Option<&str>) -> anyhow::Result<i32> {
    match action {
        "create" => run_mode_create(ctx, name),
        "ls" => {
            warn_deprecated("mode ls", "mode list");
            run_mode_ls(ctx)
        }
        "list" => run_mode_ls(ctx),
        "rm" => {
            warn_deprecated("mode rm", "mode remove");
            run_mode_rm(ctx, name)
        }
        "remove" => run_mode_rm(ctx, name),
        _ => {
            anyhow::bail!(
                "Unknown mode action '{}'. Use: create, list, remove",
                action
            )
        }
    }
}

pub fn validate_mode_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        anyhow::bail!("Mode name cannot be empty.");
    }
    let bytes = name.as_bytes();
    let first = bytes[0];
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        anyhow::bail!(
            "Invalid mode name '{}': must start with a lowercase letter or digit.",
            name
        );
    }
    if !bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-' || *b == b'_')
    {
        anyhow::bail!(
            "Invalid mode name '{}': must match [a-z0-9][a-z0-9_-]*.",
            name
        );
    }
    if name == "mode" {
        anyhow::bail!("Mode name 'mode' is reserved.");
    }
    Ok(())
}

fn run_mode_create(ctx: &mut Context, name: Option<&str>) -> anyhow::Result<i32> {
    let name = name.ok_or_else(|| anyhow::anyhow!("Usage: claudy mode create <name>"))?;
    validate_mode_name(name)?;

    let mode_dir = Path::new(&ctx.paths.modes_dir).join(name);
    if mode_dir.exists() {
        ctx.output.warn(&format!(
            "Mode '{}' already exists at {}",
            name,
            mode_dir.display()
        ));
        return Ok(0);
    }

    std::fs::create_dir_all(&mode_dir)?;

    crate::adapters::mcp::server::ensure_registered_mode(&ctx.paths.modes_dir, name);

    ctx.output.success(&format!(
        "Created mode '{}' at {}",
        name,
        mode_dir.display()
    ));
    ctx.output
        .info("Place CLAUDE.md, settings.json, or other Claude config files in this directory.");
    Ok(0)
}

fn run_mode_ls(ctx: &mut Context) -> anyhow::Result<i32> {
    let modes_path = Path::new(&ctx.paths.modes_dir);
    if !modes_path.exists() {
        ctx.output
            .info("No modes configured. Use 'claudy mode create <name>' to create one.");
        return Ok(0);
    }

    let mut entries: Vec<String> = std::fs::read_dir(modes_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| !n.starts_with('.'))
        .collect();

    if entries.is_empty() {
        ctx.output
            .info("No modes configured. Use 'claudy mode create <name>' to create one.");
        return Ok(0);
    }

    entries.sort();
    ctx.output.header("Modes");
    for name in &entries {
        ctx.output.info(&format!("  {}", name));
    }
    Ok(0)
}

fn run_mode_rm(ctx: &mut Context, name: Option<&str>) -> anyhow::Result<i32> {
    let name = name.ok_or_else(|| anyhow::anyhow!("Usage: claudy mode rm <name>"))?;
    validate_mode_name(name)?;

    let mode_dir = Path::new(&ctx.paths.modes_dir).join(name);
    if !mode_dir.exists() {
        ctx.output.warn(&format!("Mode '{}' does not exist.", name));
        return Ok(0);
    }

    let confirmed = ctx
        .prompt
        .confirm(&format!("Remove mode '{}' and all its files?", name), false)?;
    if !confirmed {
        ctx.output.info("Cancelled.");
        return Ok(0);
    }

    std::fs::remove_dir_all(&mode_dir)?;
    ctx.output.success(&format!("Removed mode '{}'.", name));
    Ok(0)
}
