#!/usr/bin/env python3
"""Generate deterministic benchmark expectations from the frozen Python source."""
import argparse
import dataclasses
import hashlib
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--source',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();source=a.source.resolve()
baseline=json.loads((ROOT/'docs/source-baseline.json').read_text())['sources'][0]
revision=subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD'],text=True).strip()
if revision != baseline['revision']:
    p.error('reference revision differs from pinned baseline')
if subprocess.check_output(['git','-C',str(source),'status','--porcelain','--untracked-files=no'],text=True):
    p.error('tracked reference files have local changes')
for entry in baseline['files']:
    if hashlib.sha256((source/entry['path']).read_bytes()).hexdigest() != entry['sha256']:
        p.error('reference source hash mismatch: '+entry['path'])
sys.path.insert(0,str(source/'src'))
from semantic_operators.types import Boolean,Choice,Score,make_answer
from semantic_operators import bench
from semantic_operators.flows import FlowResult,Step
questions={'b':Boolean('q'),'c':Choice('q',{'a':None,'b':None}),'s':Score('q',['low','middle','high'])}
labels=[{'b':True,'c':'b','s':2},{'b':True,'c':'a','s':1},{'c':'a'},{}]
specs=[
    {'b':[True,{'true':.9,'false':.1}],'c':['a',{'a':.8,'b':.2}],'s':[1.5,{'low':0,'middle':.5,'high':.5}]},
    {'b':[False,{'true':.1,'false':.9}],'c':['a',{'a':.5,'b':.5}],'s':[1.3,{'low':.2,'middle':.3,'high':.5}]},
    {'b':[False,{'true':.5,'false':.5}],'c':['a',{'a':.9,'b':.1}],'s':[0.,{'low':1,'middle':0,'high':0}]},
    {'b':[True,{'true':1.,'false':0.}],'c':['b',{'a':0.,'b':1.}],'s':[2.,{'low':0,'middle':0,'high':1}]},
]
cases=[bench.Case(f'synthetic-{i}',label) for i,label in enumerate(labels)]
answers=[{name:make_answer(questions[name],value,probabilities) for name,(value,probabilities) in spec.items()} for spec in specs]
report=bench.score(questions,cases,answers,[1.,2.,3.,4.],9.)
def output(r):
    return {'questions':{name:{'correct':s.correct,'undecided':s.undecided,'total':s.total,'answered':s.answered,'accuracy':s.accuracy,'accuracy_when_answered':s.accuracy_when_answered,'mean_p_correct':s.mean_p_correct,'p_correct':s.p_correct} for name,s in r.questions.items()},'misses':[{'case':m.case,'question':m.question,'expected':m.expected} for m in r.misses],'decisions':r.decisions,'latencies_ms':r.latencies_ms,'total_ms':r.total_ms}
thresholds=[{'threshold':v,'expected':output(bench.at_min_confidence(report,questions,cases,v))} for v in [0.,.5,.8,.9,1.]]
strict=bench.at_min_confidence(report,questions,cases,.95)
restored=bench.at_min_confidence(strict,questions,cases,0.)
ndcg=[]
for order,grades,k in [(['a','b','c'],{'a':3,'b':2,'c':0},10),(['b','a','missing'],{'a':3,'b':2,'c':0},2),([],{'a':3},10),(['unknown'],{},10),(['a'],{'a':0},10),(['a'],{'a':3},0),(['a','a'],{'a':3,'b':2},10)]:
    ndcg.append({'order':order,'grades':grades,'k':k,'expected':bench.ndcg(order,grades,k)})
flow_results=[FlowResult('yes',None,[Step('synthetic',{})]),FlowResult(None,'q',[Step('synthetic',{}),Step('synthetic',{})]),FlowResult('no',None,[])]
flow=bench.FlowReport(flow_results,['yes','yes','yes'],[1.,2.,3.])
flow_expected={'total':flow.total,'stopped':flow.stopped,'correct':flow.correct,'misses':flow.misses,'accuracy_when_answered':flow.accuracy_when_answered,'calls':flow.calls}
result={'reference_revision':revision,'python_version':sys.version.split()[0],'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'labels':labels,'answers':specs,'report':output(report),'thresholds':thresholds,'no_restore':output(restored),'stability':bench.stability([report,strict]),'ndcg':ndcg,'flow':flow_expected}
a.output.write_text(json.dumps(result,indent=2)+'\n')
print('Exported benchmark report, threshold, stability, NDCG and flow fixtures')
