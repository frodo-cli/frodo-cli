use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use flate2::read::GzDecoder;
use reqwest::{header::USER_AGENT, Client};
use serde::Deserialize;
use tar::Archive;
use tracing::info;

/// Run self-update: check latest release, optionally download and replace binary.
pub async fn run(check_only: bool) -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    let release = fetch_latest_release().await?;
    info!("current version: {}, latest: {}", current, release.tag_name);

    if release.tag_name.trim_start_matches('v') == current {
        println!("Already up to date ({}).", current);
        return Ok(());
    }

    println!("Update available: {} -> {}", current, release.tag_name);
    if check_only {
        println!("Use `frodo self-update` to apply.");
        return Ok(());
    }

    let asset = select_asset(&release).ok_or_else(|| {
        color_eyre::eyre::eyre!("no compatible asset found for this platform; aborting")
    })?;
    let tmp = download(&asset.browser_download_url).await?;
    install(&tmp)?;
    println!("Updated to {}", release.tag_name);
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

async fn fetch_latest_release() -> Result<Release> {
    let client = Client::builder().build()?;
    let url = "https://api.github.com/repos/frodo-cli/frodo-cli/releases/latest";
    let release = client
        .get(url)
        .header(USER_AGENT, "frodo-cli")
        .send()
        .await?
        .error_for_status()?
        .json::<Release>()
        .await?;
    Ok(release)
}

fn select_asset(release: &Release) -> Option<&Asset> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let expected = format!("frodo-{os}-{arch}.tar.gz");
    release.assets.iter().find(|a| a.name == expected)
}

async fn download(url: &str) -> Result<PathBuf> {
    let client = Client::builder().build()?;
    let mut resp = client
        .get(url)
        .header(USER_AGENT, "frodo-cli")
        .send()
        .await?;
    resp.error_for_status_ref()?;

    let mut tmp = tempfile::NamedTempFile::new()?;
    while let Some(chunk) = resp.chunk().await? {
        tmp.write_all(&chunk)?;
    }
    Ok(tmp.into_temp_path().to_path_buf())
}

fn install(tarball: &Path) -> Result<()> {
    let exe = env::current_exe()?;
    let exe_name = exe
        .file_name()
        .ok_or_else(|| color_eyre::eyre::eyre!("cannot resolve current exe name"))?;

    let file = File::open(tarball)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    let mut extracted: Option<PathBuf> = None;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        if path.file_name() == Some(exe_name) {
            let dest = exe.with_extension("new");
            entry.unpack(&dest)?;
            extracted = Some(dest);
            break;
        }
    }

    let new_bin = extracted.ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "could not find binary {} in archive",
            exe_name.to_string_lossy()
        )
    })?;

    // Backup current binary.
    let backup = exe.with_extension("old");
    if let Err(err) = fs::rename(&exe, &backup) {
        info!("backup failed (continuing): {err}");
    }

    // Replace.
    fs::rename(&new_bin, &exe)?;
    Ok(())
}
