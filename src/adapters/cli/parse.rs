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

/// Split a standalone `--guard` token out of args before the first `--`.
/// `claudy zai -- --guard` forwards the flag to the claude CLI verbatim.
pub fn split_guard_flag(args: &[String]) -> (bool, Vec<String>) {
    let mut guard = false;
    let mut rest = Vec::with_capacity(args.len());
    for (i, arg) in args.iter().enumerate() {
        if arg == "--" {
            rest.extend_from_slice(&args[i..]);
            break;
        }
        if arg == "--guard" {
            guard = true;
        } else {
            rest.push(arg.clone());
        }
    }
    (guard, rest)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn guard_flag_stripped_before_profile() {
        let (guard, rest) = split_guard_flag(&args(&["zai", "--guard", "--yolo"]));
        assert!(guard);
        assert_eq!(rest, args(&["zai", "--yolo"]));
    }

    #[test]
    fn guard_flag_flag_first_removed() {
        let (guard, rest) = split_guard_flag(&args(&["--guard", "zai"]));
        assert!(guard);
        assert_eq!(rest, args(&["zai"]));
    }

    #[test]
    fn guard_flag_not_stripped_after_double_dash() {
        let (guard, rest) = split_guard_flag(&args(&["zai", "--", "--guard"]));
        assert!(!guard);
        assert_eq!(rest, args(&["zai", "--", "--guard"]));
    }
}
