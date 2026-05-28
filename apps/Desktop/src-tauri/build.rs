fn main() {
    println!("cargo:rerun-if-changed=WindowsAppManifest.xml");

    let windows_attributes =
        tauri_build::WindowsAttributes::new().app_manifest(include_str!("WindowsAppManifest.xml"));
    let attributes = tauri_build::Attributes::new().windows_attributes(windows_attributes);

    tauri_build::try_build(attributes).expect("failed to build Codex Tools");
}
