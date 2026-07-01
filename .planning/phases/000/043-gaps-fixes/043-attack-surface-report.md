# Attack Surface Scan

## Metadata
- Generated At: 2026-05-08T13:31:57Z
- Scope: `.planning/phases/043-gaps-fixes`
- Requested Report: `.planning/phases/043-gaps-fixes/043-attack-surface-report.md`
- Requested DB: `.planning/phases/043-gaps-fixes/043-attack-surface-db.jsonl`
- Max Variants: 20

## Scan Result

No candidate passed the pro-con audit and uniqueness gate.

### Rejection Summary

- `stage2-plaintext-secret-artifact`: the simulator Stage 2 debug lane does persist passwords, seed phrases, and receiver secrets into `wallets/private/wlt_secrets_debug.md`, but this exact surface already exists in the repository attack inventory as `AS-20260501-031` in `.planning/temp/attack-surface-db.jsonl`, so it was not re-admitted as a new accepted finding.
- `backup-size-seam-drift`: the backup-listing `file_len` seam is a phase-quality boundary issue, but it does not cross a security trust boundary strongly enough for this scan.
- `claim-tempdir-collision`: the release-load temp-dir naming issue is a real robustness concern, but it does not reach the security threshold for a distinct attack-surface admission here.

### Notes

- The scoped code was still inspected against the live simulator, wallet, storage, and utils surfaces.
- Because the only strong security candidate was a semantic duplicate of an existing accepted finding, no new JSONL row was appended.
