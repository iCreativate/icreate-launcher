# Attribution

Every non-code asset in this repository, with its author, source and license. iCreate is based
on ArtCraft Launcher. Code and original assets are MIT OR Apache-2.0 (see [`LICENSE-MIT`](LICENSE-MIT),
[`LICENSE-APACHE`](LICENSE-APACHE) and [`NOTICE`](NOTICE)). When you add an asset, add a row here
in the same change; third-party assets must be permissively licensed and keep their license file
next to them.

## Bundled in the app

| Path | Title | Author | Source | License |
|---|---|---|---|---|
| `assets/app-icon/` (all files) | iCreate Slate launcher icon (geometric i on slate tile) | iCreativate | Original Slate family; see [`assets/app-icon/README.md`](assets/app-icon/README.md) | iCreativate trademark, [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt) |
| `packaging/macos/dmg/` (all files) | macOS DMG window background and layout | iCreativate | Slate palette + geometric-i monogram; see [`packaging/macos/dmg/README.md`](packaging/macos/dmg/README.md) | MIT OR Apache-2.0; monogram trademark [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt) |
| `assets/social/linkedin.svg` | LinkedIn logo, linking to ArtCraft's company page | Fonticons, Inc. ([Font Awesome Free](https://fontawesome.com) 6.7.2) | Font Awesome Free `svgs/brands/linkedin.svg`; white fill and a square view box, see [`assets/social/README.md`](assets/social/README.md) | CC BY 4.0. The logo is a trademark of LinkedIn Corporation. |
| `assets/social/x.svg`, `youtube.svg`, `tiktok.svg`, `instagram.svg`, `discord.svg` | X, YouTube, TikTok, Instagram and Discord logos, linking to ArtCraft's accounts | [Simple Icons](https://simpleicons.org/) contributors | Simple Icons, `fill` set to white for tinting; see [`assets/social/README.md`](assets/social/README.md) | CC0 1.0. The logos are trademarks of X Corp., Google LLC, TikTok / ByteDance, Meta Platforms, Inc. and Discord Inc. |
| `assets/social/github.svg`, `assets/social/globe.svg` | GitHub logo (linking to ArtCraft's GitHub) and a globe (linking to ArtCraft's website) | [Simple Icons](https://simpleicons.org/) contributors; [Lucide](https://lucide.dev) contributors | Simple Icons `github.svg` and Lucide `globe.svg` (lucide-static 0.468.0), white for tinting; see [`assets/social/README.md`](assets/social/README.md) | CC0 1.0 (GitHub logo; a trademark of GitHub, Inc.); ISC (globe) |
| `docs/screenshots/artcraft-launcher-grid.png`, `artcraft-launcher-table.png` | Screenshots of ArtCraft Launcher (macOS: grid view in dark theme, table view in light theme), used in the README | ArtCraft Launcher contributors | Captured from the app | MIT OR Apache-2.0 for the screenshot itself; the ArtCraft logo in it stays an ArtCraft trademark ([`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt)) and the app icons in it keep their own licenses (above) |
| (not bundled) the pictures on each app's details page | Screenshots of the Craft Apps, fetched when a page is opened and kept in the launcher's cache folder | The apps' contributors | [storytold/craft-launcher-assets](https://github.com/storytold/craft-launcher-assets), pinned to a tag (`SCREENSHOTS_REV` in `crates/core/src/details.rs`); each picture's source and license are listed in that repository's `ATTRIBUTION.md` | Each app's own license for the interface (MIT OR Apache-2.0); the work shown in them is public domain or made in the app, as that file says |
| `docs/brand/icreate-mark.svg`, `icreate-mark-black.svg` | iCreate geometric-i monogram (header / tray) | iCreativate | Original Slate family | iCreativate trademark, [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt) |
| `docs/brand/icreate-mark-64.png`, `icreate-mark-black-44.png` | iCreate monogram, tray / menu-bar sizes | iCreativate | Rendered from the SVG masters | iCreativate trademark, [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt) |
| `assets/fonts/DMSans-Variable.ttf` | DM Sans, variable (body text) | The DM Sans Project Authors (Colophon Foundry, Indian Type Foundry, Google) | Google Fonts | SIL OFL 1.1, [`assets/fonts/OFL-DMSans.txt`](assets/fonts/OFL-DMSans.txt) |
| `assets/fonts/Syne-Variable.ttf` | Syne, variable (headings and app names) | The Syne Project Authors (Bonjour Monde) | Google Fonts | SIL OFL 1.1, [`assets/fonts/OFL-Syne.txt`](assets/fonts/OFL-Syne.txt) |
| `assets/fonts/SourceCodePro-Variable.ttf` | Source Code Pro, variable (labels and buttons) | Adobe | Google Fonts | SIL OFL 1.1, [`assets/fonts/OFL-SourceCodePro.txt`](assets/fonts/OFL-SourceCodePro.txt) |
| (dependency) `epaint_default_fonts/fonts/NotoEmoji-Regular.ttf` | Noto Emoji Regular, monochrome emoji fallback for arbitrary release-note, feed and alert text | The Noto Project Authors (Google) | Bundled transitively by `egui 0.37`'s `monochrome_emoji_fonts` feature from `epaint_default_fonts`; <https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts> | SIL OFL 1.1, `epaint_default_fonts/fonts/OFL.txt`; see also the dependency `README` and egui `README.md` |
| (dependency) `epaint_default_fonts/fonts/emoji-icon-font.ttf` | emoji-icon-font, monochrome symbol/emoji fallback | John Slegers | Bundled transitively by `egui 0.37`'s `monochrome_emoji_fonts` feature from `epaint_default_fonts`; <https://github.com/jslegers/emoji-icon-font> | MIT, Copyright (c) 2014 John Slegers, `epaint_default_fonts/fonts/emoji-icon-font-mit-license.txt` |
| (dependency) `epaint_default_fonts/fonts/egui-icons.ttf` | egui-icons, five-glyph subset of emoji-icon-font | John Slegers | [egui 6d518fc7](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts); included by default_fonts independently of monochrome_emoji_fonts | MIT, Copyright (c) 2014 John Slegers; full upstream text in `NOTICE` |
| (dependency) `epaint_default_fonts/fonts/Ubuntu-Light.ttf` | Ubuntu Light, default font fallback | Canonical Ltd. | [Pinned egui source](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts) | Ubuntu Font Licence 1.0; full upstream text and copyright in `NOTICE` |
| (dependency) `epaint_default_fonts/fonts/Hack-Regular.ttf` | Hack Regular, default monospace fallback | Source Foundry Authors; Bitstream Inc.; DejaVu contributors | [Pinned egui source](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts) | MIT and Bitstream Vera License; DejaVu work public domain; full upstream notices in `NOTICE` |

### Catalogue app icons (iCreate Slate family)

Original iCreativate icons for each suite entry. Shared slate tile, ivory glyph, unique accent
and underline bar. SVG masters: `branding/icon-system-2026/masters/`. Not ArtCraft artwork.

| Path | Title | Author | Source | License |
|---|---|---|---|---|
| `assets/app-icons/*.png` | iCreate Slate catalogue icons (128 px): Photo, Vector, Video, Light, PDF, Motion, Layout, UI, Write, Audio, Slides, CAD, Sheets, Texture, Material, Labs, Solid | iCreativate | Original Slate family | iCreativate trademark, [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt); redistributable with this repo under MIT OR Apache-2.0 subject to those terms |

## Documentation and README

| Path | Title | Author | Source | License |
|---|---|---|---|---|
| `docs/brand/` (iCreate mark files) | iCreate name and geometric-i monogram | iCreativate | Original | iCreativate trademark, [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt) |
