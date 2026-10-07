use std::path::Path;
use std::process::Command;

fn main() {
    let ui_dist = Path::new("../../ui/dist/index.html");
    if !ui_dist.exists() {
        let status = Command::new("pnpm")
            .args(["--dir", "../../ui", "build"])
            .status()
            .or_else(|_| {
                Command::new("npm")
                    .args(["--prefix", "../../ui", "run", "build"])
                    .status()
            });

        if let Ok(s) = status {
            if !s.success() {
                eprintln!("cargo:warning=Failed to build UI assets with pnpm/npm");
            }
        }
    }

    tauri_build::build();
}
