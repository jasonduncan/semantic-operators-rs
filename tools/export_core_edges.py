#!/usr/bin/env python3
"""Export additional synthetic numeric and wire cases from the frozen Python source."""
import argparse
import copy
import hashlib
import json
import math
import random
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--source', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
baseline = json.loads((ROOT / 'docs/source-baseline.json').read_text())['sources'][0]
source = args.source.resolve()
revision = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
if revision != baseline['revision']:
    parser.error('reference revision differs from pinned baseline')
if subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain', '--untracked-files=no'], text=True):
    parser.error('tracked reference files have local changes')
for item in baseline['files']:
    if hashlib.sha256((source/item['path']).read_bytes()).hexdigest() != item['sha256']:
        parser.error('reference source hash mismatch: '+item['path'])
sys.path.insert(0, str(source/'src'))
from semantic_operators.types import Boolean, Choice, Score, make_answer, complement
from semantic_operators.interfaces.wire import decode, encode, WireError
inputs = []
for p in [0, 0.07, 0.12345678905, 0.12345678915, 0.49999999995, 0.50000000005, 0.99999999995, 1]:
    for direction in [-math.inf, p, math.inf]:
        x = p if direction == p else math.nextafter(p, direction)
        if 0 <= x <= 1:
            inputs.append(x)
complements = [{'input': p, 'expected': complement(p)} for p in inputs]
base = {'state': 'synthetic', 'questions': [{'name': 'q', 'type': 'boolean', 'instructions': 'q'}]}
requests = []
for field, value in [('schema_version', None), ('schema_version', 1), ('request_id', 7), ('state', False)]:
    r = copy.deepcopy(base); r[field] = value; requests.append(r)
for spec in [
    {'name':'q','type':'boolean','instructions':'q','true':7},
    {'name':'q','type':'boolean','instructions':'  '},
    {'name':'q','type':'boolean','instructions':'\u001c\u001d\u001e\u001f'},
    {'name':'q','type':'score','instructions':'q','levels':['a','a']},
    {'name':'q','type':'score','instructions':'q','levels':[1,'b']},
    {'name':'q','type':'score','instructions':'q','levels':[]},
    {'name':'q','type':'choice','instructions':'q','options':['a','a']},
    {'name':'q','type':'choice','instructions':'q','options':[]},
    {'name':'q','type':'choice','instructions':'q','options':[{'name':'a','typo':1},'b']},
    {'name':'q','type':'choice','instructions':'q','options':[{'name':'a','description':7},'b']},
]:
    requests.append({'state':'synthetic','questions':[spec]})
wire = []
for request in requests:
    try:
        decode(request)
    except WireError as error:
        wire.append({'request':request,'path':error.path,'message':str(error)})
encodings = []
for p in [0.012345,0.00012345,0.12345,0.12355,0.50005,0.99995,0.0,1.0]:
    question = Boolean('q')
    probs = {'true':p,'false':complement(p)}
    answer = make_answer(question,p>0.5,probs)
    encodings.append({'probabilities':probs,'value':p>0.5,'expected':encode({'q':answer},{'q':question})})
normalization = []
for n in [3, 7, 17]:
    options = {f'level-{i}': None for i in range(n)}
    question = Choice('q', options)
    tail = (1 + 0.005*n + 1e-9) - 0.03125*(n-1)
    if not 0 <= tail <= 1:
        continue
    for x in [math.nextafter(tail,-math.inf), tail, math.nextafter(tail,math.inf)]:
        probs = dict.fromkeys(options, 0.03125)
        probs[f'level-{n-1}'] = x
        record = {'question':{'type':'choice','instructions':'q','options':list(options.items())}, 'value':f'level-{n-1}', 'probabilities':probs}
        try:
            a = make_answer(question,record['value'],probs)
            record['expected'] = {'value':a.value,'confidence':a.confidence,'decided':a.decided,'probabilities':a.probabilities}
        except ValueError:
            record['expected_error'] = True
        normalization.append(record)
levels = ['low','medium','high']
for center in [1.28,1.32,1.5]:
    for v in [math.nextafter(center,-math.inf),center,math.nextafter(center,math.inf)]:
        probs = {'low':0.2,'medium':0.3,'high':0.5}
        record = {'question':{'type':'score','instructions':'q','levels':levels},'value':v,'probabilities':probs}
        try:
            a = make_answer(Score('q',levels),v,probs)
            record['expected'] = {'value':a.value,'confidence':a.confidence,'decided':a.decided,'probabilities':a.probabilities}
        except ValueError:
            record['expected_error'] = True
        normalization.append(record)
rng = random.Random(17)
for n in [3,5,11,21]:
    levels = [f'level-{i}' for i in range(n)]
    for _ in range(3):
        weights = [rng.random() for _ in levels]
        probs = dict(zip(levels,[w/sum(weights) for w in weights]))
        expected = sum(i*probs[label] for i,label in enumerate(levels))
        margin = 0.005*(1+sum(range(n)))
        for center in [expected-margin,expected+margin]:
            for v in [math.nextafter(center,-math.inf),center,math.nextafter(center,math.inf)]:
                record = {'question':{'type':'score','instructions':'q','levels':levels},'value':v,'probabilities':probs}
                try:
                    a = make_answer(Score('q',levels),v,probs)
                    record['expected'] = {'value':a.value,'confidence':a.confidence,'decided':a.decided,'probabilities':a.probabilities}
                except ValueError:
                    record['expected_error'] = True
                normalization.append(record)
result = {'normalization_cases' :normalization,'reference_revision' :revision,'python_version':sys.version.split()[0],'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'complement_cases':complements,'wire_errors':wire,'wire_encodings':encodings}
args.output.write_text(json.dumps(result,indent=2)+'\n')
print(f'Wrote {len(complements)+len(wire)+len(encodings)+len(normalization)} Python edge cases')
