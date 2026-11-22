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

**Result after owner retry on 2026-11-20: passed for the successful native path.** The owner entered and submitted the change, then reopened with the new fictional phrase. Independent native verification locked the vault, confirmed that the previous phrase is rejected, and confirmed that the new phrase opens it. The active generation changed; both conversations/messages, the approved memory, the draft brief and the sensitive restriction remain. Sharing is off, no clients are listed, and the previous active generation appears among retained recovery copies. The app was left locked.

The first incomplete handoff did not establish a result; this later verification supersedes that status. The two fictional clients had already been revoked before this rotation, so an empty client list does not independently prove removal of populated registrations during this native run. That behavior retains its automated coverage. Native failure paths and physical interruption testing remain separate.

## Physical file drop

Generate local fictional ZIPs with `scripts/create_import_fixtures.py --output <new temporary folder>`. Drag the valid ZIP from Finder into the unlocked isolated app, verify imported counts and omissions, and repeat for idempotence. A picker import or programmatic backend call does not count as physical drag-and-drop evidence.

**Result after owner-assisted drops on 2026-11-20: passed.** The owner physically dragged the selected fictional ZIP from Finder into the isolated app. Native inspection reports 2 new conversations, 0 refreshed and 6 text messages processed; totals change from 2 conversations/2 messages to 4 conversations/8 messages. Import notes disclose 7 hidden, tool, system or non-text messages omitted from the reader. The reading-nook conversation displays its current reply and retains alternate replies/history; searching for an alternate reply term returns that conversation. The pre-existing sensitive conversation remains hidden and sharing stays off.

The owner dropped the same ZIP a second time. Native inspection reports **0 new, 2 refreshed, 6 text messages processed**, with totals unchanged at **4 conversations/8 messages**. No duplicate conversations or messages were added. The app was left locked. These owner gestures and observed results supersede the earlier inconclusive automated drag attempts. This checks a valid fictional ZIP and repeat import; it does not establish physical-drop behavior for every malformed/oversized export or representative personal exports.

## Finish

Revoke remaining fictional client registrations, lock and quit the separate app. Retain or remove only the explicitly generated temporary fixture as appropriate; never infer cleanup targets from the personal app folder. Keep actual observations and outstanding limitations in VALIDATION and WORK_LOG. Apple signing, representative personal-export checks and public release are separate gates.

**Observed finish:** both fictional client registrations were revoked in the app, sharing remained off, the lock screen appeared, and the separate app quit. Temporary fictional storage and backups remain outside the repository. The personal vault was not opened or modified.

## Failure-path follow-up — 2026-11-20

Using the same isolated fictional vault (four conversations/eight messages):

- Native picker import of the malformed later-JSON ZIP reports invalid conversation JSON and no import. Counts remain unchanged.
- A 138-byte synthetic ZIP advertises an uncompressed `conversations.json` entry of 256 MiB plus one byte, matching the existing oversized-entry regression method. Native picker import reports the 256 MB limit; counts remain unchanged. No huge payload is allocated. This is picker evidence, not a malformed physical-drop check.
- A reviewed 157-byte fictional Markdown export is cancelled in the native Save panel after naming the destination. No destination file is created; the preview and enabled controls return without a success claim.
- A separate copy of the fictional encrypted backup, truncated by 17 bytes, is rejected by native inspection. The password field clears, no replacement review is offered, and the active generation/counts remain unchanged. The intact source backup is not modified.
- Retrying the intact backup reaches a verified preview (two conversations/two messages, one memory/brief, two historical client registrations to remove). Quitting the app before confirmation and relaunching leaves the original active generation selected, starts locked, accepts the existing current password and shows all four conversations/eight messages with sensitivity retained and sharing off. This is ordinary quit before confirmation, not a simulated power failure during publication.

The owner locked the Mac with Control–Command–Q for about ten seconds and unlocked it while the reviewed Save panel was open and fictional sharing enabled (no registered clients). **Baseline failure:** sharing remained on, the local broker still reached client authentication rather than its sharing-off gate, and saving the pre-lock preview succeeded. The app was immediately locked manually. This reproduces the previously open screen-lock gap on this Mac, not a bypass of vault-password authentication.

The fix adds a one-way screen-lock observer while retaining sleep/session/idle controls. Its process-local native-notification regression checks that locking closes the session, switches sharing off and rejects the pre-lock export; unlock signals do not unlock the vault. This test does not broadcast simulated system notifications to other apps. **Rebuilt-app retest passed:** the owner repeated Control–Command–Q with sharing on and a new reviewed Save panel open. Before saving, the local broker rejected the probe at its sharing-off gate and the destination was absent. Clicking Save returned to the vault unlock screen and still created no file. Unlocking restored four conversations/eight messages, one hidden sensitive conversation and sharing off.

**Separate sleep/wake check passed:** with sharing enabled again, the owner selected Apple menu → Sleep, waited about ten seconds, then woke and unlocked the Mac. Pre-Drink showed its vault unlock screen; the local broker returned sharing off. Unlocking preserved four conversations/eight messages and the sensitive restriction, with sharing still off. The test app was then locked and quit. These observations verify this host's owner-performed sleep/wake cycle, not which individual notification fired, every supported OS/hardware configuration or physical power-loss durability. No clients were registered during these checks; the broker probe establishes the sharing gate, not an authenticated client retrieval test.

