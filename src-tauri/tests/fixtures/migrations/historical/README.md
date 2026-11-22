# Historical schema fixtures

These SQL files are copied byte-for-byte from this repository's recorded commits. `manifest.json` identifies the source commit/path and SHA-256 for each file; the regression verifies those hashes without fetching Git history or using the network. Do not reformat these SQL files or substitute current DDL. Every record inserted by `tests/support/historical_vault.rs` is invented. No encrypted database, personal export or credential is included.

| Starting version   | Construction                                                                                                  |
| ------------------ | ------------------------------------------------------------------------------------------------------------- |
| 2                  | Original `20a8fc2` base DDL                                                                                   |
| 3                  | `589967e` base DDL after the saved-version approval change                                                    |
| 4                  | `2ed7241` base and provider-curation DDL                                                                      |
| 5, freshly created | Version-4 base and the widened curation DDL (identical in `b5b314a` and `6be64ee`)                            |
| 5, upgraded from 4 | Version-4 DDL followed by the original `b5b314a` provider migration; retains the earlier saved-job constraint |
| 6                  | Version-4 base and `6be64ee` curation DDL                                                                     |
| 7                  | Version-4 base and `a818a43` curation/owner-hold DDL                                                          |

The helper sets the corresponding `user_version`, as the historical initializer did. It includes raw JSON, alternate branches, manual privacy, retained sensitivity, newly detectable sensitive content, linked/unlinked approvals, an orphaned source, briefs, tags, snapshots, agent grants, curation history/retry receipts and a paused curation job. Runtime checks cover real encrypted files, upgrade, reopen, new Grok/Claude jobs, encrypted backup and historical backup restoration.

The failure fixtures deliberately add an aborting SQL trigger or invalid foreign reference. Tests check transactional rollback and retry after removing only that artificial fault. This is not physical disk-full or power-loss evidence. These are faithful historical schema constructions with invented records, not vaults produced by executing every old app binary or representative user data. Schema 1 predates the recorded app baseline and retains its separate plaintext-migration regression; no new historical-v1 acceptance is claimed.
