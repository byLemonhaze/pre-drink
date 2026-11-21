#!/usr/bin/env python3
"""Opt-in Grok ACP acceptance using only disposable provider-matched fixtures."""
import argparse
import fcntl
import json
import os
from pathlib import Path
import queue
import signal
import subprocess
import threading
import time
from curation_prompts import common, prompts

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('config', type=Path)
parser.add_argument('phase', choices=['draft', 'resume', 'retrieve'])
parser.add_argument('--foreign-source', action='append', default=[])
parser.add_argument('--allow-subscription-usage', action='store_true')
args = parser.parse_args()
if not args.allow_subscription_usage:
    parser.error('Explicit subscription-usage consent required.')
path = args.config.resolve(strict=True)
root = path.parent
if root.parent != Path('/private/tmp') or not root.name.startswith('pd-curator-client-'):
    parser.error('Only disposable curation fixtures are accepted.')
if path.name != ('reader.json' if args.phase == 'retrieve' else 'curator.json'):
    parser.error('Use the correct fictional role configuration.')
for p in [root, path]:
    if p.stat().st_uid != os.getuid() or p.stat().st_mode & 0o077:
        parser.error('Fixture and credentials must be owner-only.')
config = json.loads(path.read_text())
if config.get('fixture_provider') != 'grok':
    parser.error('Grok must use its own provider-scoped fixture.')
assert Path(config['env']['PRE_DRINK_SOCKET']).resolve() == root / 'agents.sock'
assert config['args'] == ['--mcp']
assert set(config['env']) == {'PRE_DRINK_SOCKET', 'PRE_DRINK_CLIENT_ID', 'PRE_DRINK_CLIENT_TOKEN'}
auth = Path.home() / '.grok/auth.json'
if not auth.is_file():
    parser.error('Existing Grok sign-in required. No login or API-key fallback is attempted.')
ghome = root / 'grok-home'
ghome.mkdir(mode=0o700, exist_ok=True)
cache = ghome / 'auth.json'
if not cache.exists():
    # A private copy prevents a client's refresh from modifying the real cache.
    with os.fdopen(os.open(cache, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), 'wb') as f:
        f.write(auth.read_bytes())
if cache.is_symlink() or cache.stat().st_uid != os.getuid() or cache.stat().st_mode & 0o077:
    parser.error('Isolated authentication cache must be a private regular file.')
(ghome / 'config.toml').write_text('''[compat.claude]
skills=false
rules=false
agents=false
mcps=false
hooks=false
[compat.cursor]
skills=false
rules=false
agents=false
mcps=false
hooks=false
[memory]
enabled=false
[subagents]
enabled=false
[cli]
auto_update=false
use_leader=false
''')
env = {k:v for k,v in os.environ.items() if k not in {'XAI_API_KEY','GROK_API_KEY'} and not k.startswith('PRE_DRINK_')}
env.update(GROK_HOME=str(ghome), GROK_DISABLE_AUTOUPDATER='1', GROK_MEMORY='0', GROK_SUBAGENTS='0')
inspection = json.loads(subprocess.check_output(['grok','--cwd',str(root/'work'),'inspect','--json'],env=env,timeout=15))
assert all(s.get('disabled') for s in inspection['mcpServers'])
assert not any(inspection[k] for k in ['hooks','plugins','projectInstructions'])
# Grok installs its public bundled skill catalog during its first invocation.
# Permit only that catalog, never user/project skills or compatibility imports.
assert all(s.get('source',{}).get('type') == 'bundled' and Path(s['source']['path']).resolve().is_relative_to(ghome/'bundled') for s in inspection['skills'])
prompt = (common + prompts[args.phase]).replace('ChatGPT', 'Grok').replace('pre-drink', 'pre_drink')
prompt += '\nUse built-in tool discovery only if needed to find these three Pre-Drink tools: curate_memory, get_projects, get_context_pack.'
if args.phase == 'draft':
    if len(args.foreign_source) != 2:
        parser.error('Supply both foreign IDs from fixture startup.')
    prompt += '\nForeign source IDs for denial checks: ' + json.dumps(args.foreign_source)
counter = os.open(root/'grok-model-invocations', os.O_RDWR|os.O_CREAT, 0o600)
with os.fdopen(counter, 'r+') as f:
    fcntl.flock(f, fcntl.LOCK_EX)
    count = int(f.read() or '0')
    if count >= 3:
        parser.error('Three-invocation limit reached; no model call made.')
    f.seek(0); f.write(str(count+1)); f.truncate(); f.flush()
