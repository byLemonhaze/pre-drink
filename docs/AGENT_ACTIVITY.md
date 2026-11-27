# Agent access history and individual controls

**Agent access → Your connections** lists each registration's existing scope, paused state and last authenticated request time. **Pause access** keeps the credential and scope but blocks every tool, including connection verification. **Resume access** restores that same grant; it never enables global sharing, unlocks the vault or changes provider scope. Pauses persist across restart and encrypted backup. Already returned context cannot be recalled.

**Revoke access** opens a review naming that connection and requires acknowledgement before invalidating its credential. Cancel changes nothing. Other connections and global sharing stay unchanged on success. **Revoke all connections…** reviews the current list, requires acknowledgement and stops global sharing/work sessions before attempting revocation. A changed registration list rejects the deletion and asks for fresh review. Neither operation removes conversations, memories or provider drafts. After revocation, reconnecting requires new settings. Restore/password change still removes all registrations; retained history is not permission to reconnect.

Revoke-all records the verified owner action before ending an extended work session, so normal idle expiry cannot interrupt the credential deletion midway. Already-expired sessions remain locked. The owner view ignores obsolete refresh/page replies after newer reads or control changes.

If a storage error prevents a pause or revocation from being saved, the desktop session stops global sharing. The UI reports that fallback; it must not claim the persistent change succeeded. Successful calls already being processed/written under the session mutex finish before a pause can take effect. Later calls are rechecked. Resume failures never broaden access.

## What is recorded

The SQLCipher-encrypted vault retains at most **100 events per registered connection**, with a **1,000-event overall ceiling**, ordered by monotonically increasing event ID. Timestamps use the Mac's clock and are displayed in local time. The owner view loads 50 at a time and has manual refresh/older-page controls.

| Stored metadata                           | Meaning                                                                                                                                                                                         |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Client label and optional registration ID | Owner-chosen label; the ID becomes null on revocation, retaining the historical label. Labels themselves may be personal and stay encrypted. IDs are not exposed in the activity view response. |
| Time                                      | When the event was recorded; not an independently verified timestamp.                                                                                                                           |
| Fixed operation category                  | One of the eight MCP tool names, a generic unsupported-tool category, or a connection-management action. Unknown caller-supplied tool text is never stored.                                     |
| Result                                    | Started, processed, rejected/failed, or blocked because paused.                                                                                                                                 |

No prompt, query, response, error text, source/conversation ID, curation body, token or password is stored in this record. Agents have no history-reading or control tool. Scope changes still require revoking and creating settings; pause is not a scope editor.

Only correctly authenticated requests reaching an unlocked, sharing-enabled broker enter attributed request history. Invalid, unknown and revoked credentials reaching that boundary increment one encrypted anonymous counter; no submitted identity, tool text or per-attempt row is retained. Malformed/oversized requests, rejected peer identity, and attempts while locked or globally sharing-off are not counted. A valid paused client can produce a blocked event. Connection-management events are owner actions and may occur with sharing off.

Before dispatch, the backend commits a Started event and last-request timestamp. If that fails, no tool runs. After dispatch it records the outcome before responding. If outcome recording fails, the request returns an error warning that it may have run; Started remains as an uncertain outcome. Processed means server processing succeeded, **not** that the client received, understood or used a reply. Oversized replies and disconnected clients can have a Processed record. Current-unlock setup verification continues to require a successfully written, bounded response and remains separate from persistent history.

This is a useful local activity record, not a tamper-proof or complete forensic audit. A noisy connection can roll out its own older events. The global ceiling removes the oldest event from the busiest connection bucket first, breaking equal-size ties by oldest event; more than 1,000 distinct buckets cannot all be retained. Owner actions without a connection and all revoked registrations share one archived bucket, also limited to 100. Revocation can therefore trim archived history. Labels do not define buckets. The owner can clear records; backups preserve earlier history. No telemetry is sent anywhere.

## Clear and retention

**Clear activity…** requires acknowledgement and clears retained events, both numeric counters, last-request times and ephemeral request-seen evidence for the current vault session. It preserves credentials, scopes and paused states. The operation is atomic and rejects a stale unlock generation; new requests may immediately create new events. It does not erase older encrypted backups or guarantee physical erasure from every filesystem snapshot. Revocation preserves retained activity labels until normal rollover or explicit clearing.

## Storage compatibility

Schema **10** adds the singleton `agent_activity_totals` table and fair retention to schema 9's controls/history. Existing scopes, pauses, last-request times, approvals and encryption are preserved. The same migration transaction trims previous history to the new limits and counts removed rows; denied counts begin at zero because previous attempts cannot be reconstructed. Failed migration leaves the earlier schema and records intact. Schema 2–9 upgrade tests target 10; encrypted backups carry schema 10 and older builds reject them. Keep a verified pre-upgrade backup; never edit the schema number to bypass a version guard.

The UI shows **at least N** rejected credentials and **at least N** removed older events since this upgrade or the last clear. Each counter saturates at JavaScript's exact integer maximum (9,007,199,254,740,991). Storage stays one row regardless of traffic. Each counted denial updates that encrypted row; this is not request rate limiting. A storage failure still denies access and reports that counting failed. Event retention and its counter update are atomic with the event insertion. Numeric totals survive encrypted reopen/backup/restore; restored credentials remain revoked. No counter contains caller-controlled text.

## Verification

Backend checks cover all-tool pause enforcement, independent clients, scope-preserving resume, broker request-seen behavior, sanitized metadata, no invalid-token attribution, quiet-client protection against paused flooding, the total ceiling, saturated anonymous denial counts, migration trimming/rollback, retention-write rollback, counter backup/restore, bounded pagination, clear/revoke-all review/session rules, injected recording failures, global-sharing fallback, encrypted reopen/backup/restore and version-8 migration rollback. Frontend checks cover control labels, honest activity-result wording, counters without event rows, and single-revoke acknowledgement/cancel/targeting. Native acceptance is recorded separately in VALIDATION; passing automated checks alone is not native or independent security acceptance.
