# Releasing iCreate

Tag a version. GitHub Actions builds the installers, writes `SHA256SUMS.txt`, and publishes a GitHub Release. The running launcher looks up `iCreativate/icreate-launcher` `/releases/latest`, then installs the matching `artcraft-launcher-<ver>-…` file after checking its SHA-256.

```sh
# Version lives only in [workspace.package] version (Cargo.toml).
cargo xtask version set 1.0.0
git add Cargo.toml Cargo.lock crates apps xtask
git commit -m "Release 1.0.0"
git tag v1.0.0
git push origin main --tags
```

The tag must match Cargo.toml (`v1.0.0` ↔ `1.0.0`). A mismatch fails the workflow.

Until Apple and Windows certificates are in GitHub secrets, the same workflow still publishes **unsigned** builds. The release notes tell people how to open them.

## What a tag builds

| Platform | File | Notes |
|---|---|---|
| macOS 11+ | `artcraft-launcher-<ver>-macos-universal.dmg` | Intel **and** Apple silicon in one image (`lipo`). Self-update prefers `macos-universal`. |
| Windows 10/11 | `artcraft-launcher-<ver>-windows-x64.msi` | Per-user, no admin. `package.ps1` also writes a portable zip. |
| Linux x86_64 and aarch64 | `artcraft-launcher-<ver>-linux-<arch>.AppImage` and `.deb` | Built on Ubuntu 22.04 (glibc 2.35). |

Asset **names** stay `artcraft-launcher-…` on purpose: install, checksums and self-update key off that stem, not the display name iCreate.

A version with a hyphen (`1.0.0-rc.1`) is marked prerelease. GitHub's `/releases/latest` **skips** prereleases, so self-update on stable installs will not pick it up.

## Certificates

You do **not** need these for the first unsigned release. Add them when you want Gatekeeper and SmartScreen to stay quiet.

### Apple (macOS)

