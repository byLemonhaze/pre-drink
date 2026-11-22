# Release readiness — 2026-11-22

Pre-Drink is usable as an early personal tool. It is not independently security-audited or ready for an unrestricted public download. Keep original exports and verified encrypted backups while using it. A passing test count is evidence for particular behaviors, not a safety certification.

## A bounded path to a limited beta

The working estimate is two to three focused rounds, excluding Apple distribution. Findings can add work; this is not a calendar guarantee. Avoid expanding the provider list or redesigning the product during these rounds.

1. **Boundary and recovery pass.** Review schema-8 staging, backup/restore, password rotation, provider isolation and shared publication. Exercise combinations introduced by resumable imports, not just each feature alone. Fix confirmed defects and record remaining limits. The current local pass adds checkpoint recovery and provider-isolation regressions; an independent reviewer is still needed for the broader security gate.
2. **Native acceptance.** Finish human screen-reader listening, representative-data upgrade/lifecycle checks and the remaining saving-state observation using isolated storage and backups. Clearly distinguish automated rules, native observations and tests requiring the owner. Do not repeat destructive checks on personal data.
3. **Fresh-user rehearsal and release reconciliation.** Walk through the packaged app's first launch, restore, official import, memory review, exact sharing preview, client setup and revocation using fictional storage. Check that the quickstart and recovery guide match the shipped build. Verify the private source-only repository and local build identity. Resolve blocking review findings before inviting other users.

Apple enrollment, Developer ID signing, notarization and a downloaded clean-install check remain last. Public source/download publication requires the owner's launch decision. No paid services, model calls or GitHub Actions runs are needed for the local rounds above.

## Current evidence and limits

| Area                    | Evidence available                                                                                  | Still distinct from that evidence                                                         |
| ----------------------- | --------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| ChatGPT import          | Representative export backend acceptance and native post-import use; fictional interruption/resume  | Broader representative upgrade/lifecycle acceptance and all future export formats         |
| Grok and Claude imports | Versioned fictional parser contracts and mixed-provider isolation                                   | Representative exports from each provider; avoid broad compatibility claims until checked |
| Agent curation          | Backend authorization/revision/publication tests; bounded fictional Codex and Grok client runs      | Broad semantic quality, Claude client acceptance and unrelated web-chat integrations      |
| Recovery                | Encrypted backup, wiped-storage restore, rotation, stale-operation rejection and simulated failures | Physical power loss, full disks and every historical schema/filesystem                    |
| Accessibility           | Keyboard/focus/native layout observations and measured theme contrast targets                       | Human VoiceOver speech/announcement review and full accessibility conformance             |
| Distribution            | Locally built/ad-hoc-signed app and private source repository                                       | Apple signing/notarization and downloaded clean-install trust                             |

The current backend pass is a self-review plus regression testing. It must not be described as an independent audit. The detailed evidence is in VALIDATION and NATIVE_WALKTHROUGH; IMPORTS records final-save, memory and checkpoint-format limitations.

## Latest review triage — Nov 22

The supplied review at `fcdcf0f` identified a confirmed deletion/resume defect and a newly patched dependency advisory. Those took priority over more walkthrough work. Conversation deletion now atomically discards any pending import, with explicit review wording and running-import exclusion. The patched `url` dependency and advisory gate are documented in [DEPENDENCY_CHECKS](DEPENDENCY_CHECKS.md). The review is useful external evidence about that older snapshot; it is not independent approval of the new fix. Native acceptance and a fresh-source review remain required. Apple distribution remains deferred.
