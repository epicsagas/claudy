use std::io::{self, IsTerminal, Write};

use crate::ports::ui_ports::OutputPort;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Human,
    Json,
    Plain,
}

impl Format {
    pub fn parse(s: &str) -> Self {
        match s {
            "json" => Format::Json,
            "plain" => Format::Plain,
            _ => Format::Human,
        }
    }
}

pub struct Output {
    pub stdout: Box<dyn Write>,
    pub stderr: Box<dyn Write>,
    pub format: Format,
    pub quiet: bool,
    pub color: bool,
}

impl OutputPort for Output {
    fn header(&mut self, title: &str) {
        if self.format != Format::Human || self.quiet {
            return;
        }
        let styled = self.style("bold", title);
        let _ = writeln!(self.stdout, "{}", styled);
    }

    fn success(&mut self, msg: &str) {
        if self.quiet {
            return;
        }
        let label = if self.color {
            "\x1b[32m✓\x1b[0m"
        } else {
            "OK"
        };
        let _ = writeln!(self.stdout, "{} {}", label, msg);
    }

    fn warn(&mut self, msg: &str) {
        let label = if self.color {
            "\x1b[33m⚠\x1b[0m"
        } else {
            "WARN"
        };
        let _ = writeln!(self.stderr, "{} {}", label, msg);
    }

    fn error(&mut self, msg: &str) {
        let label = if self.color {
            "\x1b[31m✗\x1b[0m"
        } else {
            "ERR"
        };
        let _ = writeln!(self.stderr, "{} {}", label, msg);
    }

    fn info(&mut self, msg: &str) {
        if self.quiet {
            return;
        }
        let label = if self.color {
            "\x1b[34mℹ\x1b[0m"
        } else {
            "INFO"
        };
        let _ = writeln!(self.stdout, "{} {}", label, msg);
    }

    fn write_line(&mut self, msg: &str) -> std::io::Result<()> {
        writeln!(self.stdout, "{}", msg)
    }
}

impl Output {
    pub fn new(format: Format, quiet: bool) -> Self {
        let color = format == Format::Human
            && std::env::var("NO_COLOR").is_err()
            && io::stdout().is_terminal();
        Output {
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
            format,
            quiet,
            color,
        }
    }

    pub fn style(&self, kind: &str, input: &str) -> String {
        if !self.color {
            return input.to_string();
        }
        match kind {
            "bold" => format!("\x1b[1m{}\x1b[0m", input),
            "dim" => format!("\x1b[2m{}\x1b[0m", input),
            "green" => format!("\x1b[32m{}\x1b[0m", input),
            "yellow" => format!("\x1b[33m{}\x1b[0m", input),
            "red" => format!("\x1b[31m{}\x1b[0m", input),
            _ => input.to_string(),
        }
    }
}

pub fn banner(target: &crate::domain::launch_blueprint::LaunchTarget, mode: Option<&str>) -> String {
    use std::time::SystemTime;

    let tip = random_tip();

    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let palette = PALETTES[(seed as usize) % PALETTES.len()];

    let art = colorize_logo(palette);

    let bright = "\x1b[1m";
    let reset = "\x1b[0m";
    let dim = "\x1b[2m";
    let label_color = "\x1b[38;5;245m";   // muted (#707a8a)
    let accent_color = "\x1b[38;5;220m";  // yellow (#fcd535)
    let palette_accent = palette[0];

    // Title line
    let version = crate::adapters::version::VALUE;
    let title = format!(
        "  {palette_accent}{bright}Claudy{reset} with {bright}{}{reset} {dim}·{reset} {dim}v{version}{reset}",
        target.display_name,
    );

    // Info line 1: Profile, Provider, Mode
    let mode_val = mode.unwrap_or("default");
    let info1 = format!(
        "  {label_color}Profile{reset}   {bright}{:<10}{reset}{label_color}Provider{reset}   {bright}{:<10}{reset}{label_color}Mode{reset}   {bright}{}{reset}",
        target.profile,
        target.display_name,
        mode_val,
    );

    // Info line 2: Models
    let models = format_model_tiers(&target.model, &target.model_tiers);
    let info2 = format!(
        "  {label_color}Models{reset}    {bright}{}{reset}",
        models,
    );

    // Commands line
    let commands = format!(
        "  {accent_color}COMMANDS{reset}  {bright}ls{reset} · {bright}doctor{reset} · {bright}mode{reset} · {bright}mcp{reset} · {bright}channel{reset} · {bright}update{reset} · {bright}setup{reset} · {bright}ping{reset}"
    );

    // Tip line
    let tip_line = format!(
        "  {label_color}TIP{reset}       {dim}{}{reset}",
        tip,
    );

    format!(
        "\n{art}{title}\n\n{info1}\n{info2}\n\n{commands}\n{tip_line}\n\n"
    )
}

