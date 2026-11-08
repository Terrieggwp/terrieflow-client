# Implementation and release risks

| Risk | Current control | Remaining action |
| --- | --- | --- |
| Wrong update repository | compile-time repository plus URL/repository validation | inspect final binary and perform real update |
| Partial or mixed release | fixed manifest, checksums, manual workflow, environment approval | publish only after candidate evidence is reviewed |
| Broken update/rollback | staged update, digest validation, local smoke contracts | complete public older-to-newer E2E test |
| Secret leakage | Credential Manager, redaction, `.gitignore`, repository scan | repeat scan on the exact release commit |
| Unsigned binaries | SHA-256 manifest and explicit warning | obtain Authenticode certificate |
| Privilege boundary in user-writable paths | constrained core host and verified inventory | move to signed, machine-protected installation |
| Unsupported platforms mistaken for releases | Windows x64-only workflow and documentation | qualify each platform before advertising it |
| Stale screenshots or private data | screenshot checklist | capture sanitized images from final candidate |

