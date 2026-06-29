#!/usr/bin/env python3
from __future__ import annotations

import csv
import re
from collections import Counter, defaultdict
from pathlib import Path

KV = re.compile(r"([A-Za-z_]+)=([^\s]+)")


def parse(path: Path):
    game = {}
    decisions = []
    results = []
    for line in path.read_text(errors="ignore").splitlines():
        d = dict(KV.findall(line))
        if line.startswith("GAME"):
            game = d
        elif "mode=" in line and "turn=" in line and "cell=" in line:
            d["raw"] = line
            decisions.append(d)
        elif line.startswith("RESULT"):
            d["raw"] = line
            results.append(d)
    decisions.sort(key=lambda x: int(x.get("turn", "0")))
    results.sort(key=lambda x: int(x.get("turn", "0")))
    return game, decisions, results


def first_divergence(a, b):
    for da, db in zip(a, b):
        if da.get("cell") != db.get("cell") or da.get("mode") != db.get("mode"):
            return int(da.get("turn", db.get("turn", "0"))), da, db
    return None, None, None


def result_at(results, turn):
    for r in results:
        if int(r.get("turn", "-1")) == turn:
            return r
    return {}


def result_seq(results, start, count=8):
    vals=[]
    for r in results:
        t=int(r.get("turn","-1"))
        if t >= start:
            vals.append(r.get("result",""))
            if len(vals)>=count: break
    return ",".join(vals)


def chase_run(decisions, results, start_turn):
    length=0
    hits=0
    misses=0
    kills=0
    for d in decisions:
        t=int(d.get("turn","-1"))
        if t < start_turn: continue
        if d.get("mode") != "CHASE":
            if length>0: break
            continue
        length += 1
        rr=result_at(results,t).get("result","")
        if rr == "HIT": hits += 1
        elif rr == "MISS": misses += 1
        elif rr == "KILL": kills += 1
    return length,hits,misses,kills


def p_bucket(p):
    try: x=float(p)
    except Exception: return 'p_unknown'
    if x < 0.15: return 'p_<0.15'
    if x < 0.25: return 'p_0.15_0.25'
    if x < 0.35: return 'p_0.25_0.35'
    return 'p_>=0.35'


def n_bucket(n):
    try: x=int(n)
    except Exception: return 'n_unknown'
    if x <= 10: return 'n_<=10'
    if x <= 15: return 'n_11_15'
    return 'n_16_20'


def main():
    rows=[]
    deltas={}
    with Path('diagnostics/iter21_vs_iter18_seed_deltas.tsv').open() as f:
        for r in csv.DictReader(f, delimiter='\t'):
            deltas[int(r['seed'])]=float(r['delta_variant_minus_baseline'])
    groups={}
    for g,file in [('top_loss','diagnostics/iter21_top_loss_seeds.txt'),('top_win','diagnostics/iter21_top_win_seeds.txt')]:
        for x in Path(file).read_text().split(): groups[int(x)]=g
    for seed,group in sorted(groups.items()):
        g18,d18,r18=parse(Path(f'diagnostics/traces_iter18_vs_iter21_plus/iter18/seed_{seed}.trace'))
        g21,d21,r21=parse(Path(f'diagnostics/traces_iter18_vs_iter21_plus/iter21/seed_{seed}.trace'))
        turn, dec18, dec21=first_divergence(d18,d21)
        res18=result_at(r18,turn) if turn is not None else {}
        res21=result_at(r21,turn) if turn is not None else {}
        cr21=chase_run(d21,r21,turn or 0)
        cr18=chase_run(d18,r18,turn or 0)
        rows.append({
            'group': group,
            'seed': seed,
            'delta': deltas[seed],
            'n': g21.get('n',''),
            'p': g21.get('p',''),
            'n_bucket': n_bucket(g21.get('n','')),
            'p_bucket': p_bucket(g21.get('p','')),
            'turn': turn if turn is not None else '',
            'iter21_mode': (dec21 or {}).get('mode',''),
            'iter18_mode': (dec18 or {}).get('mode',''),
            'iter21_cell': (dec21 or {}).get('cell',''),
            'iter18_cell': (dec18 or {}).get('cell',''),
            'iter21_result': res21.get('result',''),
            'iter18_result': res18.get('result',''),
            'iter21_seq8': result_seq(r21,turn or 0,8),
            'iter18_seq8': result_seq(r18,turn or 0,8),
            'iter21_chase_len': cr21[0],
            'iter21_chase_hits': cr21[1],
            'iter21_chase_misses': cr21[2],
            'iter21_chase_kills': cr21[3],
            'iter18_chase_len': cr18[0],
            'iter18_chase_hits': cr18[1],
            'iter18_chase_misses': cr18[2],
            'iter18_chase_kills': cr18[3],
            'iter21_raw': (dec21 or {}).get('raw',''),
            'iter18_raw': (dec18 or {}).get('raw',''),
        })
    out=Path('diagnostics/iter21_top_swing_features.tsv')
    header=list(rows[0].keys())
    with out.open('w') as f:
        f.write('\t'.join(header)+'\n')
        for row in rows:
            f.write('\t'.join(str(row[h]).replace('\t',' ') for h in header)+'\n')
    summary=Path('diagnostics/iter21_top_swing_features.tsv.summary')
    with summary.open('w') as f:
        for group in ['top_loss','top_win']:
            f.write(f'[{group}]\n')
            subset=[r for r in rows if r['group']==group]
            for key in ['n_bucket','p_bucket','turn','iter21_result','iter18_result','iter21_chase_len','iter21_chase_misses','iter21_chase_kills']:
                c=Counter(str(r[key]) for r in subset)
                f.write(key+'\t'+str(dict(c.most_common()))+'\n')
            avg=lambda k: sum(float(r[k]) for r in subset)/len(subset)
            f.write(f"avg_chase_len\t{avg('iter21_chase_len'):.3f}\n")
            f.write(f"avg_chase_misses\t{avg('iter21_chase_misses'):.3f}\n")
            f.write(f"avg_chase_kills\t{avg('iter21_chase_kills'):.3f}\n\n")
    print(out)
    print(summary)

if __name__=='__main__': main()
