# Agent access history and individual controls

**Agent access → Your connections** lists each registration's existing scope, paused state and last authenticated request time. **Pause access** keeps the credential and scope but blocks every tool, including connection verification. **Resume access** restores that same grant; it never enables global sharing, unlocks the vault or changes provider scope. Pauses persist across restart and encrypted backup. Already returned context cannot be recalled.

**Revoke access** invalidates one credential. **Revoke all connections…** reviews the current list, requires acknowledgement and stops global sharing/work sessions before attempting revocation. A changed registration list rejects the deletion and asks for fresh review. Neither operation removes conversations, memories or provider drafts. After revocation, reconnecting requires new settings. Restore/password change still removes all registrations; retained history is not permission to reconnect.

Revoke-all records the verified owner action before ending an extended work session, so normal idle expiry cannot interrupt the credential deletion midway. Already-expired sessions remain locked. The owner view ignores obsolete refresh/page replies after newer reads or control changes.

If a storage error prevents a pause or revocation from being saved, the desktop session stops global sharing. The UI reports that fallback; it must not claim the persistent change succeeded. Successful calls already being processed/written under the session mutex finish before a pause can take effect. Later calls are rechecked. Resume failures never broaden access.

## What is recorded

The SQLCipher-encrypted vault retains at most **1,000 events**, ordered by monotonically increasing event ID. Timestamps use the Mac's clock and are displayed in local time. The owner view loads 50 at a time and has manual refresh/older-page controls.

| Stored metadata                           | Meaning                                                                                                                                                                                         |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Client label and optional registration ID | Owner-chosen label; the ID becomes null on revocation, retaining the historical label. Labels themselves may be personal and stay encrypted. IDs are not exposed in the activity view response. |
| Time                                      | When the event was recorded; not an independently verified timestamp.                                                                                                                           |
| Fixed operation category                  | One of the eight MCP tool names, a generic unsupported-tool category, or a connection-management action. Unknown caller-supplied tool text is never stored.                                     |
| Result                                    | Started, processed, rejected/failed, or blocked because paused.                                                                                                                                 |

No prompt, query, response, error text, source/conversation ID, curation body, token or password is stored in this record. Agents have no history-reading or control tool. Scope changes still require revoking and creating settings; pause is not a scope editor.

Only correctly authenticated requests reaching an unlocked, sharing-enabled broker enter the request history. Invalid/revoked credentials, malformed requests and attempts while locked or globally sharing-off are not attributed or recorded. A valid paused client can produce a blocked event. Connection-management events are owner actions and may occur with sharing off.

Before dispatch, the backend commits a Started event and last-request timestamp. If that fails, no tool runs. After dispatch it records the outcome before responding. If outcome recording fails, the request returns an error warning that it may have run; Started remains as an uncertain outcome. Processed means server processing succeeded, **not** that the client received, understood or used a reply. Oversized replies and disconnected clients can have a Processed record. Current-unlock setup verification continues to require a successfully written, bounded response and remains separate from persistent history.

This is a useful local activity record, not a tamper-proof or complete forensic audit. An authorized client can generate enough requests to roll older events out of the cap; the owner can clear records; backups preserve earlier history. No telemetry is sent anywhere.

## Clear and retention

**Clear activity…** requires acknowledgement and clears retained events, last-request times and ephemeral request-seen evidence for the current vault session. It preserves credentials, scopes and paused states. The operation is atomic and rejects a stale unlock generation; new requests may immediately create new events. It does not erase older encrypted backups or guarantee physical erasure from every filesystem snapshot. Revocation preserves retained activity labels until normal rollover or explicit clearing.

## Storage compatibility

Schema **9** adds `agent_controls` and `agent_activity` inside the existing encrypted database. Migration from schema 8 is additive/transactional: existing clients retain their scopes, default to unpaused, and have no invented prior history or last-used timestamp. Saved-memory approval is unchanged. Failed creation leaves the previous schema version and records intact. Schema 2–7 upgrade tests now target 9; a dedicated version-8 DDL test covers the new step.

Encrypted backups carry schema 9. Older builds that support only schema 8 reject these vaults instead of silently ignoring paused permissions. The SQLCipher/Argon2 format and password are unchanged. Preserve a verified backup before upgrades; do not edit schema-version numbers to bypass the version guard.

## Verification

Backend checks cover all-tool pause enforcement, independent clients, scope-preserving resume, broker request-seen behavior, sanitized metadata, no invalid-token attribution, bounded pagination, clear/revoke-all review/session rules, injected recording failures, global-sharing fallback, encrypted reopen/backup/restore and version-8 migration rollback. Frontend checks cover control labels and honest activity-result wording. Native acceptance is recorded separately in NATIVE_WALKTHROUGH; passing automated checks alone is not native or independent security acceptance.
