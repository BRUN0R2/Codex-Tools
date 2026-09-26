use std::fs;
use std::path::Path;

const PACKAGE_MANIFEST_FILE_NAME: &str = "AppxManifest.xml";
const PACKAGE_NAMESPACE: &str = "http://schemas.microsoft.com/appx/manifest/foundation/windows10";
const RESTRICTED_CAPABILITY_NAMESPACE: &str =
    "http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities";
const ELEVATION_CAPABILITY_NAME: &str = "allowElevation";

pub fn codex_package_allows_elevation(executable_path: &Path) -> Result<bool, String> {
    let package_directory = executable_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "Codex executable is not inside a package directory.".to_owned())?;
    let manifest_path = package_directory.join(PACKAGE_MANIFEST_FILE_NAME);
    let manifest = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "Failed to read Codex package manifest {}: {error}",
            manifest_path.display()
        )
    })?;

    package_manifest_allows_elevation(&manifest).map_err(|error| {
        format!(
            "Failed to inspect Codex package manifest {}: {error}",
            manifest_path.display()
        )
    })
}

fn package_manifest_allows_elevation(manifest: &str) -> Result<bool, String> {
    let document = roxmltree::Document::parse(manifest).map_err(|error| error.to_string())?;
    let package = document.root_element();
    if !package.has_tag_name((PACKAGE_NAMESPACE, "Package")) {
        return Err("The package manifest has no Package root element.".to_owned());
    }

    let capabilities = package
        .children()
        .find(|node| node.has_tag_name((PACKAGE_NAMESPACE, "Capabilities")))
        .ok_or_else(|| "The package manifest has no Capabilities element.".to_owned())?;

    Ok(capabilities.children().any(|node| {
        node.has_tag_name((RESTRICTED_CAPABILITY_NAMESPACE, "Capability"))
            && node.attribute("Name") == Some(ELEVATION_CAPABILITY_NAME)
    }))
}

#[cfg(test)]
mod tests {
    use super::package_manifest_allows_elevation;

    const MANIFEST_PREFIX: &str = r#"<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10" xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"><Capabilities>"#;
    const MANIFEST_SUFFIX: &str = "</Capabilities></Package>";

    #[test]
    fn elevation_requires_the_restricted_package_capability() {
        let without_elevation = format!(
            "{MANIFEST_PREFIX}<rescap:Capability Name=\"runFullTrust\" />{MANIFEST_SUFFIX}"
        );
        let with_elevation = format!(
            "{MANIFEST_PREFIX}<rescap:Capability Name=\"allowElevation\" />{MANIFEST_SUFFIX}"
        );

        assert_eq!(
            package_manifest_allows_elevation(&without_elevation),
            Ok(false)
        );
        assert_eq!(package_manifest_allows_elevation(&with_elevation), Ok(true));
    }
}
