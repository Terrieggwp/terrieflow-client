# Pinned Windows x64 Skia release dependency

This directory is the durable, versioned source used by the GitHub-hosted
Windows release workflow. The archive is committed as an ordinary Git object
instead of an Actions artifact or an LFS object, so its availability does not
depend on retention periods, runner state, credentials, or LFS bandwidth.

Archive:

`skia-binaries-a25a0fdb7d90429aa2d1-x86_64-pc-windows-msvc-d3d-gl-jpegd-jpege-pdf-textlayout-static.tar.gz`

- Size: `25357970` bytes
- SHA-256: `dfae17eb3e66eb7d242b557efff3599bcc12715da492f5e4772bb0b12b24087f`
- `skia-bindings`: `0.99.0`
- Skia tag: `m150-0.98.1`
- Skia commit: `90af6f5c5a3ed8788b805fe6ca3fcdf28c2c2bc9`
- Build: `scripts/build-skia-binaries.ps1`, LLVM 23.1.2 `clang-cl`, static CRT
- License: BSD-3-Clause; the release bundle carries the corresponding notices

The release workflow reads the expected key and SHA-256 from
`packaging/windows-x64/versions.json` and fails before building if the committed
archive differs. Do not replace or repack the archive in place. A future Skia
update must use a new reviewed archive and update the manifest deliberately.

Independent rebuilds are not expected to reproduce the same archive bytes:
COFF timestamps, archive member paths, and gzip metadata affect the output.
That is why release builds reuse this exact reviewed archive.
