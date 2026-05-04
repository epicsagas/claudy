use std::path::Path;

use anyhow::Context as _;

use crate::domain::context::Context;

pub fn run_mcp(ctx: &mut Context) -> anyhow::Result<i32> {
    crate::adapters::mcp::server::run_mcp_server(Path::new(&ctx.paths.config_file))
}

pub fn install_mcp(ctx: &mut Context) -> anyhow::Result<i32> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;
    let path = home.join(".claude.json");
    crate::adapters::mcp::server::ensure_registered(&path);

    // Also register in all existing modes
    register_all_modes(&ctx.paths.modes_dir);

    ctx.output
        .success("MCP server registered in Claude Code settings");
    Ok(0)
}

pub fn uninstall_mcp(ctx: &mut Context) -> anyhow::Result<i32> {
    let home = dirs::home_dir().context("Cannot determine home directory")?;
    let path = home.join(".claude.json");
    crate::adapters::mcp::server::unregister(&path);

    // Also unregister from all modes
    unregister_all_modes(&ctx.paths.modes_dir);

    ctx.output
        .success("MCP server removed from Claude Code settings");
    Ok(0)
}

fn for_each_mode<F>(modes_dir: &str, mut f: F)
where
    F: FnMut(&Path, &str),
{
    let modes_path = Path::new(modes_dir);
    if !modes_path.exists() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(modes_path) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with('.') {
                f(&entry.path(), &name);
            }
        }
    }
}

fn register_all_modes(modes_dir: &str) {
    for_each_mode(modes_dir, |_path, name| {
        crate::adapters::mcp::server::ensure_registered_mode(modes_dir, name);
    });
}

fn unregister_all_modes(modes_dir: &str) {
    for_each_mode(modes_dir, |path, _name| {
        let config = path.join(".claude.json");
        crate::adapters::mcp::server::unregister(&config);
    });
}
