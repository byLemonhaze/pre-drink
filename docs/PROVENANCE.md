# Provenance and attribution

Pre-Drink is an **independent Tauri/Rust/React implementation**, not a GitHub fork of Loominary and not a repackaged build of it. There is no upstream merge history to claim.

The starting product brief identified [Laumss/loominary](https://github.com/Laumss/loominary) as a reference for official export structure, conversation branches, search and tagging ideas. Its [ChatGPT parser](https://github.com/Laumss/loominary/blob/main/src/utils/fileParser/chatgptParser.js) informed understanding of the format. This project uses its own Rust parser and database model. No Loominary source files, userscripts, extensions, branding or live-capture features were copied.

Loominary's [public README](https://github.com/Laumss/loominary#readme) describes a broader platform and capture/export feature set. Pre-Drink deliberately uses official ChatGPT/Claude/Grok account exports and emphasizes an encrypted native vault, reviewed memory, curated briefs and scoped local MCP. This is a product-direction comparison, not a claim of overall superiority or equivalent maturity. Refer to Loominary's current documentation for its capabilities rather than assuming absences.

The brief also names Chat Locker as a product comparison: the desired distinction is curated structured memory, agent access and controlled context packs. This repository makes no independently verified claims about Chat Locker's current feature set.

Third-party libraries remain under their respective licenses, as recorded by their packages. Pre-Drink’s original code is released under the MIT license in the repository root, reflecting the free/open-source product direction. Third-party code is not relicensed by that file. Before copying third-party implementation code later, inspect its exact license, preserve required notices, and record copied files/versions here. Do not describe reference use as a fork.

Repository documentation captures generic product decisions, not private conversation transcripts or user data. All test fixtures are synthetic.

The independent Grok account adapter used the public field descriptions/structure in [NinZine/chat-converter](https://github.com/NinZine/chat-converter/blob/master/grok_parser.py); the Claude adapter used [Loominary’s Claude format handling](https://github.com/Laumss/loominary/blob/main/src/utils/fileParser/claudeParser.js) as a field reference. No implementation was copied. Provider manifests link the references and distinguish invented fixture data from representative owner exports. Official download guidance and additional-provider candidates are linked in PROVIDERS.
