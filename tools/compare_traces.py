#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

PICK_RE = re.compile(r'turn=(\d+) picked cell=(\d+),(\d+) target_cluster=([^ ]+) active_clusters=(\d+) remaining_cells=(\d+) scans=(\d+)')
RESULT_RE = re.compile(r'turn=(\d+) result=(MISS|HIT|KILL)')
SCAN_RE = re.compile(r'turn=(\d+) mode=SCAN')

def parse(path: Path):
    picks = []
    results = {}
    scans = set()
    for line in path.read_text().splitlines():
        m = PICK_RE.search(line)
        if m:
            turn, r, c, cluster, active, rem, scans_count = m.groups()
            picks.append({
                'turn': int(turn),
                'cell': (int(r), int(c)),
                'cluster': cluster,
                'active': int(active),
                'remaining': int(rem),
                'scans': int(scans_count),
            })
        m = RESULT_RE.search(line)
        if m:
            results[int(m.group(1))] = m.group(2)
        m = SCAN_RE.search(line)
        if m:
            scans.add(int(m.group(1)))
    return picks, results, scans

for seed in sys.argv[1:]:
    a = Path(f'diagnostics/traces_iter18_20260626/seed_{seed}.trace')
    b = Path(f'diagnostics/traces_iter16_20260626/seed_{seed}.trace')
    p18, r18, s18 = parse(a)
    p16, r16, s16 = parse(b)
    first = None
    for i, (x, y) in enumerate(zip(p18, p16), 1):
        if x['cell'] != y['cell'] or x['cluster'] != y['cluster']:
            first = (i, x, y)
            break
    if first is None:
        first = (min(len(p18), len(p16)) + 1, None, None)
    print(f'seed={seed} iter18_shots={len(p18)} iter16_shots={len(p16)} first_pick_divergence_index={first[0]}')
    if first[1]:
        i, x, y = first
        print(f'  iter18 turn={x["turn"]} cell={x["cell"]} cluster={x["cluster"]} active={x["active"]} rem={x["remaining"]} prev_result={r18.get(x["turn"]-1)}')
        print(f'  iter16 turn={y["turn"]} cell={y["cell"]} cluster={y["cluster"]} active={y["active"]} rem={y["remaining"]} prev_result={r16.get(y["turn"]-1)}')
    # first turn where result sequence differs for corresponding shot turns
    diffs = []
    for x, y in zip(p18[:20], p16[:20]):
        rx = r18.get(x['turn'])
        ry = r16.get(y['turn'])
        if rx != ry:
            diffs.append((x['turn'], rx, y['turn'], ry))
            break
    if diffs:
        print(f'  first_result_diff iter18_turn={diffs[0][0]} {diffs[0][1]} iter16_turn={diffs[0][2]} {diffs[0][3]}')
