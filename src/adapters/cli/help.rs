use std::io;

use crate::adapters::version;
use crate::ports::ui_ports::OutputPort;
use crate::providers::index::ProviderIndex;

pub fn show_brief(w: &mut dyn OutputPort) -> io::Result<()> {
    w.write_line(&format!(
        "Claudy v{} - Multi-provider launcher for Claude CLI",
        version::VALUE
    ))?;
    w.write_line("")?;
    w.write_line("Usage: claudy [options] <command>")?;
    w.write_line("")?;
    w.write_line("Commands:")?;
    w.write_line("  config       Configure a provider")?;
    w.write_line("  list         List profiles")?;
    w.write_line("  info         Provider details")?;
    w.write_line("  test         Test providers")?;
    w.write_line("  status       Show installation state")?;
    w.write_line("  update       Update to latest version")?;
    w.write_line("  mode         Manage Claude config modes")?;
    w.write_line("  channel       Manage the remote code channel")?;
    w.write_line("  uninstall    Remove Claudy")?;
    w.write_line("")?;
    w.write_line("Tip: add --yolo to a provider command to skip permission prompts.")?;
    w.write_line("")?;
    w.write_line("Run claudy --help for full help.")?;
    Ok(())
}

pub fn show_full(w: &mut dyn OutputPort, catalog: &ProviderIndex) -> io::Result<()> {
    w.write_line(&format!("Claudy v{}", version::VALUE))?;
    w.write_line("Multi-provider launcher for Claude CLI")?;
    w.write_line("")?;
    w.write_line("Usage:")?;
    w.write_line("  claudy [options] <command> [args]")?;
    w.write_line("  claudy <provider> [args]     Launch a provider")?;
    w.write_line("  claudy <provider> <mode>     Launch with a config mode")?;
    w.write_line("")?;
    w.write_line("Commands:")?;
    w.write_line("  config [provider]")?;
    w.write_line("  list")?;
    w.write_line("  info <provider>")?;
    w.write_line("  test [provider]")?;
    w.write_line("  status")?;
    w.write_line("  install")?;
    w.write_line("  update")?;
    w.write_line("  mode create <name>          Create a config mode")?;
    w.write_line("  mode ls                     List modes")?;
    w.write_line("  mode rm <name>              Remove a mode")?;
    w.write_line("  uninstall")?;
    w.write_line("")?;
    w.write_line("Options:")?;
    w.write_line("  -h, --help")?;
    w.write_line("  -V, --version")?;
    w.write_line("")?;
    w.write_line("Examples:")?;
    w.write_line("  claudy zai                  Launch Claude with ZAI provider")?;
    w.write_line("  claudy ollama               Launch Claude with Ollama")?;
    w.write_line("  claudy zai --yolo           Skip permission prompts")?;
    w.write_line("  claudy zai work --yolo      Use 'work' mode with ZAI provider")?;
    w.write_line("  claudy or <alias>           Launch via OpenRouter alias")?;
    w.write_line("")?;
    w.write_line("Providers:")?;
    for category in catalog.categories() {
        w.write_line(&format!("  {}", category))?;
        let providers_in_category = catalog.providers_by_category(&category);
        let mut providers_in_category: Vec<_> = providers_in_category.into_iter().collect();
        providers_in_category.sort_by(|a, b| a.id.cmp(&b.id));
        for provider in providers_in_category {
            w.write_line(&format!("    {:<12} {}", provider.id, provider.description))?;
        }
    }
    w.write_line("")?;
    w.write_line("Advanced:")?;
    w.write_line("    or <alias>    OpenRouter model via native API")?;
    w.write_line("    custom-name   Anthropic-compatible endpoint")?;
    Ok(())
}
