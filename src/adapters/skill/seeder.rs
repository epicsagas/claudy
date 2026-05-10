use std::fs;
use std::path::Path;

/// Bundled skills to seed into ~/.claude/skills/.
/// Each entry is (skill_dir_name, SKILL.md content).
fn bundled_skills() -> Vec<(&'static str, &'static str)> {
    vec![(
        "analytics-insights",
        include_str!("bundled/analytics-insights.md"),
    )]
}

/// Install all bundled skills into the given skills directory.
/// Idempotent — skips skills whose content already matches.
/// Returns (installed, skipped) counts.
pub fn install_skills(skills_dir: &Path) -> (usize, usize) {
    let mut installed = 0;
    let mut skipped = 0;

    for (name, content) in bundled_skills() {
        let dir = skills_dir.join(name);
        let file = dir.join("SKILL.md");

        if file.exists()
            && let Ok(existing) = fs::read_to_string(&file)
            && existing == content
        {
            skipped += 1;
            continue;
        }

        if let Err(e) = fs::create_dir_all(&dir) {
            tracing::error!(skill = name, error = %e, "Failed to create skill directory");
            continue;
        }

        let Some(path_str) = file.to_str() else {
            tracing::error!(skill = name, "Skill path is not valid UTF-8, skipping");
            continue;
        };

        match crate::config::atomic::write_atomic(path_str, content.as_bytes(), 0o644) {
            Ok(()) => {
                tracing::info!(skill = name, "Installed skill");
                installed += 1;
            }
            Err(e) => {
                tracing::error!(skill = name, error = %e, "Failed to install skill");
            }
        }
    }

    (installed, skipped)
}

/// Remove all bundled skills from the given skills directory.
/// Returns count of removed skills.
pub fn uninstall_skills(skills_dir: &Path) -> usize {
    let mut removed = 0;

    for (name, _) in bundled_skills() {
        let dir = skills_dir.join(name);
        if dir.exists() {
            match fs::remove_dir_all(&dir) {
                Ok(()) => {
                    tracing::info!(skill = name, "Removed skill");
                    removed += 1;
                }
                Err(e) => {
                    tracing::error!(skill = name, error = %e, "Failed to remove skill");
                }
            }
        }
    }

    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_install_skills_creates_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let skills_dir = dir.path().join("skills");

        let (installed, skipped) = install_skills(&skills_dir);

        assert_eq!(installed, bundled_skills().len());
        assert_eq!(skipped, 0);

        for (name, content) in bundled_skills() {
            let file = skills_dir.join(name).join("SKILL.md");
            assert!(file.exists(), "skill {name} should exist");
            assert_eq!(fs::read_to_string(&file).unwrap(), content);
        }
    }

    #[test]
    fn test_install_skills_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let skills_dir = dir.path().join("skills");

        let (installed1, skipped1) = install_skills(&skills_dir);
        let (installed2, skipped2) = install_skills(&skills_dir);

        assert_eq!(installed1, bundled_skills().len());
        assert_eq!(skipped1, 0);
        assert_eq!(installed2, 0);
        assert_eq!(skipped2, bundled_skills().len());
    }

    #[test]
    fn test_install_skills_overwrites_changed_content() {
        let dir = tempfile::tempdir().expect("tempdir");
        let skills_dir = dir.path().join("skills");

        install_skills(&skills_dir);

        // Corrupt one skill file
        let (name, original) = bundled_skills()
            .into_iter()
            .next()
            .expect("at least one skill");
        let file = skills_dir.join(name).join("SKILL.md");
        fs::write(&file, "corrupted").unwrap();

        let (installed, skipped) = install_skills(&skills_dir);

        assert_eq!(installed, 1);
        assert_eq!(skipped, 0);
        assert_eq!(fs::read_to_string(&file).unwrap(), original);
    }

    #[test]
    fn test_uninstall_skills_removes_dirs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let skills_dir = dir.path().join("skills");

        install_skills(&skills_dir);
        let removed = uninstall_skills(&skills_dir);

        assert_eq!(removed, bundled_skills().len());
        for (name, _) in bundled_skills() {
            assert!(!skills_dir.join(name).exists());
        }
    }

    #[test]
    fn test_uninstall_skills_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let skills_dir = dir.path().join("skills");

        let removed = uninstall_skills(&skills_dir);
        assert_eq!(removed, 0);
    }
}
