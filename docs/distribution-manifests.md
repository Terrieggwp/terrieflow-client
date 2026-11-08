# Distribution scope

The first official TerrieFlow beta publishes only these Windows x64 artifacts:

- `TerrieFlow-Client-Setup-x64.exe`;
- `TerrieFlow-Client-windows-x64.zip`;
- `RELEASE-SHA256SUMS.txt`;
- `Xray-core-source.zip`;
- `mihomo-source.zip`.

All assets belong to one numeric GitHub release in `Terrieggwp/terrieflow-client`. The client is compiled with that same repository as its update source. No legacy mirror or personal access token is required.

The source tree still contains Linux, winget, AUR, Windows x86, and Windows ARM64 packaging work inherited from MultiCore. It is retained for future development, but those packages are not generated, advertised, or uploaded by the official first-beta workflow. Re-enable a platform only after its installation, update, rollback, and signing story is independently verified.

Nothing is submitted automatically to package managers.
