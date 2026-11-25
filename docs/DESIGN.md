# Interface direction

Pre-Drink defaults to a warm cabinet workspace: cream paper, muted cabinet surfaces, serif headings, and burgundy controls. A locally remembered Dark option uses warm charcoal and pale rose accents. The original orange glass remains the app mark. Appearance is independent of vault approval, privacy, and agent-sharing state.

The November 25 cabinet brief informs this foundation pass. It is a visual and interaction direction, not a claim that every proposed product capability is implemented. See the scope mapping below.

## Palette and type

| Role              | Light           | Dark            |
| ----------------- | --------------- | --------------- |
| Canvas            | `#dfd8cc`       | `#23211f`       |
| Paper surface     | `#f2ebdd`       | `#2d2925`       |
| Raised controls   | `#e6ded0`       | `#353029`       |
| Borders           | `#c8bcaa`       | `#52483b`       |
| Primary text      | `#292421`       | `#f1e8da`       |
| Secondary text    | `#60574e`       | `#c4b8a7`       |
| Primary button    | `#702d35`       | `#e3b0a2`       |
| Button text       | `#fff5e9`       | `#341c20`       |
| Small accent text | `#702d35`       | `#edb1a7`       |
| Glass artwork     | orange on cream | orange on cream |

Use locally available Iowan/Palatino/Georgia serif for headings and the wordmark, system sans-serif for reading and controls, and monospace for compact metadata. No remote fonts or bitmap interface. Decorative cabinet drawers contain only fixed labels, never vault data. Functional boundaries, focus indicators, and body text retain contrast checks in both themes. Status always includes words, not color alone.

`public/theme.css` owns semantic colors. `src/cabinet.css`, loaded after the older screen styles, owns the cabinet foundation while individual screens migrate. It does not replace the existing keyboard, modal, privacy, or import logic. Reduced-motion preference disables transitions and animations.

## One glass mark

`src-tauri/icons/source.svg` is the canonical glass silhouette: orange on a cream rounded tile. The favicon and in-app brand images use that same artwork; generated PNG/ICNS assets package it for macOS. The documentation mark is the same vector scaled down. Do not substitute the generic Lucide glass icon in the lock screen or sidebar. Other utility icons may use Lucide.

## Appearance behavior

The lock header and workspace sidebar each expose labeled Light and Dark buttons with a pressed state. Light is the default even when macOS uses dark mode. `public/theme.js` restores the preference before the app bundle paints; `public/theme.css` defines shared semantic colors for startup, forms, dialogs, archive, memory, briefs and sharing. Only the non-sensitive value `light` or `dark` is saved in WebView local storage. Unavailable storage falls back safely; no vault contents or credentials are written there. The native window theme and background are also requested through narrowly allowed appearance APIs. Native title-bar styling may vary with the OS.

## Responsive lock screen

The lock screen occupies one viewport, with a spacious brand/story column and a focused password panel. Technical diagnostics and backup restoration open contained dialogs instead of extending the page. The same first-run form, wrong-password error and recovery actions must fit the supported 980 × 640 window. Short windows reduce spacing, and narrow browser previews omit the decorative story column. Exceptional long recovery/migration content and enlarged text retain internal scrolling as an accessibility fallback.

The password panel keeps the no-reset message and local-storage boundary explicit. Changing the theme never changes the password or sharing state. Recovery dialogs keep their pending-action guard and keyboard cancellation.

## Workspace

The sidebar is 208 pixels wide. Repeated import-source and three-layer copy is removed. The header keeps lock and agent-sharing state visible while scrolling. The reader has more room, with privacy and reviewed deletion beside its title. Memory/brief filters wrap at narrow widths. Long content scrolls inside the workspace; the one-viewport requirement applies specifically to the ordinary lock screen.

The sharing picker renders 25 choices per page and retains selections across pages and searches. Exact outgoing Markdown and exclusions remain the review point. Paging and visual changes do not change approval or export policy.

## Distribution and review state

The owner supplied a cabinet-style UI brief after the earlier cream/orange direction. This foundation keeps light as default, retains dark mode and the orange glass, and applies the new paper/burgundy direction. These refinements are implemented for review; they are not a claim that final visual acceptance or public-launch readiness is complete. Native checks use only fictional data. The static landing page still needs a full visual alignment pass and remains unpublished. Keep the repository private until the owner's explicit launch decision.

