//! The iCreate app catalogue: the apps the launcher knows about.
//!
//! The list lives in `apps.json` at the repo root so names, repos, status and categories can be
//! edited without touching Rust. [`suite`] is the compiled-in copy. Swap an app's `repo` there to
//! install from a fork; id, former names and macOS team stay the keys install, update and checksum
//! matching already use.
//!
//! Text here is English, the source the UI translates from: each `tagline` has a catalog row
//! keyed by the app id (`tr_ctx(app.id, app.tagline)` in `ui-egui`).

use std::sync::OnceLock;

use serde::Deserialize;

/// Who makes an app. Official apps live under [`GITHUB_OWNER`] unless `apps.json` points `repo`
/// at a fork; a community app is maintained outside ArtCraft and is labelled as such wherever it
/// is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppKind {
    Official,
    Community,
}

/// Home-screen grouping. Values match `apps.json` `category`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Photo,
    Design,
    Video,
    Audio,
    Documents,
    Office,
    /// Productivity / work apps (Email, Calendar, Notes, …), separate from Craft creative apps.
    Work,
}

impl Category {
    pub const ALL: [Self; 7] =
        [Self::Photo, Self::Design, Self::Video, Self::Audio, Self::Documents, Self::Office, Self::Work];

    pub fn label(self) -> &'static str {
        match self {
            Self::Photo => "Photo",
            Self::Design => "Design",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Documents => "Documents",
            Self::Office => "Office",
            Self::Work => "Work",
        }
    }
}

/// How far along an app is. Values match `apps.json` `status`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maturity {
    Stable,
    Beta,
    Alpha,
    /// Announced in the catalogue; no installable build yet.
    Planned,
    /// Earlier than Planned; exploring foundations and scope.
    Research,
}

impl Maturity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Beta => "Beta",
            Self::Alpha => "Alpha",
            Self::Planned => "Planned",
            Self::Research => "Research",
        }
    }

    /// Alpha apps get the early-preview warning badge.
    pub fn is_early(self) -> bool {
        matches!(self, Self::Alpha)
    }

    /// Catalogue-only entries: never offer Install or Download.
    pub fn is_catalogue_only(self) -> bool {
        matches!(self, Self::Planned | Self::Research)
    }
}

/// One app in the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftApp {
    /// Machine name: repo, binary, package and asset stem (`photocraft`).
    pub id: &'static str,
    /// Installer and GitHub project name (`PhotoCraft`). Matching still keys off this.
    pub name: &'static str,
    /// iCreate title as stored in `apps.json` (`iCreate Photo (powered by PhotoCraft)`).
    pub display_name: &'static str,
    /// Adobe, Microsoft or other product this app stands in for.
    pub replaces: &'static str,
    /// What it's for, in a few words (English; the UI translates it by app id).
    pub tagline: &'static str,
    /// Icon file in `assets/app-icons` (`photocraft.png`).
    pub icon: &'static str,
    /// Brand colour (sRGB), for surfaces: getartcraft.com's `--app`.
    pub color: [u8; 3],
    /// The brand colour for text on a light background (the site's light `--app-ink`).
    pub ink_light: [u8; 3],
    /// The brand colour for text on a dark background (the site's dark `--app-ink`).
    pub ink_dark: [u8; 3],
    /// GitHub `owner/repo`. Edit this in `apps.json` to install from a fork.
    pub repo: &'static str,
    /// Older names its release assets may carry (PdfCraft shipped as PrintCraft until 0.2.1).
    pub former_ids: &'static [&'static str],
    /// Older display names its installers registered (PdfCraft's MSI said PrintCraft until 0.2.1).
    pub former_names: &'static [&'static str],
    pub kind: AppKind,
    /// Who publishes it, as the card's byline shows it (`ArtCraft`, `Tangency Labs`).
    pub publisher: &'static str,
    /// The publisher its Windows installer names, when it isn't ArtCraft's (`Learning Machines LLC`).
    pub msi_publisher: Option<&'static str>,
    /// The bundle id when it isn't `ai.storyteller.<id>` (a community app brings its own).
    pub bundle: Option<&'static str>,
    /// Older bundle ids its Flatpak shipped under (SolidCraft was `ai.storyteller.solidcraft`).
    pub former_bundles: &'static [&'static str],
    /// Its website, when it isn't a page on getartcraft.com (a community app has its own).
    pub site: Option<&'static str>,
    /// The Apple Developer team whose Developer ID signature its macOS build must carry. `None`
    /// while those builds carry no signature the launcher can check: it then opens a copy that's
    /// there and links to the releases, but installs nothing.
    pub mac_team: Option<&'static str>,
    pub category: Category,
    pub maturity: Maturity,
    /// SPDX-style or short licence label shown on the details page (`MPL-2.0`, `Licence TBA`).
    pub licence: Option<&'static str>,
    /// Hidden from the catalogue unless the signed-in account has iCreate Pro. Existing installs still open.
    pub early_access: bool,
}

