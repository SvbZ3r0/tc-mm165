#!/usr/bin/env python3
from pathlib import Path

import sys
source = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('Battleships.rs')
out = Path(sys.argv[2]) if len(sys.argv) > 2 else Path('tmp_trace/Battleships_trace.rs')
s = source.read_text()
s = s.replace('use std::io::{self, BufRead, Write};', 'use std::fs::File;\nuse std::io::{self, BufRead, Write};', 1)
helpers = r'''
fn diag_log(diag: &mut Option<File>, msg: &str) {
    if let Some(file) = diag.as_mut() {
        let _ = writeln!(file, "{}", msg);
    }
}

fn diag_top_hunt(
    diag: &mut Option<File>,
    turn: usize,
    candidates: &[(usize, usize, f64, f64, f64, f64)],
) {
    if diag.is_none() {
        return;
    }
    for (rank, &(r, c, score, prob, density, center)) in candidates.iter().take(5).enumerate() {
        diag_log(
            diag,
            &format!(
                "turn={} mode=HUNT rank={} cell={},{} score={:.6} prob={:.6} density={:.6} center={:.6}",
                turn, rank + 1, r, c, score, prob, density, center
            ),
        );
    }
}
'''
s = s.replace('\nfn main() {', '\n' + helpers + '\nfn main() {', 1)
s = s.replace('''fn best_hunt_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
    scans: &[Scan],
    remaining_cells: usize,
    rng: &mut u64,
) -> Option<(usize, usize)> {''', '''fn best_hunt_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
    scans: &[Scan],
    remaining_cells: usize,
    rng: &mut u64,
    turn: usize,
    diag: &mut Option<File>,
) -> Option<(usize, usize)> {''', 1)
s = s.replace('    let mut best = None;\n    let mut best_score = -1.0f64;', '    let mut best = None;\n    let mut best_score = -1.0f64;\n    let mut diag_candidates: Vec<(usize, usize, f64, f64, f64, f64)> = Vec::new();', 1)
s = s.replace('''            let score = prob[r][c] * density_scale + center_bias + jitter;
            if score > best_score {''', '''            let score = prob[r][c] * density_scale + center_bias + jitter;
            if diag.is_some() {
                diag_candidates.push((r, c, score, prob[r][c], density_scale, center_bias));
            }
            if score > best_score {''', 1)
s = s.replace('''
    best
}

fn placement_score_for_hits''', '''
    if diag.is_some() {
        diag_candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        diag_top_hunt(diag, turn, &diag_candidates);
    }
    best
}

fn placement_score_for_hits''', 1)
s = s.replace(') -> Option<(usize, usize)> {\n    if active_hits.is_empty() {', ') -> Option<((usize, usize), f64)> {\n    if active_hits.is_empty() {', 1)
s = s.replace('    Some((candidates[0].0, candidates[0].1))\n}\n\nfn best_chase_cell', '    Some(((candidates[0].0, candidates[0].1), candidates[0].2))\n}\n\nfn best_chase_cell', 1)
s = s.replace('''fn best_chase_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    active_clusters: &[Vec<(usize, usize)>],
    remaining: &[usize],
    rng: &mut u64,
) -> Option<((usize, usize), usize)> {''', '''fn best_chase_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    active_clusters: &[Vec<(usize, usize)>],
    remaining: &[usize],
    rng: &mut u64,
    turn: usize,
    diag: &mut Option<File>,
) -> Option<((usize, usize), usize)> {''', 1)
s = s.replace('''        if let Some((r, c)) = chase_cell(n, grid, shot, cluster, remaining, rng) {
            let score = (cluster.len() as f64) * 100000.0 + prob[r][c];
            if best.map_or(true, |(_, _, best_score)| score > best_score) {
                best = Some(((r, c), idx, score));
            }
        }''', '''        if let Some(((r, c), chase_score)) = chase_cell(n, grid, shot, cluster, remaining, rng) {
            let score = (cluster.len() as f64) * 100000.0 + prob[r][c];
            diag_log(diag, &format!("turn={} mode=CHASE cluster={} cluster_len={} cell={},{} score={:.6} chase_score={:.6} prob={:.6}", turn, idx, cluster.len(), r, c, score, chase_score, prob[r][c]));
            if best.map_or(true, |(_, _, best_score)| score > best_score) {
                best = Some(((r, c), idx, score));
            }
        }''', 1)
