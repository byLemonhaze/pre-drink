# Curation quality and effort acceptance

The installed-client checks prove a small workflow, not that an entire archive becomes good memory. This protocol measures usefulness and owner effort before public claims. It responds to the [Nov 20 product review](https://pre-drink-product-take-and-launch-ny2r.pages.grok.bot/), whose code baseline was `2ed7241`. No real-export quality run has been completed under this protocol.

## Data and consent boundary

Use fictional format-matched archives first. A real-export run uses a separate encrypted copy with a verified backup, never the original working vault. Keep raw outputs, note texts, titles, source IDs and model logs outside Git. Publish only approved aggregate results that reveal no personal projects or content.

Local import permission is not permission to send an archive to a cloud model. Before a real cloud run, the owner chooses the provider, eligible data scope, and a bounded usage budget. Same-provider permissions stay enforced. Manual local work remains available; there is no bundled local model. Do not enable automatic publication merely for this experiment.

## Measure one bounded batch

Record the app revision, parser contract, model/client version, provider, batch size, message/text volume, and whether the material is fictional or owner-local. Establish the source-based expected decisions, unresolved questions and project facts before scoring the draft. Include rejected assistant suggestions, changed decisions, repeated discussions, unknowns, sensitive sources and instruction-like transcript text.

| Measure              | How to record it                                                                                                                                                               |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Setup effort         | Owner-active minutes from an unlocked imported vault to the first successful scoped client call; count errors and manual settings steps separately.                            |
| Useful result time   | Wall time and owner-active time until one correct, reusable, released project summary. Keep import/model/wait/review times separate.                                           |
| Factual precision    | Supported atomic claims / all atomic claims in a draft. Mark unsupported, contradicted and uncertain claims separately. A concise omission is different from an invented fact. |
| Important coverage   | Expected high-value decisions/questions represented / expected high-value decisions/questions in the sampled batch. Do not extrapolate sampled coverage to the whole archive.  |
| Correction burden    | Notes accepted unchanged, edited, merged, rejected and still undecidable; active review minutes and corrections per accepted note.                                             |
| Consolidation        | Duplicate notes, superseded decisions left as current, and relevant material stranded in unrelated notes.                                                                      |
| Evidence quality     | Correct provider, current source versions and sufficient cumulative evidence for each archive contribution.                                                                    |
| Retrieval usefulness | Known-topic questions answered by released notes; count lexical misses separately from extraction errors. Preserve recognizable project names.                                 |
| Publication behavior | Actual release basis and withheld reasons. With rules off, automatic publication is zero by design; this is not a quality failure.                                             |
| Usage                | Actual subscription/API usage if the client exposes it. Label unavailable costs as unknown, never as free.                                                                     |

For optional automatic rules, first use fictional cases with a narrow explicit owner opt-in. Record eligible/held/released counts and reasons, plus false-safe and unnecessary-review outcomes. A high auto-release rate is not a success criterion. Any incorrectly released sensitive/stale/foreign-source item stops acceptance and requires a regression fix. Never weaken credential detection or source restrictions to improve that rate.

## Release interpretation

Functional blockers: foreign access, unauthorized release, unsupported decisive claims presented as certain, lost provenance, retry duplication or failed revocation. Fix these before widening the run. Review burden and usefulness require owner assessment; no universal percentage is asserted without representative evidence. Compare two prompt/workflow revisions on the same bounded sample before claiming improvement.

Current evidence: one small fictional English project per Codex/Grok client, with correct decision/suggestion distinction and resume/retry behavior. Grok's summary omitted the project label, producing a lexical context-pack miss even though `get_projects` returned both notes. This is an explicit follow-up quality case. Broad contradiction detection remains the curator's responsibility, not a backend semantic capability.

## Versioned fictional corpus (quality-v1)

`src-tauri/tests/fixtures/curation/quality-v1/cases.json` contains ten invented cases and a separate reviewer rubric. Run from the repository root:

```sh
python3 scripts/create_curation_quality_exports.py
cargo run --manifest-path src-tauri/Cargo.toml --locked --example verify_quality_exports
```

The packager creates three ignored local ZIPs, using the supported ChatGPT mapping, Grok account and Claude message shapes. It refuses overwrite. Only source messages enter ZIPs; keep `expected` and `must_not` out of the curator prompt. The verification example checks exact source text/roles through every production parser and checks that the fictional contact source stays absent from scoped source listings. These are format-matched examples, not official provider exports.

Run each provider in a fresh isolated encrypted vault with automatic rules off. Use stable conversation order for the linked Orchid Courtyard material/change/confirmation cases; then assess consolidation across conversations. Record the earlier versus current decision, uncertainty, correct evidence, redundant drafts, and owner corrections against the rubric. Retain literal project names and query Tangerine Atlas after review to test retrieval. Hostile quoted text must never become an action; a transcript statement cannot broaden provider permissions. French is a quality case, not evidence of multilingual sensitive/instruction detection.

Status: packaging, all three parser paths and contact-source exclusion verified locally. **No model has been scored on this broader corpus yet.** The earlier Codex/Grok small-workflow results remain the only installed-model evidence. Additional cloud runs require a bounded usage decision; real-data runs also require explicit provider/data consent.
