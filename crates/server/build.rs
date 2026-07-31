use std::fs;
use std::path::Path;

fn main() {
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
  let version = env!("CARGO_PKG_VERSION");

  let workspace_root = Path::new(&manifest_dir)
    .parent()
    .and_then(|p| p.parent())
    .expect("build script: CARGO_MANIFEST_DIR should be under crates/<name>");

  fs::write(workspace_root.join("version"), version)
    .expect("failed to write version file");
}