impl CraftApp {
    /// Card and details heading: `iCreate Photo`, stripped of the "powered by" credit.
    pub fn title(&self) -> &'static str {
        match self.display_name.split_once(" (powered by") {
            Some((title, _)) if !title.is_empty() => title,
            _ => self.display_name,
        }
    }

    /// Whether name, title, tagline, replacement product or category contain `needle`.
    pub fn matches_query(&self, needle: &str) -> bool {
        let n = needle.trim().to_lowercase();
        if n.is_empty() {
            return true;
        }
        [self.name, self.display_name, self.title(), self.tagline, self.replaces, self.category.label()]
            .into_iter()
            .any(|s| s.to_lowercase().contains(&n))
    }

    /// `ai.storyteller.<id>`: macOS bundle id, Flatpak id and Linux desktop-file id.
    pub fn bundle_id(&self) -> String {
        self.bundle.map_or_else(|| format!("ai.storyteller.{}", self.id), str::to_string)
    }

    pub fn releases_url(&self) -> String {
        format!("https://github.com/{}/releases", self.repo)
    }

    pub fn repo_url(&self) -> String {
        format!("https://github.com/{}", self.repo)
    }

    /// The app's website: its page on getartcraft.com, or a community app's own site.
    pub fn website(&self) -> String {
        self.site.map_or_else(|| format!("https://getartcraft.com/apps/{}", self.id), str::to_string)
    }

    /// Whether this app may appear with Install / Download at all (false for Planned/Research).
    pub fn is_installable(&self) -> bool {
        !self.maturity.is_catalogue_only()
    }

    /// Whether the launcher installs this app from a `kind` package.
    pub fn installs(&self, kind: crate::PackageKind) -> bool {
        self.is_installable() && (kind != crate::PackageKind::Dmg || self.mac_team.is_some())
    }

    /// Bundle ids this app may be installed under, current first.
    pub fn bundle_ids(&self) -> impl Iterator<Item = String> + '_ {
        std::iter::once(self.bundle_id()).chain(self.former_bundles.iter().map(|b| b.to_string()))
    }

    /// Asset stems this app's releases may use, current first.
    pub fn stems(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.id).chain(self.former_ids.iter().copied())
    }

    /// Display names this app's installers may have registered, current first.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        std::iter::once(self.name).chain(self.former_names.iter().copied())
    }

    pub fn by_id(id: &str) -> Option<&'static CraftApp> {
        suite().iter().find(|a| a.id == id)
    }
}

const fn app(
    id: &'static str,
    name: &'static str,
    tagline: &'static str,
    [color, ink_light, ink_dark]: [[u8; 3]; 3],
    repo: &'static str,
) -> CraftApp {
    CraftApp {
        id,
        name,
        display_name: name,
        replaces: "",
        tagline,
        icon: "",
        color,
        ink_light,
        ink_dark,
        repo,
        former_ids: &[],
        former_names: &[],
        kind: AppKind::Official,
        publisher: "ArtCraft",
        msi_publisher: None,
        bundle: None,
        former_bundles: &[],
        site: None,
        mac_team: Some(ARTCRAFT_TEAM_ID),
        category: Category::Design,
        maturity: Maturity::Stable,
        licence: None,
        early_access: false,
    }
}