| Item | What it is | Cost (2026) |
|---|---|---|
| **Apple Developer Program** | Annual membership. Required for Developer ID and notarisation. Enrol as iCreativate (organisation) or as yourself. [developer.apple.com/programs](https://developer.apple.com/programs/) | **US$99 / year** (local currency where Apple offers it; South Africa is typically charged in USD) |
| **Developer ID Application** | Certificate that signs `iCreate.app` and the `.dmg` for distribution **outside** the Mac App Store. Created by the Account Holder under Certificates, Identifiers & Profiles. **Not** “Apple Development” and **not** “Mac App Distribution”. | Included in the membership |
| **Developer ID Installer** | Only if you ship a `.pkg`. We ship a `.dmg`, so you can skip this. | Included |
| **Notarisation** | `notarytool` uploads the signed app; Apple scans it and issues a ticket that `stapler` attaches. Needs an **app-specific password** for the Apple ID, not the account login password. | Included |

You need all of: the `.p12` of the Developer ID Application identity, its export password, the Apple ID email, the app-specific password, and the 10-character Team ID.

Create the certificate (Account Holder only):

1. [developer.apple.com/account/resources/certificates](https://developer.apple.com/account/resources/certificates/list) → **+** → **Developer ID Application**.
2. On a Mac, Keychain Access → Certificate Assistant → Request a Certificate From a Certificate Authority → save a `.certSigningRequest`.
3. Upload the CSR, download the `.cer`, double-click to install.
4. Keychain Access → My Certificates → the “Developer ID Application: …” identity → Export → `.p12`, pick a strong password.

App-specific password: appleid.apple.com → Sign-In and Security → App-Specific Passwords → generate one named `iCreate notarytool`.

Team ID: developer.apple.com/account → Membership details.

### Windows

Windows no longer issues a software-only `.pfx` you can copy around. Since 2023 the private key must live on a FIPS token or a cloud HSM. From February 2026, public code-signing certificates last at most **459 days**.

| Option | What it is | Typical cost (2026) | Fits this workflow |
|---|---|---|---|
| **OV Authenticode** (Sectigo, SSL.com, GlobalSign, DigiCert, …) | Organisation-validated code signing. Removes “Unknown publisher”. SmartScreen reputation still builds up over downloads. Key on a USB token or cloud HSM. | About **US$200–700 / year** (resellers toward the low end; DigiCert list toward the high). Hardware token often extra (~US$100–400) unless the CA ships one. | `WINDOWS_CERTIFICATE` only works if your CA lets you export a `.pfx` **or** you use their cloud signer. A USB token **cannot** be plugged into GitHub-hosted runners. |
| **EV Authenticode** | Extended validation. Immediate SmartScreen reputation. Same hardware rules. | About **US$300–1,200 / year** | Same export/cloud constraint. |
| **Azure Trusted Signing** (Microsoft) | Cloud signing, no USB token. Needs an Azure account and identity check. Public-trust profiles are limited by Microsoft to organisations in the US, Canada, the EU and the UK (and individuals in the US/Canada). A South African sole trader may **not** qualify; an organisation with a qualifying Azure tenant might. [Azure Trusted Signing](https://azure.microsoft.com/products/trusted-signing) | **Basic ~US$10 / month** (5 000 signatures); **Premium ~US$100 / month** (100 000). Extra signatures ~US$0.005. | Use the `AZURE_*` secrets. `packaging/windows/sign.ps1` already speaks this API. |

Practical path for iCreativate in South Africa:

1. Prefer a CA that offers **cloud / HSM signing** (SSL.com eSigner, Certum SimplySign, or Azure Trusted Signing **if** Microsoft accepts the organisation).
2. If you only get a USB token, you cannot sign on GitHub-hosted runners. Either switch the Windows job to a self-hosted Windows runner with the token plugged in, or use the CA’s cloud API.
3. Timestamp every signature (the scripts already do). Signed files stay valid after the certificate expires.

Linux packages are unsigned unless you later add an OpenPGP key. AppImage and `.deb` do not need Apple or Microsoft certificates.

## GitHub secrets

Repo (or org) → **Settings → Secrets and variables → Actions → New repository secret**.

Names must match exactly. Empty secrets mean “unsigned is fine”.

### macOS

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | Base64 of the exported `.p12`: `base64 -i developer-id.p12 \| pbcopy` (macOS) or `base64 -w0 developer-id.p12` (Linux). One line, no headers. |
| `APPLE_CERTIFICATE_PASSWORD` | Password you set when exporting the `.p12`. |
| `KEYCHAIN_PASSWORD` | Any long random string. CI creates a throwaway keychain with it. `openssl rand -hex 24`. |
| `APPLE_ID` | Apple ID email that belongs to the Developer Program team. |
| `APPLE_PASSWORD` | **App-specific password**, not the Apple ID password. |
| `APPLE_TEAM_ID` | 10-character Team ID, for example `A1B2C3D4E5`. |

### Windows (pick one path)

**A. Certificate file** (only if you truly have an exportable `.pfx`, which most CAs no longer give you):

| Secret | Value |
|---|---|
| `WINDOWS_CERTIFICATE` | Base64 of the `.pfx`: `base64 -w0 codesign.pfx`. |
| `WINDOWS_CERTIFICATE_PASSWORD` | PFX password. |

**B. Azure Trusted Signing** (all six required):

| Secret | Value |
|---|---|
| `AZURE_TENANT_ID` | Directory (tenant) ID |
| `AZURE_CLIENT_ID` | App registration ID |
| `AZURE_CLIENT_SECRET` | App registration secret |
| `AZURE_SIGNING_ENDPOINT` | Regional endpoint, e.g. `https://weu.codesigning.azure.net` |
| `AZURE_SIGNING_ACCOUNT` | Trusted Signing account name |
| `AZURE_CERT_PROFILE` | Certificate profile name |

Never commit the `.p12` / `.pfx` / passwords. After pasting, confirm the workflow run’s **macOS** / **Windows** logs say it imported or signed, not “leaving unsigned”.

## Opening unsigned builds

Put these in support replies until the secrets above are set. The GitHub release body repeats them.

**macOS** — Gatekeeper: “Apple cannot check it for malicious software.”

1. Drag iCreate to Applications.
2. Finder → Applications → Control-click iCreate → **Open** → **Open**.
3. Or: `xattr -cr /Applications/iCreate.app && open /Applications/iCreate.app`

**Windows** — SmartScreen: “Windows protected your PC.”

1. **More info** → **Run anyway**, only if the `.msi` came from [our GitHub releases](https://github.com/iCreativate/icreate-launcher/releases).
2. Per-user install; no Administrator prompt.

**Linux** — no publisher signature.

```sh
chmod +x artcraft-launcher-*-linux-x86_64.AppImage
./artcraft-launcher-*-linux-x86_64.AppImage
# or
sudo apt install ./artcraft-launcher-*-linux-x86_64.deb
```

## Self-update

`crates/native/src/self_update.rs` and `LAUNCHER_REPO` in `crates/core/src/catalog.rs` already point at `iCreativate/icreate-launcher`. A published (non-prerelease) release with `SHA256SUMS.txt` and the asset names in the table above is enough. No extra endpoint.

Manual dry run without tagging: **Actions → Release → Run workflow**. Leave “Create a GitHub release” unchecked to only build artifacts.