## Provider curation controls

Agent access contains the provider workspace selector, 30-item draft pages, explicit selection and complete batch review. Cards distinguish Private draft, Withheld, Shared after your review, Shared by your automatic rule and Withdrawn. Do not collapse automatic publication into a “reviewed” badge. Facts/links needing review are explained, and source/privacy restrictions cannot be dismissed as a styling issue.

Edit/history and batch review use the shared contained modal with keyboard focus isolation and Escape cancellation while idle. Closing restores focus to the opening control. Earlier text is saved as a new held revision; it never silently reinstates a previous release. Manual Memory cards direct agent contribution edits back to this workspace.

Keep automatic rules collapsed and off initially. Consent explains future eligible project/decision publication and distinguishes archive from session origin. Registration separates source access, curation and provider-processing consent. Cloud curation is optional; avoid copy implying that connecting a cloud model is required to use the app. A local-model option must not appear available until implemented.

## Cabinet implementation and proposed follow-up

| Brief direction                                    | Current implementation / boundary                                                                                                                                                                                          |
| -------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cabinet shell and paper surfaces                   | Theme tokens, 208px navigation, editorial headings, paper cards, drawer channels, restrained depth; no continuous animation or sound.                                                                                      |
| Memory cabinet                                     | Existing paginated library; type filter buttons and card/list layouts; local search and restricted-note toggle. Counts come from the actual filtered library.                                                              |
| Read a memory                                      | A contained side dialog displays complete inert text, review state, sharing eligibility, curator revision when available, and the existing source link. Escape closes and returns focus to the opener.                     |
| Review versus access                               | Draft, Reviewed, and Auto-published describe publication basis. Withheld / Not approved / Eligible describe eligibility. Eligibility explicitly does not grant an agent access. Existing backend enforcement is unchanged. |
| Briefs                                             | Paper-folder styling over the working brief editor, linked memories, review, and deletion. No invented project folder model or snapshot semantics.                                                                         |
| Archive and lock                                   | Larger archive titles/excerpts, paper reader, consistent glass mark, compact viewport lock form. Decorative drawers disappear in short windows.                                                                            |
| Review Queue and Projects navigation               | Proposed, not added. Current review actions and provider curation stay in their existing screens. No dead navigation.                                                                                                      |
| Global search / command palette                    | Proposed. Existing archive, memory, and brief searches retain their actual scope.                                                                                                                                          |
| Project folders, preferences, constraints          | Proposed schema changes. Existing memory types remain fact, decision, project.                                                                                                                                             |
| Project/brief-specific agent grants                | Proposed backend work. Current per-client and provider boundaries remain authoritative.                                                                                                                                    |
| Autosave, conflict merging, local AI configuration | Proposed; not implied by the new interface. Explicit save and approval-reset behavior remain unchanged.                                                                                                                    |

New UX work must preserve manual-versus-automatic publication labeling, source restrictions, explicit review, provider isolation, encrypted storage, and disclosure before any cloud processing. The attached brief's unrelated product concepts are outside this app's scope.

## November 25 verification

The frontend build and 25 reading regressions pass, including both-theme contrast, independent review/access labels, full inert note text, and inspector disclosure. Native checks covered the light lock screen and light/dark workspace at 1280 × 840, both-theme unlock screens and the light first-run screen at 980 × 640, using only the fictional fixture. The first pass exposed an old unlock-button color override, corrected and rechecked in both themes. Native filtering, search, restricted visibility, inspector Tab wrapping/Escape/focus return, and brief-editor cancellation also passed. Source navigation exposed a title-search bug: the owner list command now accepts an exact source ID, with the existing restricted-content toggle still enforced. A regression covers identical punctuated titles, sources beyond the first page, missing IDs, and both private/sensitive restrictions. The exact source-link native retest passed, including return to the full archive. The full local verifier passed 229 checks; details are recorded in WORK_LOG. Tests use a new fictional encrypted cabinet fixture; no personal archive is used for screenshots or design examples. Final owner visual acceptance remains open.
