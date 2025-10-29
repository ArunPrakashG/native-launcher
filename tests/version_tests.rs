#[test]
fn crate_version_matches_version_file() {
    let crate_version = env!("CARGO_PKG_VERSION");
    let version_file = include_str!("../VERSION").trim();
    assert_eq!(
        crate_version, version_file,
        "CARGO_PKG_VERSION ({}) does not match VERSION file ({})",
        crate_version, version_file
    );
}
