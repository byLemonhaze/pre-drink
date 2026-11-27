# Resumable official-export imports

Imports remain local and require an unlocked vault. Supported official ChatGPT, Claude and Grok ZIPs use the same parser contracts and size limits as before. This feature needs no model, API, account or paid service.

## Owner workflow

1. Choose or drop an official ZIP. The app checks recognized JSON files and prepares conversations, showing the number safely checkpointed.
2. Use **Pause import**, lock the vault or quit. Committed preparation stays encrypted; the last unfinished batch is repeated later. No partial archive is searchable, shareable or accessible to agents.
3. Unlock/reopen. **Choose ZIP to resume** verifies the same recognized JSON contents, even if the file was renamed or moved. A different export is refused without discarding existing work. The ZIP must still exist until preparation is complete.
4. If all conversations were prepared, **Finish import** works without the original ZIP. Current privacy rules are rechecked before saving.
5. The final save updates conversations, raw snapshots, branches, source versions, approvals and FTS together. It removes the preparation in that same transaction. Existing privacy restrictions and publication invalidation rules still apply.
6. **Discard preparation…** asks for confirmation. It deletes only this unfinished import's checkpoints; existing archive data and the source ZIP are not deleted. Keep the original ZIP if you may want to start again.

Only one import can be prepared at a time. Multiple ZIPs selected together run sequentially; after interruption, later unstarted ZIPs must be selected again. Resume is explicit, never an automatic background task after unlock. A late error in a malformed export can leave harmless encrypted preparation; discard that preparation before importing a corrected export.

## Deleting archive data while an import is pending

Pause a running import before reviewing or deleting archive data. Deleting a conversation also discards the **entire unfinished import**, including already prepared records and the remaining resumable ZIP. The confirmation displays its filename and counts. Both changes commit atomically; an error preserves both. This prevents **Finish import** or **Resume** from silently restoring a deleted conversation, even when it was in the not-yet-prepared portion. Other published conversations stay unchanged.

Selecting the original ZIP again is a deliberate new import and can restore deleted conversations. There is no permanent deletion tombstone, secure-erasure promise or automatic cleanup of old backups. Snapshot-only pruning does not discard paused import preparation.

## Checkpoint contract

Schema 8 adds `import_job` (one row: fingerprint, display filename, total/saved counts, processed-message and skipped-message counts) and `import_items` (ordered normalized payloads with unique conversation IDs). Both live inside SQLCipher. There is no plaintext checkpoint database, key file, copied ZIP, saved absolute source path or network transfer. These tables are absent from search/MCP/context-pack queries. Backups include them; restoration/rotation never resumes work automatically.

The SHA-256 fingerprint covers a normalization-contract identifier, supported-file kind and length-delimited uncompressed recognized JSON bytes in archive order. Unrelated attachments are not fingerprinted or imported. Resume rereads/decompresses and decodes the ZIP to verify it, then skips normalization of the saved prefix. This is conversation-boundary resumption, not byte-offset decompression or streaming JSON recovery. Changing normalization semantics requires a new contract identifier. Future schema migrations must explicitly migrate or invalidate staged payloads when their shape or meaning changes; fully prepared checkpoints bypass ZIP fingerprint verification and therefore cannot rely on that identifier alone. Completed checkpoints still undergo current privacy checks at publication.

Preparation commits after 20 conversations or roughly 4 MiB of original conversation JSON, whichever comes first. A single large conversation can exceed that batch size. The cursor and payload batch commit together, so a failed write never advances the cursor. Duplicate IDs cannot cross checkpoint boundaries. Counts describe durable preparation, not an estimated time or final-save percentage. A request-scoped phase event reports checking/preparing/saving without filenames, paths or content. The interface accepts only its active request ticket; status polling does not extend the idle deadline.

A process-local run identity, cancellation flag and vault generation protect every checkpoint/publication. Locking cancels the old run; relocking/unlocking, expiry or replacement cannot make its result valid again. A second concurrent import is rejected. Reading checks cancellation between ZIP chunks/conversations; a single JSON decode or single-conversation normalization is not preempted halfway through. Final privacy checks run outside the session mutex, with cancellation checks between messages. The final database transaction holds the session mutex and must finish or roll back; a pause/lock arriving after it begins waits for it. This is not an instant cancel-at-every-instruction guarantee.

If the process dies before final commit, archive writes and checkpoint deletion roll back together. If commit succeeds before the interface receives its result, the archive is complete and no pending import remains; reopening the archive confirms the data. There is no separate durable completion receipt. A verified backup should precede a schema upgrade; v7 and older apps must not open schema 8. Encryption format/password derivation are unchanged.

## Verification and remaining limits

Local regressions cover persisted encrypted checkpoints and backup, mismatched/renamed ZIPs, skipped-prefix normalization and report counts, duplicates/malformed late records, discard, lock/reunlock/expiry/concurrent attempts, preparation and publication capacity failures with retry, current privacy rechecking, schema-7 migration preserving approvals, and a killed final transaction followed by retry. Frontend rendering tests cover paused/ready/running controls, escaped filenames, named progress and the non-cancellable save explanation. Native outcomes are recorded separately in VALIDATION.

This does not remove the 256 MiB recognized-JSON limit, bound every individual record, eliminate the final atomic transaction's memory/latency cost, or prove physical power-loss durability. Actual hardware/filesystem failures and all historical schemas remain separate coverage.

## Timestamp compatibility and maintained parsing

Claude/Grok RFC3339 date parsing uses `time` 0.3.55. The replacement preserves the previous millisecond flooring (including negative pre-epoch fractions), numeric offsets, accepted space/lowercase separators and leap-second-to-Unix representation. Mongo extended-JSON milliseconds remain unchanged. Missing/invalid timestamps remain absent rather than receiving an invented current time; source JSON retains the original value. The shared fictional adapter contract passed before and after replacement. This does not establish every real provider-export variant or remove the MCP SDK's separate Chrono dependency.