const CATALOG_JSON: &str = include_str!("../../../apps.json");

/// The catalogue, in `apps.json` order.
pub fn suite() -> &'static [CraftApp] {
    static SUITE: OnceLock<Vec<CraftApp>> = OnceLock::new();
    SUITE.get_or_init(load_suite).as_slice()
}

#[allow(clippy::expect_used)]
fn load_suite() -> Vec<CraftApp> {
    parse_catalog(CATALOG_JSON).expect("apps.json must be valid catalogue JSON")
}

fn intern(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn intern_list(items: Vec<String>) -> &'static [&'static str] {
    if items.is_empty() {
        return &[];
    }
    Box::leak(items.into_iter().map(intern).collect::<Vec<_>>().into_boxed_slice())
}

#[derive(Deserialize)]
struct CatalogFile {
    apps: Vec<CatalogRow>,
}

#[derive(Deserialize)]
struct CatalogRow {
    id: String,
    name: String,
    display_name: String,
    replaces: String,
    description: String,
    icon: String,
    repo: String,
    status: String,
    category: String,
    #[serde(default)]
    former_ids: Vec<String>,
    #[serde(default)]
    former_names: Vec<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    publisher: Option<String>,
    #[serde(default)]
    msi_publisher: Option<String>,
    #[serde(default)]
    bundle: Option<String>,
    #[serde(default)]
    former_bundles: Vec<String>,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    mac_team: Option<String>,
    color: [u8; 3],
    ink_light: [u8; 3],
    ink_dark: [u8; 3],
    #[serde(default)]
    early_access: bool,
    #[serde(default)]
    licence: Option<String>,
}

fn parse_category(value: &str) -> Result<Category, String> {
    match value {
        "Photo" => Ok(Category::Photo),
        "Design" => Ok(Category::Design),
        "Video" => Ok(Category::Video),
        "Audio" => Ok(Category::Audio),
        "Documents" => Ok(Category::Documents),
        "Office" => Ok(Category::Office),
        "Work" => Ok(Category::Work),
        other => Err(format!("unknown category {other:?}")),
    }
}

fn parse_maturity(value: &str) -> Result<Maturity, String> {
    match value {
        "stable" => Ok(Maturity::Stable),
        "beta" => Ok(Maturity::Beta),
        "alpha" => Ok(Maturity::Alpha),
        "planned" => Ok(Maturity::Planned),
        "research" => Ok(Maturity::Research),
        other => Err(format!("unknown status {other:?}")),
    }
}

fn parse_kind(value: Option<&str>) -> Result<AppKind, String> {
    match value {
        None | Some("official") => Ok(AppKind::Official),
        Some("community") => Ok(AppKind::Community),
        Some(other) => Err(format!("unknown kind {other:?}")),
    }
}

fn parse_catalog(json: &str) -> Result<Vec<CraftApp>, String> {
    let file: CatalogFile = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let mut apps = Vec::with_capacity(file.apps.len());
    for row in file.apps {
        apps.push(row_to_app(row)?);
    }
    Ok(apps)
}

fn row_to_app(row: CatalogRow) -> Result<CraftApp, String> {
    if row.id.is_empty() || row.name.is_empty() || row.repo.is_empty() {
        return Err("each app needs id, name and repo".into());
    }
    if !row.repo.contains('/') {
        return Err(format!("{} repo must be owner/name", row.id));
    }
    let kind = parse_kind(row.kind.as_deref())?;
    let category = parse_category(&row.category)?;
    let maturity = parse_maturity(&row.status)?;
    let publisher = intern(row.publisher.unwrap_or_else(|| match kind {
        AppKind::Official => "ArtCraft".into(),
        AppKind::Community => "Community".into(),
    }));
    let mac_team = match row.mac_team {
        Some(team) if !team.is_empty() => Some(intern(team)),
        Some(_) => None,
        None if kind == AppKind::Official => Some(ARTCRAFT_TEAM_ID),
        None => None,
    };
    Ok(CraftApp {
        id: intern(row.id),
        name: intern(row.name),
        display_name: intern(row.display_name),
        replaces: intern(row.replaces),
        tagline: intern(row.description),
        icon: intern(row.icon),
        color: row.color,
        ink_light: row.ink_light,
        ink_dark: row.ink_dark,
        repo: intern(row.repo),
        former_ids: intern_list(row.former_ids),
        former_names: intern_list(row.former_names),
        kind,
        publisher,
        msi_publisher: row.msi_publisher.filter(|s| !s.is_empty()).map(intern),
        bundle: row.bundle.filter(|s| !s.is_empty()).map(intern),
        former_bundles: intern_list(row.former_bundles),
        site: row.site.filter(|s| !s.is_empty()).map(intern),
        mac_team,
        category,
        maturity,
        licence: row.licence.filter(|s| !s.is_empty()).map(intern),
        early_access: row.early_access,
    })
}