log = os.fdopen(os.open(root/(args.phase+'.acp.jsonl'),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600),'w')
normalized = os.fdopen(os.open(root/(args.phase+'.jsonl'),os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600),'w')
process = subprocess.Popen(['grok','--tools','','--disable-web-search','--no-subagents','--max-turns','25',
    '--sandbox','read-only','--cwd',str(root/'work'),'agent','--no-leader','stdio'],
    stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True,env=env,start_new_session=True)
messages = queue.Queue()

def read():
    for line in process.stdout:
        try: messages.put(json.loads(line))
        except ValueError: pass
    messages.put(None)

threading.Thread(target=read,daemon=True).start()
seq, calls, discovery = 0, {}, set()
allowed = {'pre_drink__curate_memory','pre_drink__get_projects','pre_drink__get_context_pack'}

def send(value):
    process.stdin.write(json.dumps(value)+'\n'); process.stdin.flush()

def request(method, params, timeout=30):
    global seq
    seq += 1; ident = seq; end = time.monotonic()+timeout
    send({'jsonrpc':'2.0','id':ident,'method':method,'params':params})
    while True:
        remaining = end-time.monotonic()
        if remaining <= 0: raise TimeoutError('Grok acceptance deadline exceeded')
        value = messages.get(timeout=remaining)
        if value is None: raise RuntimeError('Grok connection closed')
        if value.get('method') == 'session/request_permission':
            tc=value['params'].get('toolCall',{}); title=tc.get('title','')
            permitted = title in allowed or title == 'search_tool'
            option=next((o['optionId'] for o in value['params'].get('options',[]) if o.get('kind')=='allow_once'),None) if permitted else None
            outcome={'outcome':'selected','optionId':option} if option else {'outcome':'cancelled'}
            send({'jsonrpc':'2.0','id':value['id'],'result':{'outcome':outcome}})
        elif 'method' in value and 'id' in value:
            send({'jsonrpc':'2.0','id':value['id'],'error':{'code':-32601,'message':'Unavailable in fictional acceptance'}})
        elif value.get('method') == 'session/update':
            u=value['params']['update'];log.write(json.dumps(u)+'\n');log.flush()
            if u.get('sessionUpdate') == 'tool_call':
                raw=u.get('rawInput',{}); name=u.get('_meta',{}).get('x.ai/tool',{}).get('name',u.get('title'))
                if name == 'search_tool': discovery.add(u['toolCallId'])
                elif name == 'use_tool' and raw.get('tool_name') in allowed:
                    calls[u['toolCallId']]={'type':'mcp_tool_call','server':'pre-drink','tool':raw['tool_name'].split('__',1)[1],'arguments':raw['tool_input']}
                else: raise RuntimeError('Unexpected tool requested by Grok')
                if len(calls)>25 or len(discovery)>8: raise RuntimeError('Fictional tool-call limit exceeded')
            if u.get('sessionUpdate') == 'tool_call_update' and u.get('status') in {'completed','failed'} and u['toolCallId'] in calls:
                item=calls[u['toolCallId']];out=u.get('rawOutput',{})
                assert out.get('type')=='MCP' and out.get('server_name')=='pre_drink'
                body=out.get('output',{})
                item['status']='failed' if out.get('is_error') else 'completed'
                item['result']={'content':[{'type':'text','text':body.get('OkayOutput',body.get('Error'))}]}
                normalized.write(json.dumps({'type':'item.completed','item':item})+'\n');normalized.flush()
        elif value.get('id') == ident:
            if 'error' in value: raise RuntimeError(value['error'].get('message','Grok error'))
            return value.get('result',{})

try:
    init=request('initialize',{'protocolVersion':1,'clientCapabilities':{'fs':{'readTextFile':False,'writeTextFile':False},'terminal':False}})
    methods={m['id'] for m in init.get('authMethods',[])}
    method='cached_token' if 'cached_token' in methods else 'grok.com' if 'grok.com' in methods else None
    if not method: raise RuntimeError('Existing account authentication unavailable')
    request('authenticate',{'methodId':method,'_meta':{'headless':True}})
    session=request('session/new',{'cwd':str(root/'work'),'mcpServers':[{'name':'pre_drink','command':config['command'],'args':config['args'],'env':[{'name':k,'value':v} for k,v in config['env'].items()]}]})
    result=request('session/prompt',{'sessionId':session['sessionId'],'prompt':[{'type':'text','text':prompt}]},180)
    assert result.get('stopReason')=='end_turn' and calls and all('status' in c for c in calls.values())
    normalized.write(json.dumps({'type':'turn.completed'})+'\n');normalized.flush()
    print(json.dumps({'phase':args.phase,'completed_mcp_calls':len(calls),'discovery_calls':len(discovery),'stop_reason':result['stopReason']}))
finally:
    if process.poll() is None:
        os.killpg(process.pid,signal.SIGTERM)
        try: process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid,signal.SIGKILL);process.wait(timeout=5)
    log.close();normalized.close()
