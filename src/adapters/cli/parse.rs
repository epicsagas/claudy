use crate::domain::context::Options;

#[derive(Debug, Clone)]
pub struct Parsed {
    pub options: Options,
    pub command: String,
    pub args: Vec<String>,
}

pub fn parse(args: &[String]) -> Result<Parsed, String> {
    let mut options = Options::default();
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => options.help = true,
            "-V" | "--version" => options.version = true,
            "--" => {
                positional.extend_from_slice(&args[i + 1..]);
                break;
            }
            _ => {
                if arg.starts_with('-') && positional.is_empty() {
                    return Err(format!("unknown option {}", arg));
                }
                positional.push(arg.clone());
            }
        }
        i += 1;
    }

    let (command, cmd_args) = if positional.is_empty() {
        (String::new(), Vec::new())
    } else {
        (positional[0].clone(), positional[1..].to_vec())
    };

    Ok(Parsed {
        options,
        command,
        args: cmd_args,
    })
}

pub fn parse_launcher(args: &[String]) -> (Options, Vec<String>) {
    let options = Options::default();
    let mut forwarded: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--" => {
                forwarded.extend_from_slice(&args[i + 1..]);
                break;
            }
            _ => {
                forwarded.push(arg.clone());
            }
        }
        i += 1;
    }

    (options, forwarded)
}
