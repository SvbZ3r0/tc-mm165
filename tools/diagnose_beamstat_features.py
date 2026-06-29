
from pathlib import Path
import csv,re,math
from collections import Counter
KV=re.compile(r"([A-Za-z_]+)=([^\s]+)")

def parse_beamstats(path):
    rows=[]
    for line in path.read_text(errors='ignore').splitlines():
        if line.startswith('BEAMSTAT'):
            d=dict(KV.findall(line)); d['raw']=line; rows.append(d)
    return rows

def first_div_turn(seed):
    import csv
    with Path('diagnostics/iter21_vs_iter18_trace_divergence.tsv').open() as f:
        for r in csv.DictReader(f,delimiter='\t'):
            if int(r['seed'])==seed:
                return int(r['divergence_turn'])
    return 999999

def fnum(d,k,default=math.nan):
    try: return float(d.get(k,''))
    except: return default

groups={}
for g,file in [('loss','diagnostics/iter21_top_loss_seeds.txt'),('win','diagnostics/iter21_top_win_seeds.txt')]:
    for x in Path(file).read_text().split(): groups[int(x)]=g
deltas={}
with Path('diagnostics/iter21_vs_iter18_seed_deltas.tsv').open() as f:
    for r in csv.DictReader(f,delimiter='\t'):
        deltas[int(r['seed'])]=float(r['delta_variant_minus_baseline'])
rows=[]
for seed,group in sorted(groups.items()):
    stats=parse_beamstats(Path(f'diagnostics/traces_iter21_beamstat/seed_{seed}.trace'))
    div=first_div_turn(seed)
    # Use first beamstat, and last beamstat before first divergence if present.
    chosen=[]
    if stats: chosen.append(('first',stats[0]))
    if stats:
        # no turn in beamstat, so use first for now; best_hunt calls before divergence are usually first.
        chosen.append(('prediv',stats[0]))
    for label,d in chosen[:1]:
        heat_best=fnum(d,'heat_best'); heat_second=fnum(d,'heat_second')
        beam_best=fnum(d,'beam_best'); beam_second=fnum(d,'beam_second')
        heat_ratio=heat_best/max(heat_second,1e-9)
        beam_ratio=beam_best/max(beam_second,1e-9)
        rows.append({
            'group':group,'seed':seed,'delta':deltas[seed],'div_turn':div,
            'states':int(float(d.get('states','0'))),'remaining_cells':int(float(d.get('remaining_cells','0'))),'n':int(float(d.get('n','0'))),
            'heat_cell':d.get('heat_cell',''),'beam_cell':d.get('beam_cell',''),
            'same_cell':str(d.get('heat_cell','')==d.get('beam_cell','')),
            'heat_best':heat_best,'heat_second':heat_second,'heat_ratio':heat_ratio,'heat_gap':heat_best-heat_second,
            'beam_best':beam_best,'beam_second':beam_second,'beam_ratio':beam_ratio,'beam_gap':beam_best-beam_second,
            'beam_minus_heat_norm':beam_best-heat_best,
            'raw':d.get('raw','')
        })
header=list(rows[0].keys())
out=Path('diagnostics/iter21_beamstat_features.tsv')
out.write_text('\t'.join(header)+'\n'+'\n'.join('\t'.join(str(r[h]) for h in header) for r in rows)+'\n')
# predicate ranking
preds=[]
def score(pred):
    l=sum(1 for r in rows if r['group']=='loss' and pred(r)); w=sum(1 for r in rows if r['group']=='win' and pred(r)); return abs(l-w),l,w
for k in ['states','remaining_cells','n','heat_ratio','heat_gap','beam_ratio','beam_gap','beam_best','beam_second','beam_minus_heat_norm']:
    vals=sorted(set(float(r[k]) for r in rows))
    for t in vals:
        sep,l,w=score(lambda r,k=k,t=t: float(r[k])<=t)
        if l+w>=5: preds.append((f'{k}<={t:.6g}',sep,l,w))
        sep,l,w=score(lambda r,k=k,t=t: float(r[k])>=t)
        if l+w>=5: preds.append((f'{k}>={t:.6g}',sep,l,w))
for k in ['same_cell','heat_cell','beam_cell']:
    for v in sorted(set(str(r[k]) for r in rows)):
        sep,l,w=score(lambda r,k=k,v=v: str(r[k])==v)
        if l+w>=5: preds.append((f'{k}=={v}',sep,l,w))
preds.sort(key=lambda x:(x[1],max(x[2],x[3])), reverse=True)
Path('diagnostics/iter21_beamstat_predicates.tsv').write_text('predicate\tsep\tloss\twin\n'+'\n'.join(f'{a}\t{b}\t{c}\t{d}' for a,b,c,d in preds[:100])+'\n')
# averages
summary=[]
for group in ['loss','win']:
    sub=[r for r in rows if r['group']==group]
    summary.append(f'[{group}]')
    for k in ['states','remaining_cells','n','heat_ratio','heat_gap','beam_ratio','beam_gap','beam_best','beam_second','beam_minus_heat_norm']:
        vals=[float(r[k]) for r in sub]
        summary.append(f'{k}\tavg={sum(vals)/len(vals):.6f}\tmin={min(vals):.6f}\tmax={max(vals):.6f}')
    summary.append(f"same_cell\t{Counter(r['same_cell'] for r in sub)}")
Path('diagnostics/iter21_beamstat_features.tsv.summary').write_text('\n'.join(summary)+'\n')
print(out)
print('diagnostics/iter21_beamstat_predicates.tsv')
