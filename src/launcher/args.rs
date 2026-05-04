/// Expand CLI shorthand flags and deduplicate their long forms.
///
/// `--yolo` → `--dangerously-skip-permissions` (deduped if both present).
pub fn normalize_claude_args(args: &[String]) -> Vec<String> {
    args.iter()
        .fold(
            (false, Vec::with_capacity(args.len())),
            |(mut seen_danger, mut acc), arg| {
                match arg.as_str() {
                    "--yolo" if !seen_danger => {
                        acc.push("--dangerously-skip-permissions".to_owned());
                        seen_danger = true;
                    }
                    "--yolo" | "--dangerously-skip-permissions" if seen_danger => {}
                    other => {
                        if other == "--dangerously-skip-permissions" {
                            seen_danger = true;
                        }
                        acc.push(other.to_owned());
                    }
                }
                (seen_danger, acc)
            },
        )
        .1
}

/// Extract the value of `--model <VAL>` or `--model=<VAL>` from args.
pub fn model_override(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == "--model")
        .map(|w| w[1].trim().to_owned())
        .or_else(|| {
            args.iter()
                .find_map(|a| a.strip_prefix("--model=").map(|v| v.trim().to_owned()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_no_yolo() {
        let args: Vec<String> = vec!["--model".to_string(), "opus".to_string()];
        let result = normalize_claude_args(&args);
        assert_eq!(result, args);
    }

    #[test]
    fn test_normalize_yolo() {
        let args: Vec<String> = vec![
            "--yolo".to_string(),
            "--model".to_string(),
            "opus".to_string(),
        ];
        let result = normalize_claude_args(&args);
        assert_eq!(
            result,
            vec!["--dangerously-skip-permissions", "--model", "opus"]
        );
    }

    #[test]
    fn test_normalize_already_dangerous() {
        let args: Vec<String> = vec!["--dangerously-skip-permissions".to_string()];
        let result = normalize_claude_args(&args);
        assert_eq!(result, vec!["--dangerously-skip-permissions"]);
    }

    #[test]
    fn test_normalize_yolo_and_dangerous() {
        let args: Vec<String> = vec![
            "--yolo".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ];
        let result = normalize_claude_args(&args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "--dangerously-skip-permissions");
    }

    #[test]
    fn test_model_override_long() {
        let args: Vec<String> = vec!["--model".to_string(), "opus".to_string()];
        assert_eq!(model_override(&args), Some("opus".to_string()));
    }

    #[test]
    fn test_model_override_equals() {
        let args: Vec<String> = vec!["--model=sonnet".to_string()];
        assert_eq!(model_override(&args), Some("sonnet".to_string()));
    }

    #[test]
    fn test_model_override_none() {
        let args: Vec<String> = vec!["--verbose".to_string()];
        assert_eq!(model_override(&args), None);
    }
}
