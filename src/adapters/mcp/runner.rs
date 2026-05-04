use std::path::Path;
use std::time::Duration;

use crate::domain::agent::AgentDefinition;

const MAX_CAPTURE: usize = 10 * 1024 * 1024; // 10 MB

/// Run an agent in headless mode, capturing stdout.
pub async fn run_agent(
    def: &AgentDefinition,
    prompt: &str,
    cwd: Option<&Path>,
) -> anyhow::Result<String> {
    let args: Vec<String> = def
        .args
        .iter()
        .map(|a| {
            if a == "{prompt}" {
                prompt.to_string()
            } else {
                a.clone()
            }
        })
        .collect();

    let mut cmd = tokio::process::Command::new(&def.binary);
    cmd.args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to execute agent '{}': {}", def.name, e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let status = tokio::select! {
        status = child.wait() => {
            status.map_err(|e| {
                anyhow::anyhow!("Failed to execute agent '{}': {}", def.name, e)
            })?
        }
        _ = tokio::time::sleep(Duration::from_secs(def.timeout)) => {
            // Timed out — kill the child and reap it.
            let _ = child.kill().await;
            let _ = child.wait().await;
            anyhow::bail!("Agent '{}' timed out after {}s", def.name, def.timeout);
        }
    };

    if status.success() {
        let stdout = match stdout {
            Some(out) => {
                let mut buf = Vec::new();
                use tokio::io::AsyncReadExt;
                let _ = out.take(MAX_CAPTURE as u64).read_to_end(&mut buf).await;
                buf
            }
            None => Vec::new(),
        };
        Ok(String::from_utf8_lossy(&stdout).to_string())
    } else {
        let stderr = match stderr {
            Some(err) => {
                let mut buf = Vec::new();
                use tokio::io::AsyncReadExt;
                let _ = err.take(MAX_CAPTURE as u64).read_to_end(&mut buf).await;
                String::from_utf8_lossy(&buf).to_string()
            }
            None => String::new(),
        };
        let code = status.code().unwrap_or(-1);
        anyhow::bail!(
            "Agent '{}' exited with code {}: {}",
            def.name,
            code,
            stderr.trim()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::agent::AgentDefinition;

    #[tokio::test]
    async fn test_run_agent_echo() {
        let def = AgentDefinition {
            name: "echo-test".into(),
            binary: "echo".into(),
            args: vec!["{prompt}".into()],
            description: "Echo agent for testing".into(),
            timeout: 10,
        };

        let result = run_agent(&def, "hello world", None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello world"));
    }

    #[tokio::test]
    async fn test_run_agent_timeout() {
        let def = AgentDefinition {
            name: "slow-agent".into(),
            binary: "sleep".into(),
            args: vec!["60".into()],
            description: "Slow agent for timeout testing".into(),
            timeout: 1,
        };

        let result = run_agent(&def, "ignored", None).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("timed out"));
    }
}