## Large-library and keyboard follow-up — 2026-11-20

`cargo run --manifest-path src-tauri/Cargo.toml --locked --example native_scale_fixture` creates a new private temporary directory with an encrypted fictional library: 5,000 conversations and 100,000 short text messages. It never reads personal exports or normal app storage. Launch the separate debug app with the emitted `app_data_dir` as `PRE_DRINK_DATA_DIR`; verify **Separate test storage**. This normalized-data generator is not ZIP parser or representative-export evidence.

Native observations: the library opens with the expected totals and sharing off, initially shows 100 rows, and Load more appends a second page (200+ shown, including the 4900 → 4899 boundary). Searching `gardenmarker0042` finds project 0042 beyond those pages and displays its 20 messages. A no-match query clears the reader; replacing it restores the correct result. Saving a selected message as an edited memory by keyboard produces a draft with its source link. These are functional responsiveness observations in an unoptimized build, not measured frame times or a latency guarantee; long individual chats and large memory/brief collections remain separate.

The baseline memory editor left keyboard focus on archive search; Tab reached the background Clear search control and Escape did not dismiss the editor. Replaced the four simulated modal overlays with a shared WebView modal dialog. Background controls become inert, focus starts inside, Escape cancels, and closing restores prior focus where it still exists. Dismissal is disabled during a pending save, approval or deletion. An explicit Tab boundary handles WebKit's document-focus stop.

Initial native retest: all four dialogs hide the background from the accessible dialog tree. Memory and brief editors focus their content/title, deletion review focuses its confirmation field, and approval review focuses Close. Escape leaves a draft unapproved, a new brief unsaved, and archive totals unchanged after cancelling deletion. A keyboard-opened memory editor returns focus to its originating Keep as memory control. **Final native retest passed:** forward Tab wraps from Save memory to Close memory editor; Shift-Tab wraps back to Save memory. Escape cancels without changing the saved draft. A temporary build configured at the supported minimum 980 × 640 window size was inspected using the earlier small fictional fixture: both memory and wide brief dialogs stay within the viewport, scroll internally, and keyboard navigation brings their action buttons into view. Submitting an empty brief focuses Title with “Fill out this field”; Escape closes it without adding a brief. A preceding border-drag gesture did not resize the window and is not counted as resize evidence. Both fictional sessions were locked and quit; no personal storage was used.

## Long-chat and large curation library — 2026-11-20

`examples/native_reading_fixture.rs` creates a new private temporary directory containing an encrypted fictional vault: one 2,000-message conversation (1,999 active and one alternate), 3,000 memories (one private; 100 approved), and 500 briefs (one private). It never opens default app storage or reads exports. Launch the separately named debug app with its printed data-directory override.

Native checks passed at 1280 × 840:

- Reader begins at 1–20 of 1,999 active messages. Last reaches 1981–1999; searching the late message finds it without traversing preceding pages.
- Alternate-branch search gives zero results until all branches are explicitly enabled. A search term beyond a long message's preview still finds the full source. The 20,845-character fixture expands to rendered Markdown including its final marker, then collapses again.
- Memory begins at 1–25 of 2,999 visible notes. Last reaches 2976–2999; searching finds the oldest approved note. The private note gives zero results until restricted review is enabled, then remains visibly Restricted.
- Briefs shows 1–25 of 499 visible items, and search finds the oldest brief. The editor explains its 50-source option limit; searching finds an approved note outside that initial set, and selecting it inserts the complete note with one retained source link. Cancelling leaves the draft unsaved.
- The first walkthrough reproduced macOS autocorrect changing an exact search token when focus left the field. Search inputs now disable correction/capitalization/spelling; the rebuilt app preserves the same token after clicking the branch toggle and finds the intended alternate message.
- Last-page navigation initially lost focus when its button became disabled. The rebuilt controls focus the result range; Tab then reaches First.

Both native sessions were locked and quit, with agent sharing off. No personal vault, export, client configuration or password was used. No screenshots or generated vault data are committed.

These observations establish interactive behavior at this fictional size, not a latency/RAM benchmark or a maximum-size guarantee. Paging bounds mounted cards/Markdown trees; desktop commands still load full result collections. Larger IPC payloads, maximum-size Markdown, minimum-window layout for these new controls and representative official exports remain separate checks.

## Provider-private curation walkthrough — 2026-11-20

Used a separately identified debug preview and fresh encrypted fictional storage generated by `examples/curation_walkthrough_fixture.rs`. No personal vault/export or model provider was accessed. Both automatic-publication policies stayed off.

