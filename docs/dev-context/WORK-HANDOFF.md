# TerrieFlow Client development handoff

Updated: 2026-10-15

## Repository and branch

- Project: TerrieFlow desktop client based on MultiCore.
- Upstream project: `https://github.com/fatyzzz/multicore`.
- Active development branch: `codex/terrieflow-branding`.
- License: retain the upstream GPL-3.0 license and attribution.

## Current state

The upstream MultiCore application builds successfully on Windows. TerrieFlow work so far is an isolated UI preview: it does not start the daemon, Mihomo, Xray, TUN, the Windows service, or the system proxy.

Implemented preview work:

- isolated TerrieFlow preview mode;
- TerrieFlow visual shell and branding;
- routes screen;
- split-access screen;
- subscription screen with local grace-state demonstration;
- diagnostics screen;
- restored the original MultiCore settings layout after an experimental redesign;
- enlarged protected subscription QR presentation;
- widened navigation so Russian labels fit.

Latest reviewed commit on the work laptop:

```text
a4c0366 fix: restore the MultiCore settings layout
```

The working tree was clean before this handoff document was added.

## Validation already completed

- `cargo check -p multicore-desktop` passed after the latest UI change.
- `cargo build -p multicore-desktop` passed for reviewed preview revisions.
- Preview executables launched successfully.
- Safety checks observed no newly started `multicore-daemon`, `mihomo`, or `xray` processes.

Generated preview executables are local review artifacts and are not the source of truth. Rebuild them from source on the current machine.

## Next development step

Perform a unified visual and state audit before connecting live TerrieFlow data:

1. Check every screen at Windows scaling 100%, 125%, and 150%.
2. Check minimum and expanded window sizes.
3. Fix clipped text, unexplained empty areas, wrapping, spacing, cards, colors, and icons.
4. Add consistent demonstration states: no subscription, loading, connecting, connected, disconnecting, error, expiring subscription, grace, expired subscription, empty devices/routes, and diagnostics error.
5. Define the TerrieFlow data contract for plan status, grace, traffic, devices, account links, renewal, and support.
6. Connect real data read-only only after the UI/state audit.

Known issue to address during the audit: server lists can appear as a large blank area in preview. The final UI should display subscription servers or a clear empty/error state.

## Safety boundaries

- Run TUN/VPN routing tests only on a personal network or mobile hotspot.
- Do not install or start the privileged service while connected to a corporate network.
- Do not change the system proxy during visual preview work.
- Never commit subscription URLs, access tokens, private keys, production configuration, or user databases.

## Continuing on another computer

1. Clone the private `Terrieggwp/terrieflow-client` repository.
2. Check out the active reviewed feature branch, or `main` after its pull request has been merged.
3. Open the repository folder in Codex.
4. Ask Codex to read this file, inspect `git status` and recent commits, then continue with the visual/state audit.