/// The Apple Developer team that signs every official Craft App and ArtCraft Launcher (Learning
/// Machines LLC): the macOS side of the Windows publisher check.
pub const ARTCRAFT_TEAM_ID: &str = "DJ6XS33FX8";

/// iCreate / iCreativate website.
pub const WEBSITE_URL: &str = "https://icreativate.co.za";
/// The GitHub account every official Craft App's repository lives under. The release feed can
/// only move an app within it; a community app installs from its own repo, pinned in [`suite`].
pub const GITHUB_OWNER: &str = "storytold";
/// iCreate on GitHub (the launcher's own organisation). Craft Apps still install from [`GITHUB_OWNER`].
pub const GITHUB_URL: &str = "https://github.com/iCreativate";
/// Extra social icons in the footer. Empty: iCreate points people to the website and GitHub buttons.
pub const SOCIAL: [(&str, &str); 0] = [];

/// Community chat; iCreate has no Discord yet, so this opens the website.
pub const DISCORD_URL: &str = "https://icreativate.co.za";
/// Deep link unused until a Discord invite exists; same as [`DISCORD_URL`].
pub const DISCORD_APP_URL: &str = "https://icreativate.co.za";

/// iCreate itself, described like a Craft App so it can update itself with the same machinery
/// (asset names `artcraft-launcher-<ver>-…`, bundle id `za.co.icreativate.icreate`, `iCreate.app`).
/// Not part of [`suite`]. The asset stem stays `artcraft-launcher` so packaging and self-update
/// keep matching GitHub release files.
pub const LAUNCHER: CraftApp = CraftApp {
    publisher: "iCreativate",
    bundle: Some("za.co.icreativate.icreate"),
    site: Some("https://icreativate.co.za"),
    mac_team: None,
    display_name: "iCreate",
    replaces: "",
    icon: "icreate.png",
    category: Category::Design,
    maturity: Maturity::Stable,
    ..app(
        "artcraft-launcher",
        "iCreate",
        "Installs and updates the iCreate creative suite",
        [[0x0e, 0xa5, 0xe9], [0x03, 0x69, 0xa1], [0x38, 0xbd, 0xf8]],
        "iCreativate/icreate-launcher",
    )
};

/// The launcher's own repo, checked so it can say when a newer launcher is out.
pub const LAUNCHER_REPO: &str = "iCreativate/icreate-launcher";
/// The launcher's own asset stem (release file prefix).
pub const LAUNCHER_ID: &str = "artcraft-launcher";
/// Shown in the window, About, tray and installers.
pub const APP_DISPLAY_NAME: &str = "iCreate";
/// macOS bundle id, Linux app id and Windows AppUserModelID.
pub const APP_BUNDLE_ID: &str = "za.co.icreativate.icreate";

