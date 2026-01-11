# Release Checklist

Use this checklist before publishing a production build.

## 1) Versioning

- Update the app version in all locations:
  - `package.json` (`version`).
  - `src-tauri/tauri.conf.json` (`version`).
  - `src-tauri/Cargo.toml` (`package.version`).
- Keep versions in sync across the desktop bundle and frontend metadata.

## 2) Signing & Credentials

### Windows (Signed MSI/EXE)

- Acquire a code-signing certificate and install it in the Windows certificate store.
- Set `bundle.windows.certificateThumbprint` in `src-tauri/tauri.conf.json` to the certificate thumbprint.
- Set `bundle.windows.timestampUrl` to a trusted timestamp service URL (e.g., Digicert).
- Verify signing with `signtool verify /pa <path-to-installer-or-exe>`.

### macOS (Signed & Notarized DMG/APP)

- Ensure a valid Apple Developer ID certificate is installed.
- Configure `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID` in the CI secrets.
- Confirm the `tauri build` process uses `codesign` and notarization in CI.
- Validate with `spctl --assess --verbose <path-to-app>`.

### Linux

- No code signing by default.
- Ensure package metadata (icon, description) is correct in `tauri.conf.json`.

## 3) Updater (Auto-Update)

- Confirm auto-update is required for production.
- Populate updater settings in `src-tauri/tauri.conf.json`:
  - `plugins.updater.pubkey`: public key used to verify updates.
  - `plugins.updater.endpoints`: update manifest endpoint(s).
- Publish update manifests and binaries to the endpoint defined above.
- Verify the update flow using a staging endpoint before production rollout.

## 4) Build & Package

- Run `npm run build` to build the frontend assets.
- Run `npm run tauri:build` to generate platform packages.
- Confirm artifacts are stored and labeled per OS:
  - Windows: `.msi` or `.exe`.
  - macOS: `.dmg` or `.app`.
  - Linux: `.deb`, `.rpm`, or `.AppImage`.

## 5) Post-Release Validation

- Install the packaged build on each OS and confirm:
  - App launches without warnings.
  - Version shown matches the release version.
  - Updater can check for updates (if enabled).
