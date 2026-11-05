# Security policy

## Supported versions

Security fixes are considered for the default branch and the latest official Windows x64 release. Preview builds, old releases, and platforms not listed in the current release notes may be unsupported.

## Report a vulnerability privately

Use this repository's **Security → Report a vulnerability** form (GitHub Private Vulnerability Reporting / Security Advisories). Do not open a public issue for an undisclosed vulnerability.

Include the affected version or commit, Windows version, impact, safe reproduction steps, and a minimal proof of concept where appropriate. Remove access tokens, passwords, subscription URLs, personal data, private keys, and live account details from reports and diagnostics.

Maintainers will assess reports as capacity allows. This project does not promise a response deadline, disclosure date, bounty, or fix for every report. Issues in Xray-core, Mihomo, Windows, GitHub, or another dependency may need to be reported to that project's maintainer.

## Scope

The policy covers TerrieFlow Client source code, account integration, updater, release packaging, desktop UI, daemon, and Windows privileged core host in this repository. The current unsigned beta and user-writable compatibility data directory are known hardening limitations documented in the roadmap.

Never include real secrets in tests, issues, pull requests, commits, release notes, or screenshots.
