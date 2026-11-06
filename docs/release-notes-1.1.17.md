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
- Existing local profiles, account sessions and subscription state are retained
  during a normal update.
- `release.txt` remains at `1.1.15` until the 1.1.17 release is actually
  published, so unreleased clients do not advertise a nonexistent update.

## Before publication

Publication still requires a verified Windows x64 release bundle, installer and
updater smoke test, successful GitHub checks, and the owner's separate approval
to merge and publish the release.