s = s.replace('    let mut stdout = io::stdout();', '    let mut stdout = io::stdout();\n    let mut diag: Option<File> = std::env::var("BS_DIAG").ok().and_then(|p| File::create(p).ok());', 1)
s = s.replace('    let mut rng = 0x9E37_79B9_7F4A_7C15u64 ^ ((n as u64) << 32) ^ s as u64;', '    let mut rng = 0x9E37_79B9_7F4A_7C15u64 ^ ((n as u64) << 32) ^ s as u64;\n    let mut turn = 0usize;', 1)
s = s.replace('    while alive > 0 {', '    while alive > 0 {\n        turn += 1;', 1)
s = s.replace('''            println!("SCAN {} {} {} {}", scan.r1, scan.c1, scan.r2, scan.c2);
            stdout.flush().unwrap();
            scan.count = read_line(&mut reader);''', '''            diag_log(&mut diag, &format!("turn={} mode=SCAN region={},{},{},{}", turn, scan.r1, scan.c1, scan.r2, scan.c2));
            println!("SCAN {} {} {} {}", scan.r1, scan.c1, scan.r2, scan.c2);
            stdout.flush().unwrap();
            scan.count = read_line(&mut reader);
            diag_log(&mut diag, &format!("turn={} result=SCAN count={}", turn, scan.count));''', 1)
s = s.replace('''        let target = best_chase_cell(n, &grid, &shot, &active_clusters, &remaining, &mut rng)
            .map(|(cell, cluster_idx)| (cell, Some(cluster_idx)))
            .or_else(|| best_hunt_cell(n, &grid, &shot, &remaining, &scans, remaining_cells, &mut rng).map(|cell| (cell, None)))''', '''        let target = best_chase_cell(n, &grid, &shot, &active_clusters, &remaining, &mut rng, turn, &mut diag)
            .map(|(cell, cluster_idx)| (cell, Some(cluster_idx)))
            .or_else(|| best_hunt_cell(n, &grid, &shot, &remaining, &scans, remaining_cells, &mut rng, turn, &mut diag).map(|cell| (cell, None)))''', 1)
s = s.replace('''        let ((r, c), target_cluster) = target;
        shot[r][c] = true;''', '''        let ((r, c), target_cluster) = target;
        diag_log(&mut diag, &format!("turn={} picked cell={},{} target_cluster={:?} active_clusters={} remaining_cells={} scans={}", turn, r, c, target_cluster, active_clusters.len(), remaining_cells, scans.len()));
        shot[r][c] = true;''', 1)
s = s.replace('''        let result: String = read_line(&mut reader);
        match result.as_str() {''', '''        let result: String = read_line(&mut reader);
        diag_log(&mut diag, &format!("turn={} result={}", turn, result));
        match result.as_str() {''', 1)
s = s.replace('''                        let (killed_hits, inferred_len) = infer_killed_hits(n, &remaining, &killed_cluster, (r, c));
                        mark_active_dead(&mut grid, &killed_hits, (r, c));''', '''                        let (killed_hits, inferred_len) = infer_killed_hits(n, &remaining, &killed_cluster, (r, c));
                        diag_log(&mut diag, &format!("turn={} kill_infer inferred_len={} committed_hits={} cluster_hits={}", turn, inferred_len, killed_hits.len(), killed_cluster.len()));
                        mark_active_dead(&mut grid, &killed_hits, (r, c));''', 1)
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(s)
