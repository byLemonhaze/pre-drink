# Interface direction

Pre-Drink defaults to warm cream and orange. A locally remembered Dark option keeps the charcoal/orange interface available. Appearance is a visual preference, independent of vault approval, privacy and agent-sharing state. The lock screen should feel spacious and welcoming; the workspace should prioritize readable history, clear curation state and deliberate sharing.

## Palette and type

| Role              | Light     | Dark      |
| ----------------- | --------- | --------- |
| Canvas            | `#f5efe4` | `#101112` |
| Surface           | `#fffaf1` | `#181a1c` |
| Raised controls   | `#eee5d7` | `#242729` |
| Borders           | `#d9cbb8` | `#34393b` |
| Primary text      | `#302b25` | `#e7e9e9` |
| Secondary text    | `#6e6357` | `#a4abad` |
| Brand orange      | `#ff742b` | `#ff742b` |
| Small accent text | `#a93d0a` | `#ff742b` |
| Success text      | `#396044` | `#9bc5ae` |
| Error text        | `#a7352f` | `#f1a19a` |

Use the system sans-serif stack for the wordmark, content and controls. Keep system monospace for compact metadata and code. No remote fonts. In light mode small accent text uses a deeper orange for legibility; large brand artwork and the unlock action retain the bright orange. Status always includes words, not color alone.

## One glass mark

`src-tauri/icons/source.svg` is the canonical glass silhouette: orange on a cream rounded tile. The favicon and in-app brand images use that same artwork; generated PNG/ICNS assets package it for macOS. The documentation mark is the same vector scaled down. Do not substitute the generic Lucide glass icon in the lock screen or sidebar. Other utility icons may use Lucide.

## Appearance behavior

The lock header and workspace sidebar each expose labeled Light and Dark buttons with a pressed state. Light is the default even when macOS uses dark mode. `public/theme.js` restores the preference before the app bundle paints; `public/theme.css` defines shared semantic colors for startup, forms, dialogs, archive, memory, briefs and sharing. Only the non-sensitive value `light` or `dark` is saved in WebView local storage. Unavailable storage falls back safely; no vault contents or credentials are written there. The native window theme and background are also requested through narrowly allowed appearance APIs. Native title-bar styling may vary with the OS.

## Responsive lock screen

The lock screen occupies one viewport, with a spacious brand/story column and a focused password panel. Technical diagnostics and backup restoration open contained dialogs instead of extending the page. The same first-run form, wrong-password error and recovery actions must fit the supported 980 × 640 window. Short windows reduce spacing, and narrow browser previews omit the decorative story column. Exceptional long recovery/migration content and enlarged text retain internal scrolling as an accessibility fallback.

The password panel keeps the no-reset message and local-storage boundary explicit. Changing the theme never changes the password or sharing state. Recovery dialogs keep their pending-action guard and keyboard cancellation.

## Workspace

The sidebar is 176 pixels wide (164 at narrower widths). Repeated import-source and three-layer copy is removed. The header keeps lock and agent-sharing state visible while scrolling. The reader has more room, with privacy and reviewed deletion beside its title. Memory/brief filters wrap at narrow widths. Long content scrolls inside the workspace; the one-viewport requirement applies specifically to the ordinary lock screen.

The sharing picker renders 25 choices per page and retains selections across pages and searches. Exact outgoing Markdown and exclusions remain the review point. Paging and visual changes do not change approval or export policy.

## Source-only presentation

The cream/orange light interface and charcoal/orange dark option remain the current visual base. The November cabinet experiment was withdrawn; screenshots must show the current app, with fictional content only. The repository is source-only work in progress, with no downloadable Mac release. Visual refinement remains open and does not change approval, privacy, or agent permissions.

## Provider curation controls

Agent access contains the provider workspace selector, 30-item draft pages, explicit selection and complete batch review. Cards distinguish Private draft, Withheld, Shared after your review, Shared by your automatic rule and Withdrawn. Do not collapse automatic publication into a “reviewed” badge. Facts/links needing review are explained, and source/privacy restrictions cannot be dismissed as a styling issue.

Edit/history and batch review use the shared contained modal with keyboard focus isolation and Escape cancellation while idle. Closing restores focus to the opening control. Earlier text is saved as a new held revision; it never silently reinstates a previous release. Manual Memory cards direct agent contribution edits back to this workspace.

Keep automatic rules collapsed and off initially. Consent explains future eligible project/decision publication and distinguishes archive from session origin. Registration separates source access, curation and provider-processing consent. Cloud curation is optional; avoid copy implying that connecting a cloud model is required to use the app. A local-model option must not appear available until implemented.
