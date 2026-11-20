#!/usr/bin/env python3
"""Verify actual fictional Codex MCP events, not the model's completion claim.

Run after draft/resume, an explicit fixture-owner review/release, and retrieve.
This does not call a model or inspect personal storage. Semantic quality still
requires reading the invented notes; these assertions cover observable actions.
"""
import json
from pathlib import Path
import sys

root = Path(sys.argv[1]).resolve(strict=True)
assert root.parent == Path('/private/tmp') and root.name.startswith('pd-curator-client-')


def calls(phase):
    items = []
    completed = False
    for line in (root / f'{phase}.jsonl').read_text().splitlines():
        try:
            event = json.loads(line)
        except ValueError:
            continue  # Client diagnostic lines share the private log.
        assert event.get('type') not in {'turn.failed', 'error'}, 'Client error event'
        completed |= event.get('type') == 'turn.completed'
        if event.get('type') != 'item.completed':
            continue
        item = event.get('item', {})
        assert item.get('type') in {'agent_message', 'reasoning', 'mcp_tool_call'}, 'Unexpected tool/item'
        if item.get('type') == 'mcp_tool_call':
            assert item['server'] == 'pre-drink'
            assert item['tool'] in {'curate_memory', 'get_projects', 'get_context_pack'}
            items.append(item)
    assert completed and items
    return items


def operation(call):
    return call.get('arguments', {}).get('operation', {})


def data(call):
    assert call['status'] == 'completed'
    return json.loads(call['result']['content'][0]['text'])


def writes(items, key):
    return [c for c in items if operation(c).get('action') == 'write'
            and operation(c)['input']['request_key'] == key]


draft, resume, retrieve = [calls(p) for p in ['draft', 'resume', 'retrieve']]
denied = [c for c in draft if c['status'] == 'failed']
assert len(denied) == 2
assert {operation(c)['message_id'].split(':')[0] for c in denied} == {'grok', 'claude'}
assert all(operation(c)['action'] == 'read_source' for c in denied)
assert all(c['status'] == 'completed' for c in resume)
retrieval_denied = [c for c in retrieve if c['status'] == 'failed']
assert len(retrieval_denied) == 1 and operation(retrieval_denied[0]) == {'action': 'status'}

session = writes(draft, 'session-handoff')
assert len(session) == 2 and session[0]['arguments'] == session[1]['arguments']
assert data(session[0]) == data(session[1])
assert data(session[0])['revision'] == 1 and not data(session[0])['published']
initial = writes(draft, 'archive-initial')
final = writes(resume, 'archive-final')
assert len(initial) == 1 and len(final) == 2
assert final[0]['arguments'] == final[1]['arguments']
assert data(final[0]) == data(final[1])
assert data(final[0])['id'] == data(initial[0])['id']
assert data(final[0])['revision'] == 2 and not data(final[0])['published']
old_sources = {s['message_id'] for s in operation(initial[0])['input']['sources']}
new_sources = {s['message_id'] for s in operation(final[0])['input']['sources']}
assert old_sources <= new_sources and any(s.endswith(':03') for s in new_sources)
for items, expected in [(draft, {'01', '02'}), (resume, {'03', '04'})]:
    reads = [c for c in items if operation(c).get('action') == 'read_source' and c['status'] == 'completed']
    assert {data(c)['message_id'].rsplit(':', 1)[1] for c in reads} == expected
    assert all(data(c)['next_offset'] is None for c in reads)
    projects = [c for c in items if c['tool'] == 'get_projects']
    assert projects and all(data(c) == [] for c in projects)

jobs = [c for c in draft + resume if operation(c).get('action') == 'save_job']
assert len(jobs) == 3
assert len({data(c)['id'] for c in jobs}) == 1
assert [data(c)['checkpoint'] for c in jobs] == [0, 2, 4]
assert all(data(c)['source_count'] == 4 for c in jobs)
assert operation(jobs[-1])['status'] == 'completed'
assert any(operation(c).get('action') == 'jobs' for c in resume)

projects = [data(c) for c in retrieve if c['tool'] == 'get_projects']
assert len(projects) == 1 and len(projects[0]) == 2
assert {n['id'] for n in projects[0]} == {data(session[0])['id'], data(final[0])['id']}
packs = [data(c) for c in retrieve if c['tool'] == 'get_context_pack']
assert len(packs) == 1
pack = json.dumps(packs[0]).lower()
assert all(term in pack for term in ['cedar', 'next week'])
assert 'plant selection remains open' in pack or 'undecided' in pack
assert 'hidden-garden-canary' not in pack and 'ignore the curator' not in pack
assert 'fictional-courtyard:01' not in pack  # No raw source IDs in shared output.
print(json.dumps({'passed': True, 'model_sessions': 3,
                  'mcp_calls': len(draft) + len(resume) + len(retrieve),
                  'expected_denials': 3, 'shared_notes': 2,
                  'checks': ['foreign-source isolation', 'private session retry',
                             'archive revision retry', 'fresh-session job resume',
                             'drafts excluded', 'reviewed shared retrieval',
                             'shared-reader curation denial', 'restricted source exclusion']}))
