# Roadmap

Current scope: **ChatGPT only**, local and offline by default. Encrypted archive → reviewed memory → approved briefs, with controlled agent access.

## Implemented

- Tauri 2 + React; official ZIP import; branch-aware reader; repeat imports; JSON snapshots; FTS5.
- Separate-password SQLCipher vault; Argon2id derivation; plaintext-v0.1 migration; verified encrypted backups.
- Locked startup; manual/idle/macOS lifecycle locking; sharing resets on lock/restart.
- Editable facts/decisions/projects with source navigation, deletion, explicit approval, and privacy inheritance.
- Editable Markdown briefs with linked memories; source changes/restrictions block sharing pending review.
- Official SDK stdio MCP server; authenticated private socket; revocable clients; default memory/brief scope; optional safe chat access; six tools.
- Context preview and backend privacy enforcement.
- Importer/security/MCP integration checks and native synthetic curation-flow verification.

## Remaining release and usability gates

- Developer ID signing and Apple notarization require the owner's credentials. Workflow prepared; public signing not completed.
- Independent security review before stronger assurance claims.
- Representative real ChatGPT exports, physical native file drag, large-library performance/cancellation.
- Individual installed Codex/Claude Code/Cursor setup verification; standard stdio protocol integration is tested.
- In-app backup restore and password rotation (restore currently documented/manual).
- Dedicated selective Markdown file export and per-brief client grants. Current grants cover all eligible approved memory/briefs; topics are searches, not access boundaries.
- Pagination for larger agent result sets and better topic matching.
- Conversation tagging UI (schema exists).

## Explicitly later

Claude and Grok official-export parsers. Optional local-model assistance if requested. No automatic extraction, cloud sync, semantic embeddings, Gemini, mobile, live capture, or team features in current scope.

A successful local build is not equivalent to a publicly audited, signed release. Track these gates honestly.
