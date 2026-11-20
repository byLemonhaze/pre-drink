# Native walkthrough with fictional data

This checks the actual macOS app, not a browser mock. Never use the personal vault or personal exports. The fixture generator creates a new private temporary directory and intentionally leaves it for the walkthrough. It never accepts an existing vault path.

## Prepare an isolated build

Build a debug app with a separate product name and identifier, for example `Pre-Drink Walkthrough` / `com.predrink.walkthrough`, using a temporary Tauri configuration override. Keep the window title visibly distinct. Do not modify the production identifier in source.

Run:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --locked --example native_walkthrough_fixture
```

The output identifies the generated `app_data_dir` and an explicitly fictional password. Launch the separate debug app with `PRE_DRINK_DATA_DIR` pointing to that generated directory. Confirm **Separate test storage** before proceeding. Release builds ignore this override. Keep generated databases, backup files, screenshots and configuration tokens out of Git.

The fixture contains two conversations/two messages, a timestamp falsely flagged by older rules, a genuinely sensitive fictional password message, a source-linked memory and brief with historical approvals, and an inactive original encrypted copy. The initial replacement and legacy flags are fixture setup, not native password-change evidence.

## Walkthrough and observed results — 2026-11-20

| Check                                     | Observed native result                                                                                                                                                                          |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Unlock isolated fixture                   | Separate test storage shown; restricted chats hidden initially.                                                                                                                                 |
| Preview flags                             | Two complete timestamp candidates, none selected, zero new flags, two approvals affected. The sensitive message is not offered for clearance.                                                   |
| Apply with no selection                   | Both candidates and historical approvals remain unchanged.                                                                                                                                      |
| Select conversation alone                 | Rejected with “Nothing was changed”; message clearance is required first.                                                                                                                       |
| Select timestamp message and conversation | Succeeds; one safe conversation becomes visible, the other remains sensitive, sharing stays off.                                                                                                |
| Approval reset                            | Memory shows Draft; dependent brief shows Source review needed.                                                                                                                                 |
| Cleanup without backup                    | Original copy size/date/file displayed; Review removal disabled with backup guidance.                                                                                                           |
| Native backup save                        | Native Save panel accepts an external-to-vault temporary path; app reports saved and verified, then clears password fields.                                                                     |
| Cleanup confirmation                      | Incomplete typed phrase keeps removal disabled; exact phrase plus checkbox enables it.                                                                                                          |
| Cleanup submission                        | Owner authorized final fictional deletion; UI reports removal, refreshed list shows no previous copies.                                                                                         |
| Native configuration                      | Codex and Grok Build settings show the isolated app/socket, memory-only scope, provider-specific instructions and sharing off. No real client configuration file is edited.                     |
| Sharing preview/copy                      | Selecting an approved memory plus draft brief yields one included/one excluded. Native copy succeeds; pasting into an unsaved memory editor matches outgoing Markdown. The editor is cancelled. |
| Full application restart                  | Quit while fictional sharing is on; relaunch starts locked. Unlock restores the archive with one safe/one sensitive conversation, both test registrations remain, and sharing is off.           |

A visual inspection found the previous-copy acknowledgement and typed phrase running together. The follow-up adjusts only their spacing/layout, with a separate confirmation field and wrapping action row. The rebuilt native app was inspected visually: the acknowledgement, phrase label/input and action row are separate and readable. A fixture-only inactive copy was staged from the fictional backup to reopen this review after the original copy had already been removed; a fresh native backup was verified, and the review was cancelled without deleting that staged copy. This setup is not password-change evidence.

## Owner-assisted password change

Computer-control rules require the owner to enter, confirm and submit a new credential. Use the fictional phrases only. After completion, verify the old phrase fails for the active vault, the new phrase opens it, data remains, registrations are removed, sharing is off and the previous encrypted generation is listed. This is distinct from automated replacement tests.

**Result in this session: unverified.** The reported handoff completion could not be confirmed: the existing fictional password still unlocked the active vault and the selected generation had not changed. No completed native replacement was observed. The credential-entry handoff was not completed during the follow-up. This establishes neither a successful password change nor an application defect.

## Physical file drop

Generate local fictional ZIPs with `scripts/create_import_fixtures.py --output <new temporary folder>`. Drag the valid ZIP from Finder into the unlocked isolated app, verify imported counts and omissions, and repeat for idempotence. A picker import or programmatic backend call does not count as physical drag-and-drop evidence.

**Result in this session: inconclusive.** Two computer-control attempts to drag the selected valid ZIP from Finder into the app produced no import or count change. The control gesture was not established as a successful cross-window file drop; this is not a passed gate or a diagnosed app defect. The temporary Finder window was closed.

## Finish

Revoke remaining fictional client registrations, lock and quit the separate app. Retain or remove only the explicitly generated temporary fixture as appropriate; never infer cleanup targets from the personal app folder. Keep actual observations and outstanding limitations in VALIDATION and WORK_LOG. Apple signing, representative personal-export checks and public release are separate gates.

**Observed finish:** both fictional client registrations were revoked in the app, sharing remained off, the lock screen appeared, and the separate app quit. Temporary fictional storage and backups remain outside the repository. The personal vault was not opened or modified.
