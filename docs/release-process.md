# Windows x64 release process

## Safety rules

- Releases use numeric tags: `vMAJOR.MINOR.PATCH`.
- The official update repository is exactly `Terrieggwp/terrieflow-client`.
- The first beta publishes Windows x64 only.
- Release builds run on GitHub-hosted `windows-2022`; no persistent or privileged
  self-hosted runner is part of the release path.
- The reviewed Skia archive is an immutable, versioned Git object under
  `build-support/release-dependencies/skia`. Before compilation, the prepare job
  verifies its filename and SHA-256 against `packaging/windows-x64/versions.json`
  and fails closed on any mismatch.
- Do not change `release.txt` before the complete GitHub Release exists.
- Do not publish, merge a release PR, create a release tag, or change repository
  visibility without explicit owner approval.
- The `terrieflow-release` GitHub environment should require an independent human
  reviewer when a second trusted GitHub participant is available. Do not claim
  independent approval when it is not configured and tested.

## Release workflow architecture

The manually dispatched **Prepare Windows x64 release** workflow accepts a numeric
tag and a `publish` boolean. It does not run for pull requests or untrusted refs.

The `prepare` job has `contents: read` and `actions: read` only. It checks out the
requested tag, verifies that the tag resolves to the checked-out commit, requires
that commit to be contained in `origin/main`, and requires the tag to match the
desktop package version. It then runs the release gate, source tests, strict
Clippy, packaging contracts, pinned Mihomo/Skia builds, portable bundle checks,
packaged-daemon smoke test, installer build/smoke test, license/source preservation,
and release-note checks.

The prepared candidate is a short-lived Actions artifact used only to hand the
verified files to the `publish` job. `CANDIDATE-SHA256SUMS.txt` covers every file in
that handoff. The `publish` job downloads the exact named artifact, rejects missing,
extra, duplicate, or hash-mismatched files, and only then receives `contents: write`
inside the protected `terrieflow-release` environment. The Git repository, not an
Actions artifact, is the durable source of the pinned Skia dependency.

## Prepare

1. Run the full test suite and secret/history audit on the exact release commit.
2. Confirm `Cargo.toml`, `Cargo.lock`, installer default, tag, and release notes use
   the same numeric version.
3. Complete the physical-machine install/upgrade check and retain sanitized evidence.
4. Merge the reviewed release PR only after owner approval.
5. Create the numeric tag on the approved `main` commit only after separate owner
   approval.
6. Run **Prepare Windows x64 release** with `publish=false`.
7. Download the retained candidate, verify `CANDIDATE-SHA256SUMS.txt` and
   `RELEASE-SHA256SUMS.txt`, inspect the fixed asset list, and compare the installer
   and ZIP with the physically tested candidate. A different binary hash requires
   investigation and may require another physical-machine test.
8. Stop and obtain explicit owner approval before `publish=true`.

## Initial publication exception for 1.1.17

The first public release has no earlier public TerrieFlow release from which a real
network updater E2E can start. Therefore the normal pre-publication updater gate
below cannot be satisfied for `1.1.17` alone. The initial channel is established in
this strict order:

1. Physically test the `1.1.17` candidate, including install over the prior RC and
   preservation of authorization, subscriptions, settings, Quick Panel, tray, VPN,
   TUN, routes, connections, and autostart.
2. Publish `1.1.17` only after prepare inspection, repository protections, and
   explicit owner approval.
3. Verify anonymous repository/release/API access, downloads, hashes, updater
   repository identity, and absence of GitHub HTTP 404 responses.
4. In a later, separately approved development stage, prepare and publish `1.1.18`.
5. Perform the real updater E2E from public `1.1.17` to public `1.1.18`, including
   digest/architecture checks, restart, preserved state, and rollback/refusal tests.
6. Announce the public beta only after that E2E succeeds.

This is a one-time bootstrap exception. It does not waive the updater E2E gate for
`1.1.18` or any later release.

## Required updater E2E test after the initial release

On an isolated Windows x64 test machine:

1. Install the currently published version through its official installer.
2. Confirm the client is compiled for `Terrieggwp/terrieflow-client` and has no
   embedded token.
3. Publish the reviewed next candidate through the exact production release path
   only after explicit approval.
4. Trigger the in-app update and verify download, digest/architecture checks,
   application restart, preserved account/profile state, and the displayed version.
5. Exercise an intentionally invalid package in a controlled non-production fixture
   and verify refusal/rollback without data loss.
6. Save non-secret evidence: versions, hashes, exit status, and sanitized
   screenshots/logs.

A local installer smoke test is useful but does not replace this network E2E test.
Except for the one-time `1.1.17` bootstrap procedure above, publication is blocked
until the real update path succeeds.

## Publish

After explicit owner approval, rerun the workflow with `publish=true`. The publish
job must pass the configured `terrieflow-release` environment rules. It publishes
only the fixed asset list and updates `release.txt` only after the GitHub Release is
successfully created. Verify the latest-release page, Releases API, anonymous
downloads, and published SHA-256 values before announcing anything.

## Repository opening checklist

- the default branch contains the reviewed commit and no experimental/private-only
  branch was merged accidentally;
- the complete Git history was audited for credentials, certificates, private
  infrastructure, production addresses, and confidential configuration;
- branch protection requires pull requests, strict `windows` and `linux` checks,
  resolved conversations, and blocks force-push/deletion and bypass where available;
- `terrieflow-release` restricts release tags and its actual approval behavior has
  been tested;
- Dependency Graph, Dependabot alerts, Security Advisories, and private vulnerability
  reporting are enabled where available;
- Actions have read-only `GITHUB_TOKEN` by default and cannot create or approve pull
  requests; untrusted pull-request code cannot reach publish permissions;
- GPL-3.0, BSD-3-Clause, all third-party notices, and corresponding source archives
  are present and anonymously accessible;
- About text: `Official Windows client for TerrieFlow — account sign-in, automatic subscription sync, Xray/Mihomo routing and verified updates.`;
- suggested topics: `terrieflow`, `vpn-client`, `windows`, `xray`, `mihomo`, `rust`,
  `slint`;
- README and TerrieFlow links work anonymously;
- no release, visibility, About, topics, or protection setting is changed without
  explicit owner approval.
