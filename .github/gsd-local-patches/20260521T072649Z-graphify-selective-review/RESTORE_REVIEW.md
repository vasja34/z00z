# Graphify Selective Restore Review

## 🔎 Scope

Reviewed the remaining pre-upgrade local patch sets against live GSD `v1.42.3` for:

- `.github/get-shit-done/bin/lib/graphify.cjs`
- `.github/skills/gsd-graphify/SKILL.md`

Backup sources reviewed:

- `.github/gsd-local-patches/get-shit-done/bin/lib/graphify.cjs`
- `.github/gsd-local-patches/skills/gsd-graphify/SKILL.md`

Current live snapshots preserved in this directory before finalizing the verdict.

## ⚖️ Verdict

No selective restore was applied to either live file.

This is an intentional no-op selective merge result, not an omission.

## 🧩 Review Findings

### `.github/get-shit-done/bin/lib/graphify.cjs`

Do not restore the old local hunks.

Reasons:

- The old patch set still uses `.graphify` as `graphify_out`, while live `v1.42.3` uses `graphify-out`.
- The old patch set uses `child_process.spawnSync` and `atomicWriteFileSync`; live `v1.42.3` moved to `execTool`, `execGit`, and `platformWriteSync`.
- Live `v1.42.3` adds typed `GRAPHIFY_REASON` failure classification.
- Live `v1.42.3` adds commit-freshness fields: `built_at_commit`, `current_commit`, `commits_behind`, and `commit_stale`.

Restoring old hunks here would regress the newer runtime contract and remove upstream hardening.

### `.github/skills/gsd-graphify/SKILL.md`

Do not restore the old local hunks.

Reasons:

- The old patch set depends on `Task`-based build execution.
- Live `v1.42.3` intentionally changed the build flow to inline foreground execution.
- The live skill now copies artifacts from `graphify-out`, not `.graphify`.
- The live skill renders commit-freshness status and includes the MVP-mode node-rendering guidance.

Restoring the old Task/subagent build path would reintroduce the pre-fix build flow that upstream replaced to avoid truncated graph artifacts.

## ✅ Safe Restore Decision

For these two patch sets, the safe selective-merge decision is:

- Preserve the old local versions only as backup material.
- Keep the live `v1.42.3` implementations unchanged.
- Revisit only if a new repo-specific graphify requirement appears that is not covered by upstream behavior.