- Reviewed complete ChatGPT project/fact summaries together and released the batch; both displayed owner-reviewed status.
- Edited the project into a new private revision, inspected prior/current text and saved earlier text as a new held revision. No old release silently returned.
- Switched to the separate Grok workspace, inspected its fictional session note and withdrew it. History remained while editing/re-release was disabled.
- Shared topic preview included only the remaining eligible reviewed fact, with provider/revision/session/review-basis attribution. It excluded private, held and withdrawn drafts.
- Rebuilt after correcting modal placement and asynchronous opener focus. History opened a contained editor; Escape cancelled without mutation and returned keyboard focus to the History button.
- Locked the preview; the password screen was visible and agent sharing remained off.

This verifies owner UI paths on the tested Mac, not model reasoning, arbitrary-client connectivity, native automatic-policy enablement or representative real-export processing. Protocol writing and access-boundary regressions have separate evidence in VALIDATION.

## Grok/Claude import and schema-5 upgrade — 2026-11-20

The existing isolated encrypted curation preview was rebuilt and reopened after the provider migration. Its fictional prior ChatGPT contributions retained revision/status information and its Codex/Grok registrations retained provider scopes. Sharing remained off.

- Imported the generated Grok ZIP through the native picker: one new conversation, three text messages. Confirmed Grok labels, alternate limestone/current sandstone replies, and the explicit latest-leaf fallback warning.
- Imported the generated Claude ZIP: one new conversation, two text messages. Confirmed Claude labels, cedar/reading-bench text, and omission of thinking/tool content from the reader.
- Selected both ZIPs in one picker operation and reimported: zero new, two refreshed, five text messages; library totals remained two conversations/five messages.
- Searched cedar and observed exactly the Claude conversation.
- Opened Agent access and observed prior ChatGPT project revision 3 withheld and fact revision 1 owner-reviewed. Switched to Claude's empty private workspace; inspected Claude (local MCP) and its Claude-only archive wording without creating a client or enabling sharing.
- Locked the app and observed the password screen.

Fixtures are generated by `python3 scripts/create_provider_exports.py` from invented provider/version inputs. Generated artifacts stay ignored. This is picker/native acceptance, not a new physical Finder-drop check, real-export check or model-provider call. The test used no personal vault or credentials.

## Oversized-message parts — 2026-11-20

Built a separate debug Reading Preview with a 980×640 window and a fresh encrypted fixture from `examples/native_large_message_fixture.rs`. One invented message contains 550,133 UTF-16 code units (540,132 Unicode scalar values), including emoji, Markdown-like text, a final marker and literal script-like text. No personal storage was opened.

- Unlock succeeded using the fixture phrase. An initial automated typed entry was incomplete and received the generic wrong-password error; pasting the exact existing fixture phrase succeeded. No password reset or data change occurred.
- The message started as the bounded preview. Expanding showed **Part 1 of 35** and explicit plain-text treatment.
- **Last part** reached part 35 and moved keyboard focus to the part status. Tabbing through available controls brought the final marker and literal `<script>` text into view; the text remained inert.
- Keyboard Previous, First and Next moved 35 → 34 → 1 → 2 with focus returning to the part status. End controls were disabled at their boundaries.
- Collapse restored the preview. Searching for the final marker still found the complete message; re-expanding reset to part one.
- Controls remained visible in light and dark modes at the configured minimum window. Light mode was restored, the vault was locked, its contents disappeared, and the lock screen fit without outer scrolling. The preview was then quit and its newly generated fictional directory removed.

This is native functional evidence for one oversized message, not a measured frame-time benchmark, screen-reader certification or full-message IPC memory bound. No client registration or sharing was enabled during this walkthrough.

## Guided setup, paged reader and contained choices — 2026-11-20

Used newly generated fictional encrypted storage from `native_reading_fixture`, with 2,000 messages, 3,000 memories and 500 briefs, in the separate Reading Preview at 980×640. No personal vault, real export, model account or external client settings were opened or changed.

- The new backend-paged reader showed 1–20 of 1,999 active messages. Last reached 1981–1999 with focus on the range. Searching an alternate-only marker returned zero until branches were explicitly included, then returned the one retained message. Full-source search found a marker beyond the collapsed 20,845-character message preview.
- Guided setup defaulted to shared-only and disabled creation until acknowledgement. Keyboard End/Enter changed access and cleared acknowledgement. Changing the application via type-ahead reset access to shared-only and cleared acknowledgement. Escape cancelled a highlighted choice without selecting it or closing the enclosing modal; Tab left an open chooser for the next field. Pointer selection of Claude closed its chooser and updated its label.
- Created one fictional shared-only Grok-format registration. Private TOML settings and their destination instructions appeared; credential text was redacted from tool output and no configuration screenshot was taken. Verification initially disabled the request check while sharing was off. After explicit sharing enable, the check correctly reported no successful request. Back retained the settings during setup; closing hid them. Reopening reset the wizard. The registration was revoked afterward. Positive request evidence and reset-on-lock are covered by the actual compiled stdio/backend tests; this native check did not install a configuration in a model client or exercise its positive wizard result.
- The owner reported a detached native select menu remaining over Chrome after minimizing. All app selects now use an in-window listbox. Opening options then minimizing removed the listbox from the app accessibility state; it stayed absent when the app window was inspected again. Background Chrome inspection timed out, and automatic approval review refused a full Finder screenshot because it would expose unrelated private names. Therefore no full-desktop visual comparison is claimed. The replacement list is structurally inside the WebView, with no native popup to remain above another app. Keyboard/pointer checks passed after restarting the isolated preview.
- Inspected light and dark setup/choice screens at minimum window size. Expanded choices scroll within the modal; the initial setup and verification actions remain reachable. Selected values appear in accessibility labels. This is focused functional evidence, not screen-reader certification or a comprehensive accessibility audit.

