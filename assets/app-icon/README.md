# iCreate app icon

The app icon is the **iCreate leaping figure in cyan**, from
[`docs/brand/icreate-mark.svg`](../../docs/brand/icreate-mark.svg). Every file in this
directory is rendered from it by `packaging/icons.sh`.

## Files

- `icreate.svg`: a copy of `docs/brand/icreate-mark.svg`; also the hicolor scalable icon.
- `icreate-1024.png`: 1024 px render, with extra transparent margin for Apple's icon grid.
- `icreate.icns`: macOS bundle icon (`CFBundleIconFile`).
- `icreate.ico`: Windows icon, 16–256 px, embedded in the `.exe` by
  `apps/artcraft-launcher/build.rs` and used by the MSI.
- `hicolor/<size>/apps/za.co.icreativate.icreate.png` (16–512) and
  `hicolor/scalable/apps/za.co.icreativate.icreate.svg`: Linux icon theme.
- The tray / menu-bar icon is the same mark, rendered into `docs/brand/icreate-mark-black-44.png`
  (macOS template, 22 pt @2x) and `docs/brand/icreate-mark-64.png` (cyan; Windows, Linux, FreeBSD).

## Regenerate

Run `packaging/icons.sh` on any OS. It needs `resvg`; the `.ico` and `.icns` are packed by
`cargo xtask ico` / `cargo xtask icns`.
