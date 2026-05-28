use std::path::Path;

use cargo_toml::Manifest;
use tauri_bundler::{PackageSettings, SettingsBuilder, bundle_project};

static CREATE_BUNDLES: Option<&str> = option_env!("CREATE_BUNDLES");

fn should_create_bundles() -> bool {
  CREATE_BUNDLES
    .map(|v| v == "true" || v == "1")
    .unwrap_or(false)
}

pub fn main() {
  if !should_create_bundles() {
    return;
  }

  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
  let manifest_path = Path::new(&manifest_dir).join("Cargo.toml");
  let manifest = Manifest::from_path(manifest_path).expect("Failed to read Cargo manifest");

  let package_settings = PackageSettings {
    product_name: "Orbolay".into(),
    version: env!("CARGO_PKG_VERSION").into(),
    description: manifest.package().description().unwrap_or_default().into(),
    authors: Some(manifest.package().authors().to_vec()),
    homepage: manifest.package().homepage().map(|s| s.into()),
    default_run: Some("orbolay".into()),
  };

  let settings = SettingsBuilder::new()
    .project_out_directory::<String>("target/bundles".into())
    .package_settings(package_settings)
    .no_sign(true)
    .build()
    .expect("Failed to build bundle settings");

  bundle_project(&settings).expect("Failed to create bundles");
}
