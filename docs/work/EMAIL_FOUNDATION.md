# iCreate Email — foundation decision

**Status:** Approved (plan acceptance, 2026-11-05)  
**Foundation:** Mozilla Thunderbird (latest release / ESR track)  
**Product licence for modified MPL files:** MPL 2.0  
**Catalogue maturity until first installer:** Planned (no Install / Download)

## Why Thunderbird

Cross-platform desktop mail with IMAP, SMTP, Google and Microsoft OAuth, multi-identity, signatures, search, attachments, junk controls, OpenPGP/S/MIME and offline store. Exchange/EWS exists today; Microsoft Graph is the path for Microsoft 365 before EWS Online sunset (~Oct 2026). Evolution is Linux-first. Greenfield would delay Install by 12–24+ months.

## Trademark obligations

- No Thunderbird bird, Mozilla wordmarks, or endorsement claims.
- Replace product name, icons, splash, About, user-agent branding and store listings with original iCreate Email / Slate identity.
- Keep MPL 2.0 notices and source offer for modified MPL-covered files (`NOTICE`, `LICENSE`, in-app licence UI).

## Security defaults (non-negotiable)

- Credentials and refresh tokens stay in the OS keychain or the client’s encrypted password store.
- Default: zero telemetry to iCreate. Disable or gate Mozilla telemetry in the fork.
- No message bodies, headers or credentials leave the device for an iCreate server unless the user later enables an explicit opt-in product with separate consent.
- Autoupdate from iCreativate GitHub Releases (or signed CDN), not Mozilla’s channel, once branded builds exist.

## Catalogue gate

Flip `email` in `apps.json` from `planned` → `alpha` **only** when a working Windows/macOS/Linux installer and update source exist. Until then the launcher shows **Planned**, never Install or Download.
