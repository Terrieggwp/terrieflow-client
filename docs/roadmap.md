# TerrieFlow Client roadmap

## First public beta gate

- [x] TerrieFlow branding, transparent application icon, and active/inactive tray icons;
- [x] Telegram and email/password authorization;
- [x] automatic account subscription synchronization;
- [x] active, grace-period, and expired-access UI behavior;
- [x] manual-only, Windows x64-only release workflow;
- [x] official update source and TerrieFlow artifact names;
- [x] orange high-latency state distinct from red errors;
- [ ] sanitized current screenshots captured from the final candidate;
- [ ] clean-machine install/uninstall check;
- [ ] real update from the previously published version, including rollback/failure evidence;
- [ ] Authenticode signing (not required for the first beta, but warning must remain visible);
- [ ] explicit approval to make the repository/release public.

## After beta

- move remaining compatibility paths and internal names through a tested migration;
- harden the privileged installation boundary and sign every executable/installer;
- add automated accessibility and high-DPI UI checks;
- qualify additional architectures/platforms one at a time;
- publish package-manager manifests only after their exact artifacts are supported.

Items are not promises or dates. Only release notes define what a published version supports.