Returned the preview to light mode, locked it, quit it and confirmed the process was no longer running. No screenshots, credentials or fictional vault files are committed.

## Owner holds and work sessions — 2026-11-20

Used a freshly generated disposable fictional vault and the separately named **Pre-Drink — Reading Preview** debug bundle, at its 980×640 minimum window. No personal storage, installed model calls, generated client credentials or GitHub builds were used.

- One-hour session enabled sharing and displayed approximately 60 minutes remaining. Ending it showed sharing off and restored the start control.
- Keyboard selection reached **Until my Mac locks**. Starting showed the corresponding active state; manual Lock vault returned to the locked screen. Unlocking showed sharing off; the session was not resumed. Full app quit/relaunch likewise started locked with sharing off after unlock.
- The fixture's held contribution displayed **Held by you**, with selection/review disabled. The editor showed the checked owner-hold control. Explicitly clearing and saving produced revision 3 as a **Private draft**, not a publication.
- The walkthrough discovered a narrow unstyled curation textarea. A dedicated full-width field was added and the rebuilt native modal was visually checked in light and dark modes at minimum window size. Its label, content, hold control and action buttons remained visible.
- Finished with sharing off, vault locked and the preview process confirmed stopped. Temporary fictional data was kept outside Git.

This did not wait a real hour/four hours or repeat owner-assisted physical Mac lock/sleep. Deadline/idle transitions are covered by controlled-clock regressions; the existing native notification test now starts an until-lock work session before exercising its callback. Prior physical Mac lock/sleep observations remain separate historical evidence. The new privacy-on-open count is backend-tested; a native changed-rules upgrade notice is still an acceptance case for a future isolated fixture.

## Backend library paging — 2026-11-21

A rebuilt isolated Reading Preview used fresh fictional encrypted storage containing 2,000 messages, 3,000 memories and 500 briefs. The personal vault was not opened. At the 980×640 minimum window:

- Memory displayed 1–25 of 2,999 unrestricted notes. Last reached 2,976–2,999, disabled Next/Last and moved focus to the range. Searching the oldest marker returned the complete approved note and reset the range to 1–1.
- A private-note search returned zero until the owner enabled restricted viewing, then displayed one Restricted item. No approval or sharing permission was changed.
- Briefs displayed 1–25 of 499 unrestricted briefs. A body-text search found the oldest brief beyond the initial page.
- The new-brief source picker displayed 25 of 100 approved sources, reached the last page without submitting the form, and found the oldest source. Adding it returned focus to search, removed it from available results, increased the linked-source count and appended its complete text to the editor. The unsaved fictional editor was closed.
- Sharing displayed 25 of 3,500 choices. An approved note stayed selected when searching for an unapproved brief. Selecting both produced exactly one included and one excluded item; the complete preview contained the note only and explained the brief needed approval. Copy/save remained disabled without the review checkbox. Nothing was copied, saved or sent to a client.
- App-only screenshots checked the light brief editor and dark sharing screen; appearance was returned to light. Manual lock removed the unlocked interface; app shutdown was verified and only this run’s temporary fixture was removed. No desktop-wide screenshot was taken or committed.

This verifies the observed paths, not VoiceOver, every keyboard/error/race case, peak memory, sustained query latency or every macOS version. Rust tests separately cover late Unicode matches, privacy/eligibility, deleted-page clamping and response size. No physical Mac lock/sleep cycle was repeated in this batch.

## Representative ChatGPT post-import check — 2026-11-21

An optimized, locally ad-hoc-signed app bundle used a fresh identifier and its own previously unused app-data namespace. The ordinary Pre-Drink bundle identifier and storage were not used. This release build did not enable the debug-only data-directory override. The separate directory was verified owner-only before the owner created a vault. The owner performed password creation and import before automation resumed; initial file-picker/drop and import-progress behavior were not observed in this session.

After reconnecting at the owner's locked screen, automation used existing credentials only for unlock. No new credentials were entered by the agent. Unlocked accessibility snapshots were filtered inside the local automation runtime before returning only known controls, states and aggregate counts; no unlocked screenshots or chat text were returned or committed. Cached automation state was discarded afterward.

Observed outcomes:

- Unlock returned to the populated archive with the expected total and sharing off.
- A positive literal search returned results. A unique nonexistent term showed the empty-results notice and no chat rows; clearing it restored the list.
- The local restricted-content checkbox could be enabled, and the archive loaded beyond its initial page. The list's displayed loaded count confirmed additional results after loading/navigating. Automation exposed only part of the long screen at a time, so visible accessibility-node counts were not treated as total result counts.
- A message range was exposed in the reader. Long-message expansion, every branch and message-page navigation were not retested on personal content; earlier fictional evidence remains separate.
- Memory and Briefs were empty. Selected sharing showed no items, no selections and a disabled preview button. Import had not generated or published memory, and agent sharing remained off.
- The local restricted-content checkbox was restored to off, search was cleared and the archive returned. Manual lock restored the password gate and removed archive navigation.