fn format_model_tiers(default: &str, tiers: &std::collections::HashMap<String, String>) -> String {
    if tiers.is_empty() {
        return default.to_string();
    }
    let mut entries: Vec<_> = tiers.iter().collect();
    entries.sort_by_key(|(k, _)| *k);
    let mut parts: Vec<String> = entries
        .iter()
        .map(|(tier, model)| {
            let short = shorten_model_name(model);
            format!("{} ({})", short, tier)
        })
        .collect();
    // Ensure the default model is included even if not in tiers
    if !tiers.values().any(|v| v == default) && !default.is_empty() {
        let short = shorten_model_name(default);
        parts.insert(0, format!("{} (default)", short));
    }
    parts.join(", ")
}

fn shorten_model_name(model: &str) -> &str {
    // Extract short name from full model IDs like "claude-sonnet-4-6-20250514"
    if let Some(rest) = model.strip_prefix("claude-") {
        // Keep up to the tier name: "sonnet-4.6", "opus-4.7", "haiku-4.5"
        let mut parts: Vec<&str> = rest.split('-').collect();
        if parts.len() >= 3 {
            // Remove the date suffix (e.g., "20250514")
            while parts.len() > 2 {
                if parts.last().map(|p| p.starts_with(char::is_numeric)).unwrap_or(false) && parts.last().map(|p| p.len() == 8).unwrap_or(false) {
                    parts.pop();
                } else {
                    break;
                }
            }
            let tier = parts[0];
            let version = parts[1..].join(".");
            return Box::leak(format!("{}.{}", tier, version).into_boxed_str());
        }
        return rest;
    }
    model
}

const PALETTES: &[&[&str]] = &[
    &[
        "\x1b[38;5;213m",
        "\x1b[38;5;205m",
        "\x1b[38;5;199m",
        "\x1b[38;5;163m",
        "\x1b[38;5;163m",
        "\x1b[38;5;127m",
    ],
    &[
        "\x1b[38;5;159m",
        "\x1b[38;5;123m",
        "\x1b[38;5;87m",
        "\x1b[38;5;79m",
        "\x1b[38;5;73m",
        "\x1b[38;5;67m",
    ],
    &[
        "\x1b[38;5;183m",
        "\x1b[38;5;177m",
        "\x1b[38;5;141m",
        "\x1b[38;5;135m",
        "\x1b[38;5;129m",
        "\x1b[38;5;99m",
    ],
    &[
        "\x1b[38;5;223m",
        "\x1b[38;5;222m",
        "\x1b[38;5;215m",
        "\x1b[38;5;179m",
        "\x1b[38;5;178m",
        "\x1b[38;5;173m",
    ],
    &[
        "\x1b[38;5;194m",
        "\x1b[38;5;157m",
        "\x1b[38;5;150m",
        "\x1b[38;5;114m",
        "\x1b[38;5;108m",
        "\x1b[38;5;72m",
    ],
    &[
        "\x1b[38;5;189m",
        "\x1b[38;5;153m",
        "\x1b[38;5;117m",
        "\x1b[38;5;111m",
        "\x1b[38;5;105m",
        "\x1b[38;5;69m",
    ],
];

const TIPS: &[&str] = &[
    "Use --yolo instead of --dangerously-skip-permissions",
    "use 'claudy ls' to see all available profiles",
    "symlink a profile name to launch it directly, e.g. claudy ollama",
    "run 'claudy update' to check for a newer version",
    "set NO_COLOR=1 to disable colored output",
    "use -q or --quiet to suppress the banner and tips",
    "run 'claudy doctor' to check your configuration and paths",
    "try 'claudy setup' to configure a new provider interactively",
    "use 'claudy mode create <name>' to isolate Claude config per project",
    "run 'claudy ping <profile>' to test provider connectivity",
    "use 'claudy show <profile>' to inspect resolved provider details",
    "add 'claudy mcp' to Claude Code settings to delegate tasks to other AI agents",
    "use 'claudy mcp' to expose Gemini, Codex, Aider, etc. as MCP tools",
    "try 'claudy channel start' to run a Telegram/Slack/Discord bot bridge",
    "use modes to switch between work/personal Claude configurations",
    "run 'claudy channel add telegram' to set up a Telegram bot",
    "custom agents go in config.yaml under 'agents' — any CLI with headless mode",
    "run 'claudy sync' to install or update the claudy binary",
];

fn random_tip() -> &'static str {
    use std::time::SystemTime;
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    TIPS[(seed as usize) % TIPS.len()]
}

fn colorize_logo(palette: &[&str]) -> String {
    // Slant font — each line gets its own color from the palette for a gradient effect
    let lines: &[&str] = &[
        "        __                __     ",
        "  _____/ /___ ___  ______/ /_  __",
        " / ___/ / __ `/ / / / __  / / / /",
        "/ /__/ / /_/ / /_/ / /_/ / /_/ / ",
        "\\___/_/\\__,_/\\__,_/\\__,_/\\__, /  ",
        "                        /____/   ",
    ];

    let reset = "\x1b[0m";
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let c = palette[i % palette.len()];
        out.push_str(c);
        out.push_str(line);
        out.push_str(reset);
        out.push('\n');
    }
    out
}
