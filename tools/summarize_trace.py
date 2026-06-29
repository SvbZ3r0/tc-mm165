#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

for path_arg in sys.argv[1:]:
    path = Path(path_arg)
    lines = path.read_text().splitlines()
    scans = sum('mode=SCAN' in line for line in lines)
    hunt_top = sum('mode=HUNT rank=1' in line for line in lines)
    chase = sum('mode=CHASE' in line for line in lines)
    kills = [line for line in lines if 'kill_infer' in line]
    ambiguous = []
    for line in kills:
        m = re.search(r'inferred_len=(\d+) committed_hits=(\d+) cluster_hits=(\d+)', line)
        if m:
            inferred, committed, cluster = map(int, m.groups())
            if committed + 1 < inferred or committed < cluster:
                ambiguous.append(line)
    miss_streak = 0
    max_miss_streak = 0
    for line in lines:
        if 'result=MISS' in line:
            miss_streak += 1
            max_miss_streak = max(max_miss_streak, miss_streak)
        elif 'result=HIT' in line or 'result=KILL' in line:
            miss_streak = 0
    picked = sum('picked cell=' in line for line in lines)
    print(f'{path.name}\tturns={picked+scans}\tshots={picked}\tscans={scans}\thunt_top={hunt_top}\tchase_options={chase}\tkills={len(kills)}\tambiguous_kills={len(ambiguous)}\tmax_miss_streak={max_miss_streak}')
    for line in ambiguous[:3]:
        print(f'  {line}')