The owner-created imported vault was retained and left locked in the separate app. It was not deleted or moved into the normal app. Its database had a non-plaintext SQLite header and owner-only file/directory permissions; this header sanity check is not an independent cryptographic audit. Earlier backend checks separately verified encrypted integrity, complete normalized-message preservation, backup/open and reimport. Automation connection failures before this successful run are not app startup-time measurements. No real archive curation, source grants, outgoing copies, provider/model calls or paid Actions were performed.

Remaining representative-data coverage includes the initial picker/drop/progress flow, interruption/cancel behavior, full long-message/branch interaction, old-vault upgrades and broader lifecycle/recovery UI. Do not infer these from the post-import checks.

## Native recovery and keyboard continuity — 2026-11-22

Used a separately identified **Fictional Recovery** debug preview at 980×640 with fresh encrypted storage from `examples/native_recovery_fixture.rs`. The fixture has a valid standalone backup, a truncated encrypted backup, an approved baseline memory and a newer current-only decision. Fictional malformed/valid ChatGPT ZIPs exercise retry. The generator now supplies all these inputs in a new private temporary folder; it never reads normal app storage. No personal archive, model account or paid service was accessed.

Observed before and after the fixes:

- A wrong unlock password cleared the field and reported that nothing changed; the correct existing phrase reopened the archive. The original interface dropped keyboard focus to the document on failure. The rebuilt interface returned focus to the empty password field and associated its error text with that field.
- Cancelling the native import picker preserved the archive. A malformed ZIP reported invalid conversation JSON and imported nothing. Selecting the valid ZIP afterward cleared the error, reported one new conversation and displayed the imported fictional conversation.
- The lock-screen restore panel trapped forward/reverse Tab and Escape cancelled it. Pointer-opened panels originally returned focus to the document. Explicit opener references fixed Restore backup and Vault details; both were retested successfully.
- A truncated backup and a wrong password for the valid backup were rejected. A correct-password retry reached a verified replacement review. Escaping that review preserved the current-only decision and newly imported conversation; reopening the original vault confirmed both.
- In the rebuilt flow, choosing a backup focused its password field; password errors returned focus there; preparation focused the unchecked review checkbox; keyboard Cancel returned to the password field. No checkbox was selected automatically. Restore remained disabled until explicit acknowledgement.
- Confirming the fictional restore returned to the locked screen with the backup-password instruction and preserved-previous-copy notice. Unlocking with the backup phrase showed the baseline conversation and approved memory only. The newer decision/import were absent from the selected restored version. The previous encrypted file remained on disk. After quitting from an unlocked state and relaunching, the app started locked, rejected the old current-vault phrase and accepted the backup phrase. Sharing was off before and after restart; it was never enabled in this batch.
- App-only screenshots showed the replacement review and the longer post-restore notice fitting at minimum window size. Light and dark lock-screen notices were inspected, and light mode was restored. Native Zoom enlarged the workspace and toggled back without losing the active memory view.
- Opening the Memory type list and minimizing removed its options from the accessibility tree; raising the window left it closed. This is app-level dismissal evidence, not a desktop-wide screenshot of unrelated applications.
- The same pointer-focus issue was reproduced in New memory. Main-workspace button clicks now establish focus before opening a modal. Retests passed Escape return for New memory, message-to-memory editing, saved-memory review, New brief and agent setup. Archive deletion review uses an explicit opener because its preparation is asynchronous; cancelling returned to Delete conversation without deletion. In a memory editor, the first Escape closed only its open choice list and the next Escape cancelled the editor. Existing notes/approvals remained unchanged.
- Agent setup remained shared-only, acknowledgement unchecked and creation disabled. It was cancelled without registering a client. No memory was published, copied or sent to a model.

Finished with light mode, sharing off, the vault locked and the preview process confirmed stopped. Only the newly generated fictional directories were removed after shutdown. No screenshots, generated ZIPs/vaults, passwords or private diagnostics are committed; the example contains explicitly fictional fixture phrases only.

These are native observations on this Mac, not VoiceOver certification, physical power-loss tests or acceptance of every platform. No new password creation/change or permanent-delete submission was repeated. Long-running native import progress/interruption, a changed-rules upgrade notice, broader accessibility/OS coverage and representative-data lifecycle checks remain open. Earlier physical sleep/screen-lock evidence and backend injected-failure checks remain separate. Automation's slow first attachment is not an app-startup measurement.

## Long import, upgrade notices and keyboard checks — 2026-11-22

Used a separately identified native debug preview at 980×640 and fresh encrypted fictional storage from `examples/native_upgrade_fixture.rs`. This generator creates an older-classifier/schema-6 fixture with a linked approved memory and brief, one unrelated approved memory, a schema-newer-than-app copy, and a 5,000-conversation/100,000-message official-format ChatGPT ZIP. It never reads owner storage. The old-classifier state is deliberately constructed in fixture SQL; it is not an independently produced historical app archive. Its pre-open assertion checks all three historical approvals. The first setup attempt triggered the current source-edit invalidation rule; it was corrected and the native upgrade was rerun on a fresh fixture.

