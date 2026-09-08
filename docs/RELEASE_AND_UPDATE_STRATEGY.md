# Release and Update Strategy (LP-0915)

## 1. Overview
Light Postman (`postman-client`) is designed as a standalone, zero-cloud desktop application. Distribution and automatic updates are orchestrated via Tauri v2's native updater mechanism combined with cryptographic signature verification.

---

## 2. Cryptographic Release Signing

### Key Generation
All update bundles are cryptographically signed using the Ed25519 signature algorithm:
```bash
npx tauri signer generate -w ~/.tauri/postman_client.key
```
- **Private Key (`postman_client.key`)**: Stored exclusively in encrypted CI/CD secrets (`TAURI_SIGNING_PRIVATE_KEY`) or hardware security keys. It is never committed to source control.
- **Public Key**: Baked directly into `tauri.conf.json` under `bundle.updater.pubkey`.

### Verification Flow
1. The desktop client queries the update endpoint (GitHub Releases latest asset manifest `latest.json`).
2. The client downloads the platform-specific update artifact and its corresponding `.sig` signature file.
3. Before executing the installer or replacing binaries, Tauri verifies that `Ed25519_Verify(pubkey, artifact_hash, signature)` returns true.
4. If signature validation fails or hashes do not match, the update is rejected and purged immediately.

---

## 3. Platform Distribution Targets

| Platform | Target Formats | Toolchain Requirement | Output Location |
| :--- | :--- | :--- | :--- |
| **Windows** | `.exe` (NSIS installer)<br>`.msi` (WiX3 MSI) | NSIS, WiX3 (auto-fetched by Tauri) | `src-tauri/target/release/bundle/nsis`<br>`src-tauri/target/release/bundle/msi` |
| **Linux** | `.deb` (Debian/Ubuntu package)<br>`.AppImage` (Universal portable binary) | `dpkg`, `appstream`, `patchelf` | `src-tauri/target/release/bundle/deb`<br>`src-tauri/target/release/bundle/appimage` |
| **macOS** | `.dmg` (Disk Image)<br>`.app` (Application bundle) | macOS SDK, `hdiutil` | `src-tauri/target/release/bundle/dmg` |

---

## 4. Semantic Versioning & Migration Policy

1. **SemVer 2.0.0 (`MAJOR.MINOR.PATCH`)**:
   - `MAJOR`: Breaking storage schema changes requiring manual export/import migration.
   - `MINOR`: New features (e.g., new protocols, new AI models, new export targets). Backward-compatible additive migrations.
   - `PATCH`: Security fixes, bug patches, and performance optimizations.
2. **Database Migration Immutability**:
   - As documented in `src-tauri/src/db.rs`, **never edit a previously shipped migration entry**.
   - Every schema change must append a new migration with an incremented version ID.
   - Migrations are transactional and executed sequentially on startup inside `PRAGMA foreign_keys = ON;`.
3. **Rollback Safety**:
   - If an update fails, previous application files remain untouched.
   - SQLite WAL files are safely checkpointed before any binary replacement.

---

## 5. Automated CI/CD Release Pipeline

When a Git tag matching `v*.*.*` is pushed:
1. GitHub Actions triggers `release.yml`.
2. Compiles optimized production binaries on `windows-latest`, `ubuntu-latest`, and `macos-latest` with `lto = true` and `codegen-units = 1`.
3. Signs every generated bundle with the repository's Ed25519 private key.
4. Generates a consolidated `latest.json` manifest with release notes, download URLs, and cryptographic signatures.
5. Creates a GitHub Release publishing all installers, packages, and SHA-256 checksums (`SHA256SUMS.txt`).
