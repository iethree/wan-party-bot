//! Bakes a build-time version number into the binary: the total number of commits
//! reachable from HEAD (`git rev-list --count HEAD`). Exposed as the `BUILD_VERSION`
//! compile-time env var, read via `env!` in `src/lib.rs`.

use std::process::Command;

fn main() {
    let version = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_VERSION={version}");

    // Recompute when a new commit lands (or the branch changes) so the baked-in
    // number stays current across rebuilds. `.git/logs/HEAD` is appended on every
    // commit; `.git/HEAD` changes on a branch switch.
    println!("cargo:rerun-if-changed=.git/logs/HEAD");
    println!("cargo:rerun-if-changed=.git/HEAD");
}