### Long import interruption

- The native picker started the large fictional ZIP; the app showed its active import indicator and disabled the import button.
- Clicking Lock vault while the import was active immediately removed the unlocked interface. Reopening showed the original single conversation, no partial import and sharing off. After background parsing settled, no late import appeared.
- Retrying the same ZIP without interruption reported 5,000 new conversations and 100,000 processed text messages. Searching the final fictional conversation returned its complete 20-message range.
- A separate fresh fixture was quit with Command-Q while the active import indicator was visible. Process exit was confirmed. Restart began locked; unlock preserved the original conversation and the newly enforced privacy state, with sharing off and no partial import. The upgrade notice did not repeat once its changes had been applied. Retrying after this restart again completed all 5,000 conversations/100,000 messages; the preview was then locked and quit.

These actions exercise interruption during an active native import, not a precisely instrumented transaction-commit boundary. The earlier killed-import regression covers the production transaction immediately before commit. The interface provides an activity indicator, not a percentage, resumable checkpoint or dedicated Cancel command. Parsing may continue in the background after locking; generation validation prevents its stale result from being saved. A lock request arriving after the database transaction has acquired the session mutex waits for that transaction; do not claim instantaneous cancellation at every stage.

### Upgrade notice and approvals

- Opening the corrected fictional schema-6/older-classifier vault displayed exactly two added restrictions and two linked approvals returned to draft. The unrelated project remained approved.
- Newly restricted history was absent from the default archive view. The linked memory appeared only with restricted-note review enabled; its sharing review explained the restriction and disabled approval. The linked brief was also excluded from the default brief list in the initial pass.
- A separate fixture with a schema newer than this app was refused with “This vault needs a newer Pre-Drink version.” The password field was cleared and focused; keyboard navigation still opened Vault details and Escape returned to its opener. After quit, the encrypted database hash was unchanged; only the normal instance-lock file had been added.
- The full notice fit the minimum window in light and dark themes. No privacy flag was cleared, item published, client registered or cloud service contacted.

### Keyboard and visual accessibility

- Tab from the newly unlocked document reached navigation; Return opened Memory. Choice lists handled End, Home, Up/Down, Return and letter type-ahead, returning focus to their trigger. Tab continued into search; Space toggled the focused restricted-note checkbox.
- A keyboard-opened blocked sharing review exposed its reason and disabled approval. Forward/reverse Tab remained on its only enabled action; Escape returned to its opener.
- Light-mode keyboard outlines previously used bright decorative orange, with about 2.36:1 contrast against the cream canvas. They now use the existing darker orange text token (about 5.46:1 against that canvas); dark mode retains bright orange (about 6.47:1 against its surface). App-only visual inspection confirmed visible focus on the review Close button and dark-mode Lock vault button.
- Navigation now marks the current page with `aria-current`. Import status wording covers the entire operation rather than describing the saving stage as reading. No fake progress percentage was added.

These observations and token contrast calculations are not a complete accessibility conformance audit or a VoiceOver listening test. No global macOS accessibility settings were changed. Broader assistive-technology/OS coverage, historical schema families and representative-data lifecycle checks remain separate gates. Screenshots, generated exports, encrypted fixtures and local paths remain outside Git.

The fictional preview was left locked, returned to light mode and quit. Process exit was confirmed before removing only the two freshly generated fictional directories. The separate owner-created vault was not opened, stopped or changed.

## 2026-11-22 — Encrypted resumable imports and accessibility

All inputs and storage in this pass were newly generated fictional data. No personal vault or export was opened, and agent sharing stayed off. The debug app used a separate identifier and temporary encrypted storage; the ordinary app was untouched.

- A 5,000-conversation/100,000-message ZIP showed named durable progress. An early pause on a repeat import stopped at 2,980 conversations; focus moved to **Choose ZIP to resume**. The already published archive remained intact.
- Quit/relaunch/unlock retained exactly that checkpoint without automatically resuming. Selecting a different fictional ZIP produced a mismatch error without losing progress. Renaming the matching ZIP still resumed from the saved prefix; progress advanced to 4,660 and completion reported zero new / 5,000 refreshed, with no duplicates.
- Two late pause attempts reached the final publication boundary and completed instead of pausing. Final publication is atomic and cannot be interrupted once begun. This exposed delayed phase feedback while status polling waits for the database; request-scoped content-free phase events now update the interface independently, and rendering tests assert Pause is disabled during saving. The rebuilt long import completed successfully, but the brief saving-only state was not captured in the native observation; do not claim that particular visual assertion passed natively.
- A separately constructed fully prepared encrypted fixture displayed **Finish import**, one of one prepared and an empty archive. Clicking it completed with one new conversation/one message without selecting or retaining a source ZIP. This checks recovery UI/command wiring; backend regressions separately generate ready checkpoints through real preparation.
- Keyboard checks covered **Skip to workspace**, reaching Import without traversing the sidebar, pause-to-resume focus, discard confirmation's safe default, Shift-Tab containment and Escape returning to its opener. The native discard was cancelled; actual discard persistence is covered by backend tests. Minimum-window light layout showed readable progress and controls at 980×640.
- With explicit owner permission, macOS VoiceOver was briefly switched from off to on. Its control connection timed out and no audible output was captured, so spoken labels/announcements are **not verified**. The switch was confirmed off again and System Settings was returned to its original General pane. No other global accessibility preferences changed.

