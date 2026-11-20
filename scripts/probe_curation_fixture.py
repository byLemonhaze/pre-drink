#!/usr/bin/env python3
"""No-model stdio delivery check for the disposable writing-acceptance fixture."""
import argparse
import json
import os
from pathlib import Path
import selectors
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('root', type=Path)
group = parser.add_mutually_exclusive_group(required=True)
group.add_argument('--projects', type=int)
group.add_argument('--locked', action='store_true')
args = parser.parse_args()
root = args.root.resolve(strict=True)
assert root.parent == Path('/private/tmp') and root.name.startswith('pd-curator-client-')
config = json.loads((root / 'reader.json').read_text())
assert Path(config['env']['PRE_DRINK_SOCKET']).resolve() == root / 'agents.sock'
assert config['args'] == ['--mcp']
process = subprocess.Popen([config['command'], '--mcp'], stdin=subprocess.PIPE,
                           stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                           env={**os.environ, **config['env']})
selector = selectors.DefaultSelector()
selector.register(process.stdout, selectors.EVENT_READ)
pending = b''


def send(message):
    process.stdin.write(json.dumps(message).encode() + b'\n')
    process.stdin.flush()


def receive():
    global pending
    while b'\n' not in pending:
        assert selector.select(10), 'Bridge timed out'
        chunk = os.read(process.stdout.fileno(), 65536)
        assert chunk, 'Bridge closed'
        pending += chunk
    line, pending = pending.split(b'\n', 1)
    return json.loads(line)


try:
    send({'jsonrpc': '2.0', 'id': 1, 'method': 'initialize', 'params': {
        'protocolVersion': '2025-11-25', 'capabilities': {},
        'clientInfo': {'name': 'fictional-delivery-probe', 'version': '1'}}})
    assert receive()['result']['serverInfo']['name'] == 'pre-drink'
    send({'jsonrpc': '2.0', 'method': 'notifications/initialized'})
    send({'jsonrpc': '2.0', 'id': 2, 'method': 'tools/call',
          'params': {'name': 'get_projects', 'arguments': {}}})
    result = receive()['result']
    if args.locked:
        assert result.get('isError') is True
    else:
        assert not result.get('isError')
        projects = json.loads(result['content'][0]['text'])
        assert len(projects) == args.projects
        if args.projects == 1:
            assert projects[0]['content'] == 'Fictional courtyard handoff: review the planters next week.'
    print(json.dumps({'passed': True, 'locked_denial': args.locked, 'expected_projects': args.projects}))
finally:
    process.stdin.close()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)
    selector.close()