/// A Craft App's 128 px icon (PNG), as copied from its repo into `assets/app-icons`: the UI
/// draws it, and the Linux install puts it in the desktop menu.
pub fn app_icon_png(id: &str) -> Option<&'static [u8]> {
    Some(match id {
        "photocraft" => include_bytes!("../../../assets/app-icons/photocraft.png"),
        "vectorcraft" => include_bytes!("../../../assets/app-icons/vectorcraft.png"),
        "filmcraft" => include_bytes!("../../../assets/app-icons/filmcraft.png"),
        "lightcraft" => include_bytes!("../../../assets/app-icons/lightcraft.png"),
        "pdfcraft" => include_bytes!("../../../assets/app-icons/pdfcraft.png"),
        "effectcraft" => include_bytes!("../../../assets/app-icons/effectcraft.png"),
        "designcraft" => include_bytes!("../../../assets/app-icons/designcraft.png"),
        "wordcraft" => include_bytes!("../../../assets/app-icons/wordcraft.png"),
        "deckcraft" => include_bytes!("../../../assets/app-icons/deckcraft.png"),
        "gridcraft" => include_bytes!("../../../assets/app-icons/gridcraft.png"),
        "texturecraft" => include_bytes!("../../../assets/app-icons/texturecraft.png"),
        "materialcraft" => include_bytes!("../../../assets/app-icons/materialcraft.png"),
        "craft-anything" => include_bytes!("../../../assets/app-icons/craft-anything.png"),
        "solidcraft" => include_bytes!("../../../assets/app-icons/solidcraft.png"),
        "cadcraft" => include_bytes!("../../../assets/app-icons/cadcraft.png"),
        "soundcraft" => include_bytes!("../../../assets/app-icons/soundcraft.png"),
        "figcraft" => include_bytes!("../../../assets/app-icons/figcraft.png"),
        "email" => include_bytes!("../../../assets/app-icons/email.png"),
        "calendar" => include_bytes!("../../../assets/app-icons/calendar.png"),
        "notes" => include_bytes!("../../../assets/app-icons/notes.png"),
        "drive" => include_bytes!("../../../assets/app-icons/drive.png"),
        "chat" => include_bytes!("../../../assets/app-icons/chat.png"),
        "meet" => include_bytes!("../../../assets/app-icons/meet.png"),
        "sign" => include_bytes!("../../../assets/app-icons/sign.png"),
        "forms" => include_bytes!("../../../assets/app-icons/forms.png"),
        "projects" => include_bytes!("../../../assets/app-icons/projects.png"),
        "crm" => include_bytes!("../../../assets/app-icons/crm.png"),
        "invoice" => include_bytes!("../../../assets/app-icons/invoice.png"),
        "passwords" => include_bytes!("../../../assets/app-icons/passwords.png"),
        "record" => include_bytes!("../../../assets/app-icons/record.png"),
        "3d" => include_bytes!("../../../assets/app-icons/3d.png"),
        "code" => include_bytes!("../../../assets/app-icons/code.png"),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_app_has_its_icon() {
        for app in suite() {
            let png = app_icon_png(app.id).unwrap_or_else(|| panic!("no icon for {}", app.id));
            assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "{} icon isn't a PNG", app.id);
            assert_eq!(app.icon, format!("{}.png", app.id), "{} icon field should name its PNG", app.id);
        }
        assert!(app_icon_png("notacraft").is_none());
    }

    /// Every catalog entry must have a distinct lookup key.
    #[test]
    fn suite_has_unique_ids_and_icreate_names() {
        let apps = suite();
        assert_eq!(apps.len(), 32);
        let mut ids: Vec<_> = apps.iter().map(|a| a.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), apps.len());
        for a in apps {
            assert_eq!(a.name.to_lowercase().replace(' ', "-"), a.id);
            assert!(a.display_name.starts_with("iCreate "), "{} display_name", a.id);
            assert!(!a.repo.is_empty() && a.repo.contains('/'), "{} repo", a.id);
            if a.category == Category::Work {
                assert!(a.maturity.is_catalogue_only(), "{} Work app must be Planned/Research", a.id);
                assert!(!a.is_installable(), "{} must not be installable yet", a.id);
                assert!(!a.installs(crate::PackageKind::Msi));
                assert!(a.licence.is_some(), "{} needs a licence label", a.id);
                assert!(!a.display_name.contains("powered by"), "{} is first-party Work, not a Craft fork credit", a.id);
            } else {
                assert!(a.display_name.contains(&format!("powered by {}", a.name)), "{} credits {}", a.id, a.name);
                assert!(!a.replaces.is_empty(), "{} replaces", a.id);
                assert!(a.is_installable());
            }
            match a.kind {
                AppKind::Official => {}
                AppKind::Community => assert!(!a.repo.starts_with("storytold/"), "{} is community", a.id),
            }
        }
        let photo = CraftApp::by_id("photocraft").unwrap();
        assert_eq!(photo.title(), "iCreate Photo");
        assert_eq!(photo.replaces, "Adobe Photoshop");
        assert_eq!(photo.category, Category::Photo);
        assert_eq!(photo.maturity, Maturity::Beta);
        assert_eq!(photo.repo, "iCreativate/icreate-photo");
        assert!(!apps.iter().any(|a| a.early_access), "shipping apps stay on Free; early_access is a later extras flag");
        assert!(photo.matches_query("photoshop"));
        assert!(photo.matches_query("iCreate Photo"));
        assert!(!photo.matches_query("premiere"));
        let motion = CraftApp::by_id("effectcraft").unwrap();
        assert_eq!(motion.maturity, Maturity::Alpha);
        assert!(motion.maturity.is_early());
        assert_eq!(motion.category, Category::Video);
        let email = CraftApp::by_id("email").unwrap();
        assert_eq!(email.category, Category::Work);
        assert_eq!(email.maturity, Maturity::Planned);
        assert_eq!(email.title(), "iCreate Email");
        assert!(CraftApp::by_id("calendar").unwrap().maturity == Maturity::Research);
    }

    #[test]
    fn a_fork_repo_is_taken_from_json_with_no_rust_change() {
        let json = r#"{
            "apps": [{
                "id": "photocraft",
                "name": "PhotoCraft",
                "display_name": "iCreate Photo (powered by PhotoCraft)",
                "replaces": "Adobe Photoshop",
                "description": "Image editing",
                "icon": "photocraft.png",
                "repo": "iCreativate/photocraft",
                "status": "stable",
                "category": "Photo",
                "color": [1, 2, 3],
                "ink_light": [4, 5, 6],
                "ink_dark": [7, 8, 9]
            }]
        }"#;
        let apps = parse_catalog(json).unwrap();
        assert_eq!(apps[0].repo, "iCreativate/photocraft");
        assert_eq!(apps[0].id, "photocraft");
        assert_eq!(apps[0].name, "PhotoCraft");
        assert_eq!(apps[0].maturity, Maturity::Stable);
        assert_eq!(apps[0].stems().collect::<Vec<_>>(), ["photocraft"]);
    }

    #[test]
    fn work_planned_and_research_are_catalogue_only() {
        let json = r#"{
            "apps": [{
                "id": "email",
                "name": "Email",
                "display_name": "iCreate Email",
                "replaces": "Microsoft Outlook",
                "description": "Mail",
                "icon": "email.png",
                "repo": "iCreativate/icreate-email",
                "status": "planned",
                "category": "Work",
                "licence": "MPL-2.0",
                "mac_team": "",
                "color": [1, 2, 3],
                "ink_light": [4, 5, 6],
                "ink_dark": [7, 8, 9]
            }, {
                "id": "notes",
                "name": "Notes",
                "display_name": "iCreate Notes",
                "replaces": "Apple Notes",
                "description": "Notes",
                "icon": "notes.png",
                "repo": "iCreativate/icreate-notes",
                "status": "research",
                "category": "Work",
                "licence": "Licence TBA",
                "mac_team": "",
                "color": [1, 2, 3],
                "ink_light": [4, 5, 6],
                "ink_dark": [7, 8, 9]
            }]
        }"#;
        let apps = parse_catalog(json).unwrap();
        assert_eq!(apps[0].category, Category::Work);
        assert_eq!(apps[0].maturity, Maturity::Planned);
        assert_eq!(apps[0].licence, Some("MPL-2.0"));
        assert!(!apps[0].is_installable());
        assert!(!apps[0].installs(crate::PackageKind::Msi));
        assert_eq!(apps[1].maturity, Maturity::Research);
        assert!(!apps[1].is_installable());
    }

    #[test]
    fn solidcraft_is_a_community_app_with_its_own_bundle_id() {
        let sc = CraftApp::by_id("solidcraft").unwrap();
        assert_eq!(sc.kind, AppKind::Community);
        assert_eq!(sc.repo, "tangency-labs/solidcraft");
        assert_eq!(sc.bundle_id(), "io.tangencylabs.solidcraft");
        assert_eq!(sc.bundle_ids().collect::<Vec<_>>(), ["io.tangencylabs.solidcraft", "ai.storyteller.solidcraft", "ai.storyteller.solvecraft"]);
        assert_eq!(CraftApp::by_id("gridcraft").unwrap().bundle_id(), "ai.storyteller.gridcraft");
        assert_eq!(CraftApp::by_id("gridcraft").unwrap().kind, AppKind::Official);
        assert_eq!(sc.website(), "https://solidcraft.io");
        assert!(sc.installs(crate::PackageKind::Dmg) && sc.installs(crate::PackageKind::Msi) && sc.installs(crate::PackageKind::Flatpak));
        assert_eq!(sc.mac_team, Some("G5QA5U5X2S"));
        assert_eq!(CraftApp::by_id("gridcraft").unwrap().mac_team, Some(ARTCRAFT_TEAM_ID));
    }

    #[test]
    fn texturecraft_is_an_official_app() {
        let app = CraftApp::by_id("texturecraft").unwrap();
        assert_eq!(app.kind, AppKind::Official);
        assert_eq!(app.repo, "storytold/texturecraft");
        assert_eq!(app.bundle_id(), "ai.storyteller.texturecraft");
        assert_eq!(app.website(), "https://getartcraft.com/apps/texturecraft");
        assert_eq!(app.mac_team, Some(ARTCRAFT_TEAM_ID));
    }

    #[test]
    fn materialcraft_is_an_official_app() {
        let app = CraftApp::by_id("materialcraft").unwrap();
        assert_eq!(app.kind, AppKind::Official);
        assert_eq!(app.repo, "storytold/materialcraft");
        assert_eq!(app.bundle_id(), "ai.storyteller.materialcraft");
        assert_eq!(app.website(), "https://github.com/storytold/materialcraft");
        assert_eq!(app.mac_team, Some(ARTCRAFT_TEAM_ID));
    }

    #[test]
    fn craft_anything_is_an_official_app() {
        let ca = CraftApp::by_id("craft-anything").unwrap();
        assert_eq!(ca.kind, AppKind::Official);
        assert_eq!(ca.repo, "storytold/craft-anything");
        assert_eq!(ca.bundle_id(), "ai.storyteller.craft-anything");
        assert_eq!(ca.website(), "https://getartcraft.com/apps/craft-anything");
        assert_eq!(ca.mac_team, Some(ARTCRAFT_TEAM_ID));
    }

    #[test]
    fn pdfcraft_also_answers_to_printcraft() {
        let pdf = CraftApp::by_id("pdfcraft").unwrap();
        assert_eq!(pdf.stems().collect::<Vec<_>>(), ["pdfcraft", "printcraft"]);
        assert_eq!(pdf.names().collect::<Vec<_>>(), ["PdfCraft", "PrintCraft"]);
        assert_eq!(pdf.website(), "https://getartcraft.com/apps/pdfcraft");
        assert_eq!(CraftApp::by_id("wordcraft").unwrap().website(), "https://getartcraft.com/apps/wordcraft");
    }

    #[test]
    fn icreate_updates_from_its_own_github_repo() {
        assert_eq!(LAUNCHER.name, "iCreate");
        assert_eq!(LAUNCHER.publisher, "iCreativate");
        assert_eq!(LAUNCHER.repo, "iCreativate/icreate-launcher");
        assert_eq!(LAUNCHER.bundle_id(), "za.co.icreativate.icreate");
        assert_eq!(LAUNCHER_REPO, "iCreativate/icreate-launcher");
        assert_eq!(LAUNCHER_ID, "artcraft-launcher");
        assert_eq!(APP_DISPLAY_NAME, "iCreate");
        assert_eq!(APP_BUNDLE_ID, "za.co.icreativate.icreate");
        assert_eq!(WEBSITE_URL, "https://icreativate.co.za");
        assert_eq!(GITHUB_URL, "https://github.com/iCreativate");
    }
}
