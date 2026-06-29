
from pathlib import Path
import csv, re, math
from collections import Counter
KV=re.compile(r"([A-Za-z_]+)=([^\s]+)")

def parse(path):
    decisions=[]; results=[]; game={}
    for line in path.read_text(errors='ignore').splitlines():
        d=dict(KV.findall(line))
        if line.startswith('GAME'): game=d
        elif 'mode=' in line and 'turn=' in line and 'cell=' in line:
            d['raw']=line; decisions.append(d)
        elif line.startswith('RESULT'):
            d['raw']=line; results.append(d)
    decisions.sort(key=lambda x:int(x.get('turn','0')))
    results.sort(key=lambda x:int(x.get('turn','0')))
    return game,decisions,results

def first_div(a,b):
    for da,db in zip(a,b):
        if da.get('cell')!=db.get('cell') or da.get('mode')!=db.get('mode'):
            return int(da['turn']),da,db
    return None,None,None

def res_map(results): return {int(r['turn']):r for r in results if 'turn' in r}

def early_features(decisions,results,start,window=20):
    rm=res_map(results)
    ds=[d for d in decisions if start <= int(d['turn']) < start+window]
    rs=[rm.get(int(d['turn']),{}).get('result','') for d in ds]
    modes=[d.get('mode','') for d in ds]
    first_non_chase=None
    for d in ds:
        if d.get('mode')!='CHASE':
            first_non_chase=int(d['turn'])-start; break
    return {
        'w20_chase':sum(1 for m in modes if m=='CHASE'),
        'w20_hunt':sum(1 for m in modes if m=='HUNT'),
        'w20_scan':sum(1 for m in modes if m=='SCAN'),
        'w20_hit':sum(1 for x in rs if x=='HIT'),
        'w20_miss':sum(1 for x in rs if x=='MISS'),
        'w20_kill':sum(1 for x in rs if x=='KILL'),
        'first_result':rs[0] if rs else '',
        'first4_results':','.join(rs[:4]),
        'first8_results':','.join(rs[:8]),
        'first_non_chase_offset':first_non_chase if first_non_chase is not None else 999,
    }

deltas={}
with Path('diagnostics/iter21_vs_iter18_seed_deltas.tsv').open() as f:
    for r in csv.DictReader(f,delimiter='\t'):
        deltas[int(r['seed'])]=float(r['delta_variant_minus_baseline'])
groups={}
for g,file in [('loss','diagnostics/iter21_top_loss_seeds.txt'),('win','diagnostics/iter21_top_win_seeds.txt')]:
    for x in Path(file).read_text().split(): groups[int(x)]=g
rows=[]
for seed,group in sorted(groups.items()):
    g18,d18,r18=parse(Path(f'diagnostics/traces_iter18_vs_iter21_plus/iter18/seed_{seed}.trace'))
    g21,d21,r21=parse(Path(f'diagnostics/traces_iter18_vs_iter21_plus/iter21/seed_{seed}.trace'))
    turn,dec18,dec21=first_div(d18,d21)
    f21=early_features(d21,r21,turn,20)
    f18=early_features(d18,r18,turn,20)
    row={'group':group,'seed':seed,'delta':deltas[seed],'n':g21.get('n',''),'p':g21.get('p',''),'turn':turn,'iter21_cell':dec21.get('cell',''),'iter18_cell':dec18.get('cell','')}
    for k,v in f21.items(): row['i21_'+k]=v
    for k,v in f18.items(): row['i18_'+k]=v
    rows.append(row)
header=list(rows[0].keys())
out=Path('diagnostics/iter21_top_swing_early_window.tsv')
out.write_text('\t'.join(header)+'\n'+'\n'.join('\t'.join(str(r[h]) for h in header) for r in rows)+'\n')
# best predicates
preds=[]
def score(pred):
    l=sum(1 for r in rows if r['group']=='loss' and pred(r)); w=sum(1 for r in rows if r['group']=='win' and pred(r)); return abs(l-w), l, w
for k in header:
    vals=sorted(set(str(r[k]) for r in rows))
    if len(vals)<=25:
        for v in vals:
            sep,l,w=score(lambda r,k=k,v=v: str(r[k])==v)
            if l+w>=5: preds.append((f'{k}=={v}',sep,l,w))
for k in [x for x in header if x.startswith('i21_w20') or x.startswith('i18_w20') or x.endswith('offset') or x in ['n','p','turn']]:
    nums=[]
    for r in rows:
        try: nums.append(float(r[k]))
        except: pass
    for t in sorted(set(nums)):
        sep,l,w=score(lambda r,k=k,t=t: float(r[k])<=t)
        if l+w>=5: preds.append((f'{k}<={t:g}',sep,l,w))
        sep,l,w=score(lambda r,k=k,t=t: float(r[k])>=t)
        if l+w>=5: preds.append((f'{k}>={t:g}',sep,l,w))
preds.sort(key=lambda x:(x[1],max(x[2],x[3])), reverse=True)
Path('diagnostics/iter21_top_swing_early_window_predicates.tsv').write_text('predicate\tsep\tloss\twin\n'+'\n'.join(f'{a}\t{b}\t{c}\t{d}' for a,b,c,d in preds[:100])+'\n')
print(out)
print('diagnostics/iter21_top_swing_early_window_predicates.tsv')