The fictional app was locked and quit, with process shutdown confirmed. No new model calls, paid services, remote workflows, password changes or permanent native deletions were performed. Physical power loss, all historical schemas, representative-data lifecycle and a complete human screen-reader pass remain outside this evidence.

## 2026-11-22 — Control contrast and lock explanation

Generated fresh fictional recovery storage and opened only the isolated debug app. At 980×640, inspected the light lock-screen password boundary/focus, light agent-session dropdown boundary, dark expanded dropdown with keyboard Escape returning focus, and rebuilt sharing-switch boundaries in both themes. Verified that the revised session explanation visibly states the final-import-save delay before lock completion. No agent session or connection was enabled. App-only screenshots were inspected transiently and not stored in the repository.

Restored the fictional preview to light appearance, locked it and confirmed process shutdown. No global accessibility setting or personal vault was changed. This pass does not verify VoiceOver speech, the transient Saving state, all dialogs or full accessibility conformance.

## Nov 22 review fixes and first-launch recovery rehearsal

Used the rebuilt debug preview with newly generated owner-only fictional storage. The recovery fixture now includes an already-published conversation also present in fully prepared staging, plus an unused first-launch directory. No personal archive or normal app namespace was opened.

- Opened deletion review while that checkpoint was ready. The full-import consequence, display filename/counts and explicit-new-import limitation were present. Typed confirmation started focused and deletion disabled. Light/dark layouts fit the compact window. Escape cancelled, returned focus to the opener and preserved both the archive and pending import. Permanent deletion was covered by backend regressions, not submitted in this native pass.
- Launched into unused storage and observed the create-vault screen. Chose **Restore backup**, selected the fictional verified backup, authenticated with its existing password, inspected the counts and unchecked acknowledgement, then restored. Unlock used the backup password; the expected source/memory appeared and agent sharing stayed off. This exercises first-launch recovery, not new-password creation.
- Kept a source message as a decision, edited it and saved a draft. Reviewed/approved the saved version, selected it in Share context and observed the exact quoted Markdown with Copy/Save initially disabled. Edited the memory again; it returned to Draft. Selecting that revision produced zero included items, one exclusion and disabled Copy/Save. No plaintext was copied or saved.
- Imported the fictional official-format ZIP through the native picker: one new conversation/message appeared beside the restored archive. Agent setup defaulted to shared-memory-only, with consent unchecked and settings creation disabled. Cancel restored opener focus; no registration or model call was made.
- Restored the preview's light appearance, locked it and quit. Temporary fixture cleanup is local and outside Git.

The file chooser required retrying path navigation through its visible controls; this was an automation interaction issue, not evidence of a failed importer. Remaining distinct checks: owner-performed fresh password setup, current-source independent review, human VoiceOver listening, representative upgrade/lifecycle coverage, final saving-state observation, representative Claude/Grok exports and final Apple distribution. Previous Codex/Grok client runs remain the model-led evidence; they were not repeated or charged here.

## Nov 22 — Final saving state and owner-assisted VoiceOver

Used `examples/native_saving_fixture.rs` to construct a ready encrypted checkpoint containing 10,000 fictional conversations and 200,000 short messages, with an initially empty published archive. This fixture uses the real parser and checkpoint payload format but deliberately writes prepared records directly; it is not a new end-to-end ZIP preparation benchmark. The app used its normal publication path with no artificial delays.

- Finish import required no source ZIP. Native accessibility observation captured **Saving your archive…** and **Pause import disabled**. Completion reported 10,000 new / zero refreshed and 200,000 processed messages; pending preparation disappeared.
- Searching the final fictional conversation returned its complete 20-message range. Sharing stayed off. The final state was inspected visually at the compact window size.
- With permission and the owner listening, VoiceOver announced **Vault password** and **Unlock vault** clearly, as confirmed by the owner. The owner was unsure whether saving and completion were announced automatically, so those live announcements remain unverified. This is a limited control-label check, not a full screen-reader pass.
- An initial automated unlock entry failed; pasting the existing fictional phrase unlocked successfully. No password was changed and no damaged-vault claim follows from that input attempt.
- VoiceOver was confirmed off again and Settings returned to General. The fictional vault was locked, its process stopped, and only this run's generated temporary storage removed. No personal vault was opened or modified; screenshots and private diagnostics remain outside Git.

## Nov 22 — Historical version-2 upgrade notice

The separate debug preview opened `native_historical_fixture.rs` storage built from the exact version-2 DDL and invented records. It did not relabel a current vault or open owner storage. At the minimum 980×640 window, the native notice visibly reported **five earlier approvals returned to draft** and **two new privacy restrictions**, in light and dark themes. The default archive hid the private/flagged conversations and Memory showed the safe project as Draft.

