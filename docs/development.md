# TerrieFlow Client development

The client is written in Rust with a native Slint UI. The first supported release target is Windows 10/11 x64.

## Prerequisites

- Rust `1.98.1-x86_64-pc-windows-msvc` with `rustfmt` and `clippy`;
- Visual Studio Build Tools / Windows SDK;
- Git Bash and pinned Go tooling for the patched Mihomo build;
- Inno Setup 6.7.3 for the installer;
- the pinned Skia artifacts described in [renderer.md](renderer.md) when a local rebuild requires them.

## Verify a checkout

```powershell
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-package-windows-release.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-windows-installer-contract.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-release-workflow-contract.ps1 -ExpectedVersion 1.1.17
```

The normal CI still compiles and tests useful Linux code, but it does not create official Linux release artifacts. Windows x86 and ARM64 code is retained for future verification and is not part of the first beta.

## Build the Windows x64 candidate

Build patched Mihomo first, then package with the one allowed update repository:

```powershell
& 'C:\Program Files\Git\bin\bash.exe' scripts/build-mihomo.sh --output dist/mihomo windows-amd64
$env:MULTICORE_MIHOMO_BINARIES_DIR = (Resolve-Path dist/mihomo).Path
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-windows-release.ps1 `
  -DestinationPath dist/release/terrieflow-windows-x64 `
  -UpdateRepository Terrieggwp/terrieflow-client `
  -ReleaseAssetPath dist/release/TerrieFlow-Client-windows-x64.zip
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-windows-installer.ps1 `
  -PackagePath dist/release/terrieflow-windows-x64 `
  -OutputDirectory dist/release `
  -AppVersion 1.1.17
```

Do not distribute a bare Cargo executable. A valid package also contains the daemon, updater, core host, pinned cores, dashboard assets, notices, licenses, manifests, and checksums.

## Compatibility paths

- user data: `%LOCALAPPDATA%\MultiCore`;
- profiles: `%LOCALAPPDATA%\MultiCore\profiles`;
- updater state: `%LOCALAPPDATA%\MultiCore\updates`;
- internal executable names: `multicore-daemon.exe`, `multicore-updater.exe`, `multicore-core-host.exe`.

These names are intentionally retained for migration compatibility. Do not rename them without a tested data and service migration.

## Releases

`.github/workflows/release.yml` is manual-only. The default run builds and retains a candidate; publication additionally requires the `publish` input and approval in the `terrieflow-release` GitHub environment. See [release-process.md](release-process.md).
