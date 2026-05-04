use std::path::Path;

use crate::domain::context::Context;

pub fn run_uninstall(ctx: &mut Context) -> anyhow::Result<i32> {
    let ok = ctx.prompt.confirm("Remove all Claudy files?", false)?;
    if !ok {
        return Ok(0);
    }

    let _ = std::fs::remove_file(Path::new(&ctx.paths.bin_dir).join("claudy"));
    let _ = std::fs::remove_dir_all(&ctx.paths.config_dir);
    let _ = std::fs::remove_dir_all(&ctx.paths.data_dir);
    let _ = std::fs::remove_dir_all(&ctx.paths.cache_dir);

    ctx.output.write_line("Claudy uninstalled")?;
    Ok(0)
}
