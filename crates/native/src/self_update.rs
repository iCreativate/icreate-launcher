//! iCreate updating itself.
//!
//! Delegated to [`craft_update::self_update`], with our GitHub repo and macOS bundle id.

use std::ffi::OsString;

use artcraft_launcher_core::catalog::{APP_BUNDLE_ID, APP_DISPLAY_NAME, LAUNCHER_ID, LAUNCHER_REPO};
use artcraft_launcher_core::platform::Platform;
use artcraft_launcher_core::protocol::JobPhase;
use artcraft_launcher_core::release::Release;
use artcraft_launcher_core::version::Version;

use crate::http::{Http, Stop};
use crate::paths::Paths;

pub use craft_update::self_update::{FINISH_FLAG, Install, PRETEND_VAR, is_self_updating};

/// The launcher as `craft_update` knows it: same asset stem, our GitHub repo and display name.
/// `bundle_id()` in craft-update is hardcoded as `ai.storyteller.<id>`, so macOS detection of
/// `za.co.icreativate.icreate` is wrapped in [`current`].
fn launcher_app() -> craft_update::App {
    craft_update::App {
        developer: "iCreativate",
        accent: Some([0x0e, 0xa5, 0xe9]),
        ..craft_update::App::new(LAUNCHER_ID, APP_DISPLAY_NAME, "Installs and updates the iCreate creative suite", LAUNCHER_REPO)
    }
}

/// The version this launcher counts as (see [`PRETEND_VAR`]).
pub fn current_version() -> Option<Version> {
    craft_update::self_update::current_version(&launcher_app(), env!("CARGO_PKG_VERSION"))
}

/// How this copy is installed, if it's one the launcher knows how to replace.
pub fn current() -> Option<Install> {
    let found = craft_update::self_update::current(&launcher_app());
    #[cfg(target_os = "macos")]
    {
        // craft-update only recognises `ai.storyteller.artcraft-launcher`. An iCreate.app
        // therefore looks like a loose binary; treat our bundle id as a macOS app so the
        // self-update still copies the new DMG over this one.
        if matches!(found, Some(Install::Standalone(_)))
            && let Some(mac) = macos_bundle_if_ours()
        {
            return Some(mac);
        }
    }
    found
}

#[cfg(target_os = "macos")]
fn macos_bundle_if_ours() -> Option<Install> {
    let exe = std::env::current_exe().ok()?;
    let exe = exe.canonicalize().unwrap_or_else(|_| exe);
    let bundle = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app"))?;
    let plist = plist::Value::from_file(bundle.join("Contents/Info.plist")).ok()?;
    let id = plist.as_dictionary()?.get("CFBundleIdentifier")?.as_string()?;
    (id == APP_BUNDLE_ID).then(|| Install::MacApp(bundle.to_path_buf()))
}

fn to_update_paths(paths: &Paths) -> craft_update::Paths {
    craft_update::Paths { data: paths.data.clone(), cache: paths.cache.clone() }
}

fn to_update_release(release: &Release) -> craft_update::Release {
    craft_update::Release {
        tag: release.tag.clone(),
        version: release.version.clone(),
        name: release.name.clone(),
        published_at: release.published_at,
        html_url: release.html_url.clone(),
        notes: release.notes.clone(),
        prerelease: release.prerelease,
        assets: release
            .assets
            .iter()
            .map(|a| craft_update::Asset { name: a.name.clone(), size: a.size, url: a.url.clone(), sha256: a.sha256.clone() })
            .collect(),
    }
}

fn to_update_platform(platform: Platform) -> craft_update::Platform {
    craft_update::Platform {
        os: match platform.os {
            artcraft_launcher_core::Os::Windows => craft_update::Os::Windows,
            artcraft_launcher_core::Os::MacOs => craft_update::Os::MacOs,
            artcraft_launcher_core::Os::Linux => craft_update::Os::Linux,
            artcraft_launcher_core::Os::FreeBsd => craft_update::Os::FreeBsd,
        },
        arch: match platform.arch {
            artcraft_launcher_core::Arch::X64 => craft_update::Arch::X64,
            artcraft_launcher_core::Arch::Arm64 => craft_update::Arch::Arm64,
            artcraft_launcher_core::Arch::X86 => craft_update::Arch::X86,
            artcraft_launcher_core::Arch::Riscv64 => craft_update::Arch::Riscv64,
        },
    }
}

/// Whether `release` is the version this launcher just updated itself to.
pub fn just_updated_to(paths: &Paths, release: &Release) -> bool {
    let update_paths = to_update_paths(paths);
    let update_release = to_update_release(release);
    craft_update::self_update::just_updated_to(&update_paths, &update_release)
}

/// The error of the last update, if it failed, once.
pub fn take_failure(paths: &Paths) -> Option<String> {
    let update_paths = to_update_paths(paths);
    craft_update::self_update::take_failure(&update_paths)
}

/// Downloads `release` for `platform`, puts it in place and starts the helper that relaunches
/// it once this process has exited. On success the caller must quit promptly. `last` is called
/// once the download is ready and returns when the launcher may replace itself and restart.
#[allow(clippy::too_many_arguments)]
pub fn update(
    install: &Install,
    release: &Release,
    platform: Platform,
    _http: &Http,
    paths: &Paths,
    stop: &Stop,
    phase: &mut dyn FnMut(JobPhase),
    last: &dyn Fn() -> Result<(), String>,
) -> Result<(), String> {
    let update_release = to_update_release(release);
    let update_platform = to_update_platform(platform);
    let update_paths = to_update_paths(paths);
    let update_http = craft_update::Http::new(concat!("iCreate/", env!("CARGO_PKG_VERSION")))?;
    let mut update_phase = |p: craft_update::JobPhase| {
        phase(match p {
            craft_update::JobPhase::Downloading { done, total } => JobPhase::Downloading { done, total },
            craft_update::JobPhase::Reconnecting { done, total } => JobPhase::Reconnecting { done, total },
            craft_update::JobPhase::Waiting => JobPhase::Waiting,
            craft_update::JobPhase::Verifying => JobPhase::Verifying,
            craft_update::JobPhase::Installing => JobPhase::Installing,
            craft_update::JobPhase::Removing => JobPhase::Removing,
        });
    };
    craft_update::self_update::update(
        &launcher_app(),
        install,
        &update_release,
        update_platform,
        &update_http,
        &update_paths,
        stop,
        &mut update_phase,
        last,
        Some(env!("CARGO_PKG_VERSION")),
    )
}

/// Finishes an update when this program was started as the update helper.
pub fn finish(args: &[OsString]) -> Option<Result<(), String>> {
    craft_update::self_update::finish(&launcher_app(), args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_development_build_does_not_update_itself() {
        assert_eq!(current(), None);
    }

    #[test]
    fn self_update_checks_icreativate_github() {
        assert_eq!(launcher_app().repo, "iCreativate/icreate-launcher");
        assert_eq!(launcher_app().name, "iCreate");
        assert_eq!(launcher_app().id, "icreate-launcher");
    }
}
