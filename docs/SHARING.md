# Sharing selected context

Use **Share context** to prepare one deliberate copy for another product. You need no model, API key or internet connection.

1. Create/edit memories or briefs, save them, then review and approve their saved versions.
2. Open **Share context**. Select the notes and briefs you want to include, up to 50. Search filters the chooser without clearing selections.
3. Choose **Build exact preview**. The screen shows the full outgoing Markdown, included/excluded counts and UTF-8 byte size. Exclusion reasons are visible only in the app and are not part of the exported text.
4. Read the full preview and check **I reviewed this exact text and want to share it**.
5. Choose **Copy Markdown** or **Save Markdown**. Save requires a new filename ending in `.md`; existing files are refused. Cancelling the file dialog writes nothing.
6. Decide yourself whether to paste/upload that copy into another product. Pre-Drink does not send it to a provider.

## What is eligible

Only approved, non-private, non-sensitive saved records with eligible sources are included. Private/detected-sensitive content cannot be overridden here. Drafts and briefs with missing, changed or unapproved sources are excluded. If nothing is eligible, delivery remains disabled. A selection above 128,000 bytes is rejected rather than truncated.

The export contains the chosen note text/brief title and content. It adds no archive conversation titles, message identifiers, automatic source excerpts or exclusion notices. If you manually wrote identifying information into an otherwise eligible note, that text still appears: review remains essential and detection is heuristic.

## Changes after preview

Copy/save recompute the payload and compare the saved versions, source states and unlock session to the preview. If anything relevant changed, the operation stops and you must build/review a new preview. An import that changes source message text/role returns linked memories to draft and blocks dependent briefs. Identical reimports preserve approval.

Locking/reopening invalidates the preview. A copy already completed cannot be recalled. Delivery and owner edits/locking serialize; a copy already underway can finish before a waiting lock takes effect.

## Where protection ends

The clipboard and `.md` file are plaintext outside the encrypted vault. On macOS, copying uses [Apple’s current-host-only option](https://developer.apple.com/documentation/appkit/nspasteboard/contentsoptions/currenthostonly) and first writes the [confidential clipboard marker](https://nspasteboard.org/). This limits Universal Clipboard transfer and asks cooperating managers to treat the content as confidential; it cannot enforce every manager’s behavior. Clipboard managers, other apps, cloud-synced folders and receiving AI providers may retain copies. New files have owner-only permissions on macOS, but that does not encrypt them. No automatic clipboard clearing or remote deletion is promised.

This selection has no effect on **Agent access**. Existing MCP grants can query all eligible memories/briefs in their scope, and optionally raw chats. Previewing one brief or topic does not narrow those permissions. Manage or revoke clients separately.

## Current limits

No export presets, automatic instruction-file installation, per-project MCP grants or raw-chat selection. Native clipboard support is macOS-only. In-app restore/password change are separate reviewed lifecycle operations that invalidate this session’s previews. Automated and native evidence is recorded in [VALIDATION](VALIDATION.md).
