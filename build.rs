fn main() {
    #[cfg(feature = "analytics-ui")]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set by cargo");
        let ui_dir = std::path::Path::new(&manifest_dir).join("ui");
        let dist_dir = ui_dir.join("dist");

        if !dist_dir.join("assets").exists() && ui_dir.join("package.json").exists() {
            let status = std::process::Command::new("npm")
                .arg("ci")
                .current_dir(&ui_dir)
                .status()
                .expect("failed to run npm ci");
            assert!(status.success(), "npm ci failed for analytics dashboard");

            let status = std::process::Command::new("npm")
                .arg("run")
                .arg("build")
                .current_dir(&ui_dir)
                .status()
                .expect("failed to run npm build");
            assert!(status.success(), "npm build failed for analytics dashboard");
        }

        tauri_build::build();
    }
}
