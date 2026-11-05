# Architecture and trust boundaries

## Components

| Component | Privilege | Responsibility |
| --- | --- | --- |
| `TerrieFlow.exe` | current user | Slint UI, account flow, local preferences, commands |
| account API client | current user/network | Telegram and email sign-in, profile and subscription access |
| Windows Credential Manager | current user/OS | persistent session credential |
| `multicore-daemon.exe` | current user | subscription state, configuration, connection orchestration |
| `multicore-core-host.exe` | elevated on demand | controlled launch and cleanup of network cores |
| Mihomo | child of core host | TUN/system proxy, DNS, routing, selectors, latency |
| Xray | child of core host | additional outbound transports |
| `multicore-updater.exe` | update boundary | verified staging, replacement, rollback |

## Data flow

```text
TerrieFlow UI → authenticated local daemon → privileged local core host → Mihomo/Xray
      │                    │
      │                    └→ subscription provider
      ├→ TerrieFlow account API
      └→ official GitHub release metadata and assets
```

The daemon is not a public service. Local control uses authenticated loopback or named-pipe channels. Subscription URLs and account sessions are secrets and must never appear in public diagnostics.

## Update boundary

Official builds accept updates only from `Terrieggwp/terrieflow-client`. The updater rejects non-numeric versions, wrong repositories, unexpected asset names, wrong architectures, unsafe ZIP paths, digest/size mismatches, and incomplete packages. It stages changes before replacement and retains rollback behavior.

`release.txt` is not an integrity authority. It is only a version hint; the GitHub release and asset checks remain authoritative.

## Known hardening limits

- the first beta is not Authenticode-signed;
- historical data and some runtime paths use the `MultiCore` name;
- compatibility/user-writable locations are weaker than a fully machine-protected signed installation;
- a real update from an older public release to the new release must pass before publication.

