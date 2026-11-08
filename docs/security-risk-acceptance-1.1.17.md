# TerrieFlow Client 1.1.17 security risk acceptance

## GHSA-wrw7-89jp-8q8g (`glib` 0.18.5)

**Decision:** temporarily accepted for the unsupported Linux source line of
TerrieFlow Client 1.1.17. This acceptance does not cover the Windows x64 release
artifact and does not disable or dismiss the Dependabot alert.

The advisory affects `glib >= 0.15.0, < 0.20.0`; the first fixed release is
`0.20.0`. The issue is an unsound implementation of `Iterator` and
`DoubleEndedIterator` for `glib::VariantStrIter`, which can result in undefined
behaviour. The repository currently resolves `glib 0.18.5` through the Linux GTK3
tray stack:

`tray-icon -> libappindicator -> gtk -> glib`

### Platform boundary and evidence

- TerrieFlow Client 1.1.17 is released only for Windows 10/11 x64. Linux is not a
  supported or published 1.1.17 platform.
- `apps/multicore-desktop/Cargo.toml` enables `gtk 0.18` and the
  `tray-icon/libappindicator` feature only under `cfg(target_os = "linux")`.
- The Windows target uses `tray-icon` with default features disabled and does not
  select GTK, libappindicator, or `glib`.
- CI for merge commit `5990343674829c2d806c165de0bd42289508ed07` confirms the
  boundary: the Windows job compiled none of `glib`, GTK, or libappindicator,
  while the Linux job compiled `glib 0.18.5` and the GTK/libappindicator stack.
- Linux `glib` usage is confined to Linux-only implementations of the tray and
  save-dialog modules. The affected `VariantStrIter`/`array_iter_str` API is not
  used by the TerrieFlow source at the time of this decision.
- `Cargo.lock` still contains the package because it records dependencies for all
  supported build targets; lockfile presence alone does not make the crate part of
  the Windows x64 binary.

Evidence:

- [Windows CI job](https://github.com/Terrieggwp/terrieflow-client/actions/runs/42491083398/job/128763392313)
- [Linux CI job](https://github.com/Terrieggwp/terrieflow-client/actions/runs/42491083398/job/128763392136)
- [GitHub advisory GHSA-wrw7-89jp-8q8g](https://github.com/advisories/GHSA-wrw7-89jp-8q8g)

### Residual risk and restrictions

The vulnerable crate remains in the Linux dependency graph. Absence of the known
affected API in current source reduces reachability but is not treated as a fix.
Linux builds must not be advertised or published as supported TerrieFlow 1.1.17
artifacts under this acceptance. Dependabot, Dependency Graph, and the existing CI
checks remain enabled.

This decision must be reviewed immediately if Linux distribution is proposed, if
code starts using `VariantStrIter`/`array_iter_str`, or if the advisory scope or
severity changes.

### Follow-up plan

1. Track a post-1.1.17 dependency-maintenance change for the Linux tray stack.
2. Evaluate a coordinated GTK/libappindicator upgrade or a replacement Linux tray
   backend that resolves `glib >= 0.20.0`; do not force an isolated `glib` upgrade
   across incompatible GTK bindings.
3. Validate the chosen change with the full Linux CI suite, native Wayland window
   lifecycle test, tray behaviour, save-dialog behaviour, and a dependency audit.
4. Remove this acceptance only after the Linux dependency graph no longer contains
   an affected `glib` version and the Dependabot alert is resolved.

**Recorded:** 2026-11-08

**Review deadline:** before any supported Linux release, and no later than the next
planned dependency-maintenance cycle after 1.1.17.

