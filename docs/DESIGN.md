# Interface direction

Pre-Drink uses a dark, system-inspired interface with restrained retro details. It should feel like a useful local utility: flat graphite panels, clear borders, compact controls and readable content.

## Palette and type

| Role                   | Value     |
| ---------------------- | --------- |
| Canvas                 | `#111316` |
| Surface                | `#181b20` |
| Raised controls        | `#22262d` |
| Borders                | `#343a43` |
| Primary text           | `#e6e8ec` |
| Secondary text         | `#a1a7b1` |
| Amber selection/action | `#deb477` |
| Success status         | `#9bc5ae` |
| Error text             | `#f1a19a` |

Use the system sans-serif font for content and controls, and the local system monospace stack for metadata, counters and small labels. No remote fonts. Keep reading text at comfortable sizes; avoid applying terminal styling to entire conversations.

## Component rules

- Flat panel divisions instead of floating decorative cards where practical.
- Small corner radii and visible keyboard focus rings.
- Amber for the primary action and selected navigation; muted green only for status.
- Status always includes words, not color alone.
- Native dark appearance for window chrome and form controls.
- Plain screen names: Archive, Memory, Briefs, Agent access, Import history, Vault settings.
- Preserve reduced-motion preferences; no simulated CRT flicker or scanlines over text.
- Carry the same colors and typography into the landing page and icon.

## Distribution state

The repository remains private until the owner explicitly makes it public. GitHub Pages is unpublished. Landing-page changes are local/private source changes only; never re-enable hosting or change visibility as part of visual work.

## Visual verification

Native unlock, archive, source-linked memory editor and manual lock were inspected using an isolated synthetic vault. No personal chat content was used or added to the repository. Check the landing page at desktop and narrow mobile sizes before publishing.
