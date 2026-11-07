# Windows x64 release process

## Safety rules

- Releases are numeric: `vMAJOR.MINOR.PATCH`.
- The official update repository is exactly `Terrieggwp/terrieflow-client`.
- The first beta publishes Windows x64 only.
- The Windows release runner must expose the reviewed Skia archive directory as
  `MULTICORE_SKIA_BINARIES_DIR`. The workflow verifies the filename and SHA-256
  from `packaging/windows-x64/versions.json` before building.
- Do not change `release.txt` before the complete GitHub Release exists.
- Do not publish or change repository visibility without explicit owner approval.
- The `terrieflow-release` GitHub environment should require a human reviewer.

## Prepare

1. Run the full test suite and secret scan on the exact release commit.
2. Confirm `Cargo.toml`, `Cargo.lock`, installer default, tag, and release notes use the same numeric version.
3. Capture sanitized screenshots and complete the clean-machine install/uninstall check.
4. Create the numeric tag only after review.
5. Run **Prepare Windows x64 release** with `publish=false`. Download and inspect the retained candidate and `RELEASE-SHA256SUMS.txt`.

## Required updater E2E test

On an isolated Windows x64 test machine:

1. Install the currently published version through its official installer.
2. Confirm the client is compiled for `Terrieggwp/terrieflow-client` and has no embedded token.
3. Publish or expose the reviewed candidate through the exact production release path only after explicit approval.
4. Trigger the in-app update and verify download, digest/architecture checks, application restart, preserved account/profile state, and the displayed new version.
5. Exercise an intentionally invalid package in a controlled non-production fixture and verify refusal/rollback without data loss.
6. Save non-secret evidence: versions, hashes, exit status, and sanitized screenshots/logs.

A local installer smoke test is useful but does not replace this network E2E test. Publication is blocked until the real path succeeds.

## Publish

After explicit approval and E2E success, rerun the workflow with `publish=true`. The protected environment provides the second approval. The workflow publishes only the fixed asset list, then updates `release.txt` to the released number. Verify the latest-release page and anonymous downloads before announcing the release.

## Repository opening checklist

- default branch contains the reviewed commit and no private-only branch is merged accidentally;
- Security Advisories / private vulnerability reporting are enabled;
- About text: `Official Windows client for TerrieFlow — account sign-in, automatic subscription sync, Xray/Mihomo routing and verified updates.`;
- suggested topics: `terrieflow`, `vpn-client`, `windows`, `xray`, `mihomo`, `rust`, `slint`;
- README links work anonymously;
- branch protection and the `terrieflow-release` environment are configured;
- no release, visibility, About, topics, or protection setting is changed without explicit owner approval.
