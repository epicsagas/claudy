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

pub fn banner(name: &str, mode: Option<&str>) -> String {
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
    let palette_accent = palette[0];

    let mode_line = match mode {
        Some(m) => format!("\n  {dim}mode: {bright}{m}{reset}"),
        None => String::new(),
    };

    format!(
        "{art}  {palette_accent}{bright}Claudy{reset} with {bright}{name}{reset}{mode_line}\n\n  {dim}{tip}{reset}\n\n"
    )
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
    "symlink a profile name to launch it directly, e.g. claudy-ollama",
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