Reviewed and approved the safe fictional project, restored light mode, locked and reopened. The notice did not repeat, the project remained Approved and sharing remained off. The preview was then locked/quit and its process confirmed stopped before removing only this run's generated storage. No global accessibility settings, personal vault, cloud connection, password creation/change or permanent native deletion was involved. This checks the successful legacy upgrade and fresh-approval persistence; rollback and historical backup restoration are backend evidence, not native fault injection. VoiceOver announcements remain a distinct open listening check.

## Nov 22 — Packaged fresh-vault attempt, not accepted

Built a separate optimized app from `7e7683d` with a unique compiled identifier and visible fictional title, verified its ad-hoc signature and confirmed its namespace was initially absent. Release builds ignore `PRE_DRINK_DATA_DIR`; the compiled identifier, not an edited bundle label, provided storage separation. Prepared only the existing fictional ZIP fixtures outside the repository.

The app process launched, but native control failed twice before a window could be inspected. The automation submitted no new password, and no first-run creation/import/approval success is claimed. The exact isolated process was stopped; the personal app/vault was not operated. Resume from a fresh namespace when native control is available, with the owner entering and submitting a fictional new password. Follow the current QUICKSTART acceptance sequence in RELEASE_READINESS. Do not repeat already recorded destructive recovery checks just to fill another round.

## Nov 22 — Fresh-vault acceptance completed

The owner directly entered/submitted a new fictional password and reported the empty library. Native controls recovered for this retry and independently observed an empty archive (zero conversations/messages) in the separately identified packaged app, before importing anything. The runtime is the clean tested `7e7683de1f1a` implementation; later commits changed documentation only. No personal vault or export was used.

| Step                                                | Observed result                                                                                                                                                                                                                              |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Official-shaped fictional ZIP through native picker | Two new conversations, six readable messages, zero refreshed. Import notes explained seven omitted hidden/tool/system/non-text messages and attachment limitations.                                                                          |
| Search and source-linked edit                       | Search for the reading nook returned one conversation. Kept its user message, shortened it to a project note and saved a draft; its source link remained visible.                                                                            |
| Draft export exclusion                              | Selecting the draft and building a preview returned zero included, one excluded, zero bytes. Review acknowledgement and Copy/Save were disabled.                                                                                             |
| Review saved version                                | The review dialog showed the saved note, required acknowledgement, and returned an Approved card after confirmation.                                                                                                                         |
| Exact outgoing preview                              | Selecting only that approved note returned one included, zero excluded and 269 bytes: reference-data notice plus the project note, without raw conversation text. Copy/Save remained disabled until preview acknowledgement.                 |
| Native local save                                   | Save dialog wrote the fictional Markdown outside the repository. File content and 269-byte size matched the expected displayed text; permissions were 0600. Success text clearly said the file was outside vault encryption.                 |
| Lock and reopen                                     | Lock showed an empty password field. Reopening with the existing fictional password preserved both conversations and the approved project note. Sharing stayed off; returning to Share context showed zero selected and no outgoing preview. |
| Finish                                              | Locked again and visually checked the complete cream/orange lock screen at the configured 980×640 window size. The fictional app was left locked; its isolated storage was retained.                                                         |

Automation observations: setting the secure field directly did not produce a successful unlock; normal paste of the same existing fictional password succeeded without changing credentials or storage. The initial file-content checker included an extra trailing newline; correcting that checker matched the 269-byte saved file. Neither observation required an app change. The native Go To path field likewise required its supported value-setting action when paste did not update it.

QUICKSTART’s core local flow matched the observed interface. No new external client, sharing grant, cloud/model call, real-provider export, destructive recovery, VoiceOver speech or Apple distribution check was performed. Existing functional test evidence still applies to unchanged implementation; this is additional native acceptance, not an independent security audit.

## Nov 22 — Privacy-change approval reset

Rebuilt the existing isolated fictional packaged app with the reviewed privacy fix; verified its ad-hoc signature. The first UI attempt reached an older still-running test process, so its observations were not accepted as evidence for the fix. Closed that instance, launched the rebuilt app and dismissed macOS's previous-window recovery prompt with Don't Reopen. The updated app then opened normally with the existing fictional password. No personal app or storage was operated.

At 980×640, the new dismissible privacy notice was visible and readable. Making the fictional reading-nook source non-private returned its linked project note to Draft. Selecting it for outgoing context produced **zero included, one excluded, zero bytes**, with acknowledgement, Copy and Save disabled. Fresh saved-version review restored an Approved card. A second complete private/non-private round trip in the rebuilt app returned that note to Draft again. Agent sharing remained off throughout; no client grant, copy or plaintext save occurred. The app was left locked, with the fictional note in Draft and the source non-private.

This verifies the notice and manual-note flow. Brief propagation, stale review/export rejection, no-op behavior, unrelated approvals and injected rollback are automated backend evidence; they were not repeated as native fault injection. VoiceOver automatic speech remains unconfirmed.
