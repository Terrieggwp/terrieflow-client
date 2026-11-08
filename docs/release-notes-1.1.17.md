# TerrieFlow Client 1.1.17 — Windows beta

TerrieFlow Client 1.1.17 is the first Windows beta prepared from the stable
TerrieFlow interface line. The later experimental Claude redesign is not part
of this release.

## What changed

- TerrieFlow branding is applied to the application, tray states, shortcuts,
  installer and updater.
- Telegram and email/password sign-in are available from the client.
- Account sessions and TerrieFlow subscriptions are restored after restart.
- Account subscriptions are synchronized automatically after sign-in and on
  refresh.
- Active, grace-period and expired subscription states are handled consistently
  in the main window and Quick Panel.
- Expired subscriptions show the renewal flow and no longer expose routes that
  cannot be used.
- Quick Panel, route selection, VPN control and diagnostics remain available in
  the stable pre-redesign interface.
- The animated background uses the fluid display-rate loop and pauses when the
  window is inactive or reduced motion is requested.
- Windows packaging, installer and updater validation were hardened with pinned
  component checks and release-contract tests.

## Compatibility

- Windows x64 is the beta release target.
- The Linux-only GTK3 dependency currently resolves `glib 0.18.5`, which is
  affected by GHSA-wrw7-89jp-8q8g. It is not compiled into the Windows x64
  release. The temporary acceptance, evidence, restrictions, and remediation plan
  are recorded in
  [security-risk-acceptance-1.1.17.md](security-risk-acceptance-1.1.17.md).
- Existing local profiles, account sessions and subscription state are retained
  during a normal update.
- `release.txt` remains at `1.1.15` until the 1.1.17 release is actually
  published, so unreleased clients do not advertise a nonexistent update.

## Verification and downloads

- The `rc5-infra` release candidate was installed over RC4 and exercised on a
  second physical Windows PC.
- Installation and startup, preservation of authorization, subscriptions and
  settings, Quick Panel, tray integration, VPN, TUN, routing, connections and
  autostart were verified there.
- The executable is currently unsigned, so Windows SmartScreen may display a
  warning. Download only from the official TerrieFlow repository.
- Verify every downloaded asset with `RELEASE-SHA256SUMS.txt`.

