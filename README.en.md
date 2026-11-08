<p align="center">
  <img src="brand/TerrieFlow_Logo_Kit/01_primary_horizontal/png/terrieflow-horizontal-on-dark_512.png" width="360" alt="TerrieFlow">
</p>

<p align="center"><strong>The official TerrieFlow client for Windows</strong></p>
<p align="center"><a href="README.md">Русский</a> · <a href="PRIVACY.md">Privacy</a> · <a href="SECURITY.md">Security</a></p>

TerrieFlow Client is a desktop client for TerrieFlow and compatible VPN subscriptions. It combines account sign-in, automatic subscription import, route selection, Xray/Mihomo control, and verified application updates.

## Features

- TerrieFlow sign-in with Telegram or email and password;
- automatic subscription synchronization from the connected account;
- optional manual import of a third-party VPN subscription without signing in;
- automatic, maximum-speed, and maximum-availability route modes;
- country routes, latency measurement, and a tray Quick Panel;
- consistent active, grace-period, and expired-subscription states;
- Xray and a patched Mihomo build in one verifiable package;
- updates exclusively from `Terrieggwp/terrieflow-client`, with version, architecture, and SHA-256 validation.

## Download

The first official beta supports **Windows 10/11 x64 only**.

[Download TerrieFlow Client for Windows x64](https://github.com/Terrieggwp/terrieflow-client/releases/latest/download/TerrieFlow-Client-Setup-x64.exe)

Verify the download against `RELEASE-SHA256SUMS.txt` from the same release. The current beta is not yet code-signed, so Windows SmartScreen may display a warning. Do not install a file if its name, source, or checksum differs.

## Getting started

1. Install and launch TerrieFlow.
2. Sign in with Telegram or email; an eligible subscription is imported automatically.
3. Alternatively, paste an external VPN subscription link manually.
4. Select a route and enable protection.

For compatibility with existing installations, application data currently remains in `%LOCALAPPDATA%\MultiCore`. The installed application and user-facing product are TerrieFlow.

## Beta status

Windows x64 is the only verified platform in the first public beta cycle. Linux, Windows x86, and Windows ARM64 remain useful development targets in the source tree, but are not supported release platforms yet. See the [roadmap](docs/roadmap.md) and [release process](docs/release-process.md) for limitations and release gates.

Current screenshots must use a test account and sanitized data. The checklist is in [docs/screenshots.md](docs/screenshots.md); legacy MultiCore images are not presented as the current product.

## Development

The project uses Rust `1.98.1`, Windows SDK/MSVC, and the prerequisites in [docs/development.md](docs/development.md).

```powershell
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-release-workflow-contract.ps1 -ExpectedVersion 1.1.17
```

See [docs/architecture.md](docs/architecture.md) for trust boundaries and [docs/how-it-works.md](docs/how-it-works.md) for the user flow.

## License and attribution

This project is licensed under [GPL-3.0](LICENSE). TerrieFlow Client is based on the open-source [MultiCore](https://github.com/fatyzzz/multicore) project. Notices and corresponding source archives for shipped GPL/MPL components accompany releases. TerrieFlow trademarks and brand assets do not alter the source-code license.
