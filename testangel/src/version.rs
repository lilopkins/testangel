use std::env;

pub async fn check_is_latest() -> bool {
    tracing::debug!("Checking version");
    if env::var("TA_SKIP_VERSION_CHECK")
        .unwrap_or("no".to_string())
        .eq_ignore_ascii_case("yes")
    {
        // Skip check.
        tracing::info!(
            "Version check skipped. Current version: {}",
            env!("CARGO_PKG_VERSION")
        );
        return true;
    }

    tracing::debug!("Getting latest release");
    if let Ok(latest_release) = octocrab::instance()
        .repos("lilopkins", "testangel")
        .releases()
        .get_latest()
        .await
    {
        if let Ok(tag) = semver::Version::parse(&latest_release.tag_name) {
            if let Ok(current) = semver::Version::parse(env!("CARGO_PKG_VERSION")) {
                tracing::info!("Latest version: {tag} Current version: {current}");
                tag <= current
            } else {
                tracing::warn!(
                    "Couldn't parse current version: '{}'",
                    env!("CARGO_PKG_VERSION")
                );
                false
            }
        } else {
            tracing::warn!(
                "Couldn't parse remote version: '{}'. Current version: {}",
                latest_release.tag_name,
                env!("CARGO_PKG_VERSION")
            );
            false
        }
    } else {
        // Probably offline
        tracing::warn!(
            "Couldn't fetch latest release for version check! Current version: {}",
            env!("CARGO_PKG_VERSION")
        );
        true
    }
}
