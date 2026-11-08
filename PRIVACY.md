# Privacy notice

TerrieFlow Client is a native Windows application. This notice describes the data handled by the client; the TerrieFlow website and account service may publish additional terms.

## Account and subscription data

When you sign in through Telegram or with email and password, the client communicates with the TerrieFlow account API to authenticate, retrieve basic profile information, and discover the VPN subscription available to that account. The client can then import and periodically refresh that subscription. Account API operators can observe normal request metadata such as IP address, time, client version, and device information included by the service contract.

Session credentials are stored on Windows using Credential Manager (`CredWriteW`/`CredReadW`) and are scoped to the current user. Passwords are used for sign-in and are not deliberately persisted by the client. Signing out removes the saved session credential.

## Data stored on this device

For compatibility with earlier versions, application state is stored under `%LOCALAPPDATA%\MultiCore`. It can include subscription URLs, profile metadata, downloaded Xray/Mihomo configurations, selected routes, UI preferences, updater results, and a stable local device identifier. Subscription URLs can contain credentials and should be treated as secrets.

The stable device identifier and its lock file are protected for the current Windows user. Other application files are not claimed to be encrypted. Uninstalling may preserve local state so a later version can reuse it; use the application's “delete all data” action or remove the directory after fully exiting TerrieFlow if you want to remove it.

Diagnostics are bounded and redacted before display, but users should still review any log before sharing it.

## Subscription provider requests

Refreshing a subscription sends a request to its URL. Current provider requests may include `x-hwid`, `x-device-os`, `x-ver-os`, and `x-device-model`. Providers can also observe standard network metadata. A manually imported third-party provider has its own privacy practices, which TerrieFlow does not control.

## Updates

Official builds check `Terrieggwp/terrieflow-client` on GitHub for numeric stable releases and download the matching Windows x64 assets. No GitHub personal access token is embedded in the client. GitHub receives normal request metadata and its privacy terms apply.

## Local privileged components

The desktop and daemon use authenticated local channels. Enabling protection may invoke Windows UAC for the core host, which starts and stops Xray or Mihomo. These channels are not hosted TerrieFlow network services.

Current beta binaries are unsigned. This is a security and distribution limitation, not an additional data collection mechanism.
