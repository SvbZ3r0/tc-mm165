use std::io::{self, BufRead, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Unknown,
    Miss,
    Hit,
    Dead,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScanKind {
    TopHalf,
    LeftHalf,
    TopLeft,
    Other,
}

#[derive(Clone, Copy)]
struct Scan {
    r1: usize,
    c1: usize,
    r2: usize,
    c2: usize,
    count: usize,
    kind: ScanKind,
}

impl Scan {
    fn contains(&self, r: usize, c: usize) -> bool {
        self.r1 <= r && r <= self.r2 && self.c1 <= c && c <= self.c2
    }
}

fn diag_extra_line(line: &str) {
    if let Ok(path) = std::env::var("BS_DIAG") {
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{}", line);
        }
    }
}

fn read_line<T: std::str::FromStr>(reader: &mut io::Lines<io::BufReader<io::Stdin>>) -> T {
    reader
        .next()
        .expect("Unexpected end of input")
        .expect("Failed to read line")
        .trim()
        .parse::<T>()
        .ok()
        .expect("Failed to parse input")
}

fn inside(n: usize, r: isize, c: isize) -> bool {
    r >= 0 && c >= 0 && (r as usize) < n && (c as usize) < n
}

fn decrement_ship_count(remaining: &mut [usize], len: usize) {
    if len < remaining.len() && remaining[len] > 0 {
        remaining[len] -= 1;
        return;
    }

    let mut best = 0usize;
    let mut best_dist = usize::MAX;
    for l in 1..remaining.len() {
        if remaining[l] == 0 {
            continue;
        }
        let d = if l > len { l - len } else { len - l };
        if d < best_dist {
            best_dist = d;
            best = l;
        }
    }
    if best > 0 {
        remaining[best] -= 1;
    }
}

fn placement_is_legal(
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    r: usize,
    c: usize,
    dr: usize,
    dc: usize,
    len: usize,
) -> bool {
    for k in 0..len {
        let rr = r + dr * k;
        let cc = c + dc * k;
        let cell = grid[rr][cc];
        if cell == Cell::Miss || cell == Cell::Dead || (shot[rr][cc] && cell != Cell::Hit) {
            return false;
        }
    }
    true
}

const HEATMAP_ALPHA: f64 = 0.50;
const BEAM_WIDTH_NO_HITS: usize = 180;
const BEAM_WIDTH_WITH_HITS: usize = 260;
const BEAM_PLACEMENT_LIMIT_NO_HITS: usize = 260;
const BEAM_PLACEMENT_LIMIT_WITH_HITS: usize = 420;
const BEAM_MIN_VALID_STATES: usize = 8;
const BEAM_RERANK_TOP_K: usize = 5;
const BEAM_RERANK_FACTOR: f64 = 0.05;

fn build_probabilities(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
    scans: &[Scan],
    remaining_cells: usize,
) -> Vec<Vec<f64>> {
    let mut raw = vec![vec![0.0f64; n]; n];
    let mut norm = vec![vec![0.0f64; n]; n];
    let mut legal_weight_sums = vec![0.0f64; remaining.len()];

    let mut global_unknown = 0usize;
    for r in 0..n {
        for c in 0..n {
            if grid[r][c] == Cell::Unknown && !shot[r][c] {
                global_unknown += 1;
            }
        }
    }
    let global_density = remaining_cells as f64 / global_unknown.max(1) as f64;
    let density_scans = derived_quadrant_scans(n, scans, remaining_cells);
    let scan_source: Vec<Scan> = density_scans.unwrap_or_else(|| scans.to_vec());
    let mut scan_constraints = Vec::new();
    if remaining_cells > 0 && global_density > 0.0 {
        for scan in &scan_source {
            let (scan_remaining, scan_unknown) = scan_adjusted_count(grid, *scan);
            if scan_unknown == 0 {
                continue;
            }
            let scan_density = scan_remaining as f64 / scan_unknown as f64;
            let ratio = (scan_density / global_density).clamp(0.35, 2.25);
            scan_constraints.push((*scan, scan_remaining, ratio));
        }
    }

    for len in 1..remaining.len() {
        if remaining[len] == 0 {
            continue;
        }

        for r in 0..n {
            for c in 0..=n - len {
                if placement_is_legal(grid, shot, r, c, 0, 1, len) {
                    let placement_weight = placement_scan_weight(r, c, 0, 1, len, &scan_constraints);
                    legal_weight_sums[len] += placement_weight;
                }
            }
        }

        for r in 0..=n - len {
            for c in 0..n {
                if placement_is_legal(grid, shot, r, c, 1, 0, len) {
                    let placement_weight = placement_scan_weight(r, c, 1, 0, len, &scan_constraints);
                    legal_weight_sums[len] += placement_weight;
                }
            }
        }
    }

    for len in 1..remaining.len() {
        let ships = remaining[len];
        if ships == 0 || legal_weight_sums[len] <= 0.0 {
            continue;
        }
        let raw_weight = ships as f64 * len as f64;
        let norm_weight = raw_weight / legal_weight_sums[len];

        for r in 0..n {
            for c in 0..=n - len {
                if placement_is_legal(grid, shot, r, c, 0, 1, len) {
                    let placement_weight = placement_scan_weight(r, c, 0, 1, len, &scan_constraints);
                    if placement_weight <= 0.0 {
                        continue;
                    }
                    for k in 0..len {
                        raw[r][c + k] += raw_weight * placement_weight;
                        norm[r][c + k] += norm_weight * placement_weight;
                    }
                }
            }
        }

        for r in 0..=n - len {
            for c in 0..n {
                if placement_is_legal(grid, shot, r, c, 1, 0, len) {
                    let placement_weight = placement_scan_weight(r, c, 1, 0, len, &scan_constraints);
                    if placement_weight <= 0.0 {
                        continue;
                    }
                    for k in 0..len {
                        raw[r + k][c] += raw_weight * placement_weight;
                        norm[r + k][c] += norm_weight * placement_weight;
                    }
                }
            }
        }
    }

    let mut raw_max = 0.0f64;
    let mut norm_max = 0.0f64;
    for r in 0..n {
        for c in 0..n {
            raw_max = raw_max.max(raw[r][c]);
            norm_max = norm_max.max(norm[r][c]);
        }
    }

    let scale = if norm_max > 0.0 && raw_max > 0.0 {
        norm_max / raw_max
    } else {
        1.0
    };

    let mut prob = vec![vec![0.0f64; n]; n];
    for r in 0..n {
        for c in 0..n {
            prob[r][c] = HEATMAP_ALPHA * norm[r][c] + (1.0 - HEATMAP_ALPHA) * raw[r][c] * scale;
        }
    }

    prob
}

#[derive(Clone)]
struct BeamPlacement {
    index: usize,
    bits: Vec<u64>,
    scan_counts: Vec<usize>,
    prior: f64,
}

#[derive(Clone)]
struct BeamState {
    bits: Vec<u64>,
    scan_counts: Vec<usize>,
    last_len: usize,
    last_index: usize,
    score: f64,
}

// BEGIN GENERATED PLACEMENT MODEL
include!("placement_model_generated.rs");
// END GENERATED PLACEMENT MODEL


fn bit_has(bits: &[u64], idx: usize) -> bool {
    (bits[idx / 64] & (1u64 << (idx % 64))) != 0
}

fn bit_intersects(a: &[u64], b: &[u64]) -> bool {
    a.iter().zip(b.iter()).any(|(x, y)| (x & y) != 0)
}

fn bit_or_into(dst: &mut [u64], src: &[u64]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d |= *s;
    }
}

fn build_beam_placements(
    n: usize,
    len: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    scans: &[Scan],
) -> Vec<BeamPlacement> {
    let words = (n * n + 63) / 64;
    let center = (n as f64 - 1.0) * 0.5;
    let mut placements = Vec::new();

    for (raw_index, raw) in PRECOMPUTED_PLACEMENTS.iter().enumerate() {
        if raw.n as usize != n || raw.len as usize != len {
            continue;
        }

        let mut scan_counts = vec![0usize; scans.len()];
        let mut hit_count = 0usize;
        let mut prior = 0.0;
        let mut ok = true;

        for i in 0..len {
            let idx = raw.cells[i] as usize;
            let r = idx / n;
            let c = idx % n;
            let cell = grid[r][c];
            if cell == Cell::Miss || cell == Cell::Dead || (shot[r][c] && cell != Cell::Hit) {
                ok = false;
                break;
            }
            if cell == Cell::Hit {
                hit_count += 1;
            }
            for (scan_idx, scan) in scans.iter().enumerate() {
                if scan.r1 <= r && r <= scan.r2 && scan.c1 <= c && c <= scan.c2 {
                    scan_counts[scan_idx] += 1;
                }
            }
            let dr = r as f64 - center;
            let dc = c as f64 - center;
            prior -= (dr * dr + dc * dc).sqrt() * 0.001;
        }

        if ok {
            prior += hit_count as f64 * 1000.0;
            placements.push(BeamPlacement {
                index: raw_index,
                bits: raw.bits[..words].to_vec(),
                scan_counts,
                prior,
            });
        }
    }

    placements.sort_by(|a, b| b.prior.partial_cmp(&a.prior).unwrap());
    placements
}


fn beam_fleet_posterior(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
    scans: &[Scan],
    remaining_cells: usize,
) -> Option<(Vec<Vec<f64>>, usize)> {
    let known_hits: Vec<(usize, usize)> = (0..n)
        .flat_map(|r| (0..n).map(move |c| (r, c)))
        .filter(|&(r, c)| grid[r][c] == Cell::Hit)
        .collect();

    let remaining_ships: usize = remaining.iter().sum();
    let should_use_beam = remaining_ships <= 8 || !known_hits.is_empty() || remaining_cells <= n * 2;
    if !should_use_beam {
        return None;
    }
    if known_hits.is_empty() && scans.is_empty() && remaining_cells > n * 2 {
        return None;
    }

    let mut ship_lengths = Vec::new();
    for len in (1..remaining.len()).rev() {
        for _ in 0..remaining[len] {
            ship_lengths.push(len);
        }
    }
    if ship_lengths.is_empty() || ship_lengths.len() > 18 {
        return None;
    }

    let mut dead_in_scan = vec![0usize; scans.len()];
    for (idx, scan) in scans.iter().enumerate() {
        for r in scan.r1..=scan.r2 {
            for c in scan.c1..=scan.c2 {
                if grid[r][c] == Cell::Dead {
                    dead_in_scan[idx] += 1;
                }
            }
        }
        if dead_in_scan[idx] > scan.count {
            return None;
        }
    }

    let mut by_len: Vec<Vec<BeamPlacement>> = vec![Vec::new(); remaining.len()];
    for len in 1..remaining.len() {
        if remaining[len] > 0 {
            by_len[len] = build_beam_placements(n, len, grid, shot, scans);
            if by_len[len].is_empty() {
                return None;
            }
        }
    }

    let words = (n * n + 63) / 64;
    let mut states = vec![BeamState {
        bits: vec![0u64; words],
        scan_counts: vec![0usize; scans.len()],
        last_len: 0,
        last_index: 0,
        score: 0.0,
    }];
    let beam_width = if known_hits.is_empty() { BEAM_WIDTH_NO_HITS } else { BEAM_WIDTH_WITH_HITS };
    let per_len_limit = if known_hits.is_empty() { BEAM_PLACEMENT_LIMIT_NO_HITS } else { BEAM_PLACEMENT_LIMIT_WITH_HITS };

    for &len in &ship_lengths {
        let mut next_states: Vec<BeamState> = Vec::new();
        for state in &states {
            for placement in by_len[len].iter().take(per_len_limit) {
                if state.last_len == len && placement.index < state.last_index {
                    continue;
                }
                if bit_intersects(&state.bits, &placement.bits) {
                    continue;
                }
                let mut ok = true;
                let mut scan_counts = state.scan_counts.clone();
                for idx in 0..scans.len() {
                    scan_counts[idx] += placement.scan_counts[idx];
                    if dead_in_scan[idx] + scan_counts[idx] > scans[idx].count {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    continue;
                }
                let mut bits = state.bits.clone();
                bit_or_into(&mut bits, &placement.bits);
                next_states.push(BeamState {
                    bits,
                    scan_counts,
                    last_len: len,
                    last_index: placement.index,
                    score: state.score + placement.prior,
                });
            }
        }
        if next_states.is_empty() {
            return None;
        }
        next_states.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        if next_states.len() > beam_width {
            next_states.truncate(beam_width);
        }
        states = next_states;
    }

    let mut valid: Vec<BeamState> = Vec::new();
    'state_loop: for state in states {
        for &(r, c) in &known_hits {
            if !bit_has(&state.bits, r * n + c) {
                continue 'state_loop;
            }
        }
        for (idx, scan) in scans.iter().enumerate() {
            if dead_in_scan[idx] + state.scan_counts[idx] != scan.count {
                continue 'state_loop;
            }
        }
        valid.push(state);
    }

    if valid.len() < BEAM_MIN_VALID_STATES {
        return None;
    }

    let mut posterior = vec![vec![0.0; n]; n];
    for state in &valid {
        for r in 0..n {
            for c in 0..n {
                if grid[r][c] == Cell::Unknown && !shot[r][c] && bit_has(&state.bits, r * n + c) {
                    posterior[r][c] += 1.0;
                }
            }
        }
    }
    let scale = 1.0 / valid.len() as f64;
    for r in 0..n {
        for c in 0..n {
            posterior[r][c] *= scale;
        }
    }

    Some((posterior, valid.len()))
}


fn placement_scan_weight(
    r: usize,
    c: usize,
    dr: usize,
    dc: usize,
    len: usize,
    scan_constraints: &[(Scan, usize, f64)],
) -> f64 {
    let mut weight = 1.0f64;
    for &(scan, scan_remaining, ratio) in scan_constraints {
        let mut overlap = 0usize;
        for k in 0..len {
            if scan.contains(r + dr * k, c + dc * k) {
                overlap += 1;
            }
        }
        if overlap == 0 {
            continue;
        }
        if scan_remaining == 0 {
            return 0.0;
        }
        weight *= ratio.powf(overlap as f64 / len as f64);
    }
    weight.clamp(0.15, 4.00)
}

fn scan_adjusted_count(grid: &[Vec<Cell>], scan: Scan) -> (usize, usize) {
    let mut unknown = 0usize;
    let mut known_ship = 0usize;
    for r in scan.r1..=scan.r2 {
        for c in scan.c1..=scan.c2 {
            if grid[r][c] == Cell::Dead || grid[r][c] == Cell::Hit {
                known_ship += 1;
            } else if grid[r][c] == Cell::Unknown {
                unknown += 1;
            }
        }
    }
    (scan.count.saturating_sub(known_ship), unknown)
}

fn derived_quadrant_scans(n: usize, scans: &[Scan], remaining_cells: usize) -> Option<Vec<Scan>> {
    let top = scans.iter().find(|s| s.kind == ScanKind::TopHalf)?;
    let left = scans.iter().find(|s| s.kind == ScanKind::LeftHalf)?;
    let tl = scans.iter().find(|s| s.kind == ScanKind::TopLeft)?;
    let mid = n / 2;

    let tl_count = tl.count;
    let tr_count = top.count.saturating_sub(tl_count);
    let bl_count = left.count.saturating_sub(tl_count);
    let used = tl_count + tr_count + bl_count;
    let br_count = remaining_cells.saturating_sub(used);

    Some(vec![
        Scan { r1: 0, c1: 0, r2: mid - 1, c2: mid - 1, count: tl_count, kind: ScanKind::Other },
        Scan { r1: 0, c1: mid, r2: mid - 1, c2: n - 1, count: tr_count, kind: ScanKind::Other },
        Scan { r1: mid, c1: 0, r2: n - 1, c2: mid - 1, count: bl_count, kind: ScanKind::Other },
        Scan { r1: mid, c1: mid, r2: n - 1, c2: n - 1, count: br_count, kind: ScanKind::Other },
    ])
}

fn best_hunt_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
    scans: &[Scan],
    remaining_cells: usize,
    rng: &mut u64,
) -> Option<(usize, usize)> {
    let prob = build_probabilities(n, grid, shot, remaining, scans, remaining_cells);
    let beam_post = beam_fleet_posterior(n, grid, shot, remaining, scans, remaining_cells);
    let max_prob = prob
        .iter()
        .flat_map(|row| row.iter())
        .fold(0.0f64, |a, &b| a.max(b));
    let max_beam = beam_post
        .as_ref()
        .map(|(post, _)| post.iter().flat_map(|row| row.iter()).fold(0.0f64, |a, &b| a.max(b)))
        .unwrap_or(0.0);

    let mut candidates: Vec<(f64, f64, usize, usize)> = Vec::new();

    for r in 0..n {
        for c in 0..n {
            if shot[r][c] || grid[r][c] != Cell::Unknown {
                continue;
            }

            let center_bias = {
                let mid = (n - 1) as f64 / 2.0;
                let dr = (r as f64 - mid).abs();
                let dc = (c as f64 - mid).abs();
                0.001 * (n as f64 - dr - dc)
            };

            let mut density_scale = 1.0f64;
            if remaining_cells > 0 {
                let mut global_unknown = 0usize;
                for rr in 0..n {
                    for cc in 0..n {
                        if grid[rr][cc] == Cell::Unknown && !shot[rr][cc] {
                            global_unknown += 1;
                        }
                    }
                }
                let global_density = remaining_cells as f64 / global_unknown.max(1) as f64;
                let density_scans = derived_quadrant_scans(n, scans, remaining_cells);
                let scan_source: Vec<Scan> = density_scans.unwrap_or_else(|| scans.to_vec());
                for scan in &scan_source {
                    if !scan.contains(r, c) {
                        continue;
                    }
                    let (scan_remaining, scan_unknown) = scan_adjusted_count(grid, *scan);
                    if scan_unknown == 0 {
                        continue;
                    }
                    let scan_density = scan_remaining as f64 / scan_unknown as f64;
                    let ratio = if global_density > 0.0 { scan_density / global_density } else { 1.0 };
                    density_scale *= ratio.clamp(0.25, 2.50);
                }
            }
            density_scale = density_scale.clamp(0.20, 3.00);

            *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let jitter = ((*rng >> 32) as f64) * 1e-12;
            let posterior_prob = if let Some((post, states)) = beam_post.as_ref() {
                    if *states >= 8 && max_beam > 0.0 && max_prob > 0.0 {
                        Some(post[r][c] / max_beam * max_prob)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let base_prob = if let Some(post_score) = posterior_prob {
                    prob[r][c] * 0.70 + post_score * 0.30
                } else {
                    prob[r][c]
                };
                let score = base_prob * density_scale + center_bias + jitter;
            candidates.push((score, posterior_prob.unwrap_or(0.0), r, c));
        }
    }

    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let heat_best = candidates[0].0;
    let heat_second = if candidates.len() > 1 { candidates[1].0 } else { 0.0 };
    let heat_cell = (candidates[0].2, candidates[0].3);
    let mut by_beam = candidates.clone();
    by_beam.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let beam_best = by_beam[0].1;
    let beam_second = if by_beam.len() > 1 { by_beam[1].1 } else { 0.0 };
    let beam_cell = (by_beam[0].2, by_beam[0].3);
    let heat_rank_of_beam = candidates.iter().position(|x| x.2 == beam_cell.0 && x.3 == beam_cell.1).map(|x| x + 1).unwrap_or(9999);
    let beam_rank_of_heat = by_beam.iter().position(|x| x.2 == heat_cell.0 && x.3 == heat_cell.1).map(|x| x + 1).unwrap_or(9999);
    let beam_total: f64 = candidates.iter().map(|x| x.1.max(0.0)).sum();
    let mut beam_entropy = 0.0f64;
    if beam_total > 0.0 {
        for candidate in &candidates {
            let p = candidate.1.max(0.0) / beam_total;
            if p > 0.0 { beam_entropy -= p * p.ln(); }
        }
    }
    let max_beam_candidate = candidates.iter().take(BEAM_RERANK_TOP_K).map(|x| x.1).fold(0.0f64, |a, b| a.max(b));
    let mut best_choice = candidates[0];
    let mut best_choice_score = -1.0f64;
    for candidate in candidates.iter().take(BEAM_RERANK_TOP_K) {
        let beam_norm = if max_beam_candidate > 0.0 { candidate.1 / max_beam_candidate } else { 0.0 };
        let rerank_score = candidate.0 * (1.0 + BEAM_RERANK_FACTOR * beam_norm);
        if rerank_score > best_choice_score { best_choice_score = rerank_score; best_choice = *candidate; }
    }
    let chosen_rank = candidates.iter().position(|x| x.2 == best_choice.2 && x.3 == best_choice.3).map(|x| x + 1).unwrap_or(9999);
    let chosen_beam_rank = by_beam.iter().position(|x| x.2 == best_choice.2 && x.3 == best_choice.3).map(|x| x + 1).unwrap_or(9999);
    let beam_states = beam_post.as_ref().map(|(_, states)| *states).unwrap_or(0usize);
    diag_extra_line(&format!("BRANCH branch_turn=? n={} remaining_cells={} beam_states={} heat_cell={},{} beam_cell={},{} chosen_cell={},{} heat_rank_of_beam={} beam_rank_of_heat={} chosen_heat_rank={} chosen_beam_rank={} heat_best={:.9} heat_second={:.9} beam_best={:.9} beam_second={:.9} heat_cell_beam={:.9} beam_cell_heat={:.9} chosen_heat={:.9} chosen_beam={:.9} beam_entropy={:.9}", n, remaining_cells, beam_states, heat_cell.0, heat_cell.1, beam_cell.0, beam_cell.1, best_choice.2, best_choice.3, heat_rank_of_beam, beam_rank_of_heat, chosen_rank, chosen_beam_rank, heat_best, heat_second, beam_best, beam_second, candidates[0].1, by_beam[0].0, best_choice.0, best_choice.1, beam_entropy));
    Some((best_choice.2, best_choice.3))
}


fn placement_score_for_hits(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
    active_hits: &[(usize, usize)],
    candidate: (usize, usize),
) -> f64 {
    let mut required = vec![candidate];
    required.extend_from_slice(active_hits);
    let mut score = 0.0;

    for len in 1..remaining.len() {
        let ships = remaining[len];
        if ships == 0 {
            continue;
        }
        let weight = ships as f64 * len as f64;

        for r in 0..n {
            for c in 0..=n - len {
                let mut ok = true;
                for k in 0..len {
                    let cell = grid[r][c + k];
                    if cell == Cell::Miss || cell == Cell::Dead || (shot[r][c + k] && cell != Cell::Hit) {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    continue;
                }
                if required.iter().all(|&(rr, cc)| rr == r && c <= cc && cc < c + len) {
                    score += weight;
                }
            }
        }

        for r in 0..=n - len {
            for c in 0..n {
                let mut ok = true;
                for k in 0..len {
                    let cell = grid[r + k][c];
                    if cell == Cell::Miss || cell == Cell::Dead || (shot[r + k][c] && cell != Cell::Hit) {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    continue;
                }
                if required.iter().all(|&(rr, cc)| cc == c && r <= rr && rr < r + len) {
                    score += weight;
                }
            }
        }
    }

    score
}

fn chase_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    active_hits: &[(usize, usize)],
    remaining: &[usize],
    _rng: &mut u64,
) -> Option<(usize, usize)> {
    if active_hits.is_empty() {
        return None;
    }

    let prob = build_probabilities(n, grid, shot, remaining, &[], 0);
    let mut candidates: Vec<(usize, usize, f64)> = Vec::new();

    let same_row = active_hits.iter().all(|&(r, _)| r == active_hits[0].0);
    let same_col = active_hits.iter().all(|&(_, c)| c == active_hits[0].1);

    if active_hits.len() >= 2 && same_row {
        let r = active_hits[0].0;
        let min_c = active_hits.iter().map(|&(_, c)| c).min().unwrap();
        let max_c = active_hits.iter().map(|&(_, c)| c).max().unwrap();
        for nc in [min_c as isize - 1, max_c as isize + 1] {
            if inside(n, r as isize, nc) {
                let c = nc as usize;
                if !shot[r][c] && grid[r][c] == Cell::Unknown {
                    let placement_score = placement_score_for_hits(n, grid, shot, remaining, active_hits, (r, c));
                    let placement_bonus = if active_hits.len() == 1 { placement_score * 0.25 } else { placement_score };
                    candidates.push((r, c, 10000.0 + placement_bonus + prob[r][c]));
                }
            }
        }
    } else if active_hits.len() >= 2 && same_col {
        let c = active_hits[0].1;
        let min_r = active_hits.iter().map(|&(r, _)| r).min().unwrap();
        let max_r = active_hits.iter().map(|&(r, _)| r).max().unwrap();
        for nr in [min_r as isize - 1, max_r as isize + 1] {
            if inside(n, nr, c as isize) {
                let r = nr as usize;
                if !shot[r][c] && grid[r][c] == Cell::Unknown {
                    let placement_score = placement_score_for_hits(n, grid, shot, remaining, active_hits, (r, c));
                    let placement_bonus = if active_hits.len() == 1 { placement_score * 0.25 } else { placement_score };
                    candidates.push((r, c, 10000.0 + placement_bonus + prob[r][c]));
                }
            }
        }
    }

    if candidates.is_empty() {
        let dirs = [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)];
        for &(r, c) in active_hits {
            for &(dr, dc) in &dirs {
                let nr = r as isize + dr;
                let nc = c as isize + dc;
                if inside(n, nr, nc) {
                    let rr = nr as usize;
                    let cc = nc as usize;
                    if !shot[rr][cc] && grid[rr][cc] == Cell::Unknown {
                        let placement_score = placement_score_for_hits(n, grid, shot, remaining, active_hits, (rr, cc));
                    let placement_bonus = if active_hits.len() == 1 { placement_score * 0.25 } else { placement_score };
                    candidates.push((rr, cc, 5000.0 + placement_bonus + prob[rr][cc]));
                    }
                }
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    Some((candidates[0].0, candidates[0].1))
}

fn best_chase_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    active_clusters: &[Vec<(usize, usize)>],
    remaining: &[usize],
    rng: &mut u64,
) -> Option<((usize, usize), usize)> {
    let prob = build_probabilities(n, grid, shot, remaining, &[], 0);
    let mut best: Option<((usize, usize), usize, f64)> = None;

    for (idx, cluster) in active_clusters.iter().enumerate() {
        if let Some((r, c)) = chase_cell(n, grid, shot, cluster, remaining, rng) {
            let score = (cluster.len() as f64) * 100000.0 + prob[r][c];
            if best.map_or(true, |(_, _, best_score)| score > best_score) {
                best = Some(((r, c), idx, score));
            }
        }
    }

    best.map(|(cell, idx, _)| (cell, idx))
}

fn mark_active_dead(grid: &mut [Vec<Cell>], active_hits: &[(usize, usize)], last: (usize, usize)) {
    for &(r, c) in active_hits {
        grid[r][c] = Cell::Dead;
    }
    grid[last.0][last.1] = Cell::Dead;
}

fn longest_killed_segment(n: usize, active_hits: &[(usize, usize)], last: (usize, usize)) -> Vec<(usize, usize)> {
    let mut hit_set = vec![false; n * n];
    for &(r, c) in active_hits {
        hit_set[r * n + c] = true;
    }

    let mut best: Vec<(usize, usize)> = Vec::new();

    let mut row_segment = Vec::new();
    let mut c = last.1 as isize - 1;
    while c >= 0 && hit_set[last.0 * n + c as usize] {
        row_segment.push((last.0, c as usize));
        c -= 1;
    }
    c = last.1 as isize + 1;
    while c < n as isize && hit_set[last.0 * n + c as usize] {
        row_segment.push((last.0, c as usize));
        c += 1;
    }
    if row_segment.len() > best.len() {
        best = row_segment;
    }

    let mut col_segment = Vec::new();
    let mut r = last.0 as isize - 1;
    while r >= 0 && hit_set[r as usize * n + last.1] {
        col_segment.push((r as usize, last.1));
        r -= 1;
    }
    r = last.0 as isize + 1;
    while r < n as isize && hit_set[r as usize * n + last.1] {
        col_segment.push((r as usize, last.1));
        r += 1;
    }
    if col_segment.len() > best.len() {
        best = col_segment;
    }

    best
}

fn infer_killed_hits(
    n: usize,
    remaining: &[usize],
    active_hits: &[(usize, usize)],
    last: (usize, usize),
) -> (Vec<(usize, usize)>, usize) {
    let mut hit_set = vec![false; n * n];
    for &(r, c) in active_hits {
        hit_set[r * n + c] = true;
    }

    let mut candidates: Vec<Vec<(usize, usize)>> = Vec::new();
    for len in 2..remaining.len() {
        if remaining[len] == 0 || len > n {
            continue;
        }

        let min_c = last.1.saturating_sub(len - 1);
        let max_c = last.1.min(n - len);
        for start_c in min_c..=max_c {
            let mut candidate = Vec::with_capacity(len);
            let mut ok = true;
            for k in 0..len {
                let cell = (last.0, start_c + k);
                candidate.push(cell);
                if cell != last && !hit_set[cell.0 * n + cell.1] {
                    ok = false;
                    break;
                }
            }
            if ok {
                candidates.push(candidate);
            }
        }

        let min_r = last.0.saturating_sub(len - 1);
        let max_r = last.0.min(n - len);
        for start_r in min_r..=max_r {
            let mut candidate = Vec::with_capacity(len);
            let mut ok = true;
            for k in 0..len {
                let cell = (start_r + k, last.1);
                candidate.push(cell);
                if cell != last && !hit_set[cell.0 * n + cell.1] {
                    ok = false;
                    break;
                }
            }
            if ok {
                candidates.push(candidate);
            }
        }
    }

    if candidates.is_empty() {
        let killed_hits = longest_killed_segment(n, active_hits, last);
        return (killed_hits.clone(), killed_hits.len() + 1);
    }

    candidates.sort();
    candidates.dedup();
    candidates.sort_by(|a, b| b.len().cmp(&a.len()));
    let inferred_len = candidates[0].len();
    let same_len_count = candidates
        .iter()
        .take_while(|candidate| candidate.len() == inferred_len)
        .count();

    if same_len_count == 1 {
        let killed_hits: Vec<(usize, usize)> = candidates[0]
            .iter()
            .copied()
            .filter(|&cell| cell != last)
            .collect();
        return (killed_hits, inferred_len);
    }

    let plausible_count = same_len_count.min(4);
    let plausible = &candidates[..plausible_count];

    let mut common = Vec::new();
    for &cell in active_hits {
        if plausible.iter().all(|candidate| candidate.contains(&cell)) {
            common.push(cell);
        }
    }

    (common, inferred_len)
}

fn split_hit_clusters(n: usize, hits: Vec<(usize, usize)>) -> Vec<Vec<(usize, usize)>> {
    let mut present = vec![vec![false; n]; n];
    for &(r, c) in &hits {
        present[r][c] = true;
    }

    let mut seen = vec![vec![false; n]; n];
    let mut clusters = Vec::new();
    let dirs = [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)];

    for &(sr, sc) in &hits {
        if seen[sr][sc] {
            continue;
        }

        let mut stack = vec![(sr, sc)];
        let mut cluster = Vec::new();
        seen[sr][sc] = true;

        while let Some((r, c)) = stack.pop() {
            cluster.push((r, c));
            for &(dr, dc) in &dirs {
                let nr = r as isize + dr;
                let nc = c as isize + dc;
                if inside(n, nr, nc) {
                    let rr = nr as usize;
                    let cc = nc as usize;
                    if present[rr][cc] && !seen[rr][cc] {
                        seen[rr][cc] = true;
                        stack.push((rr, cc));
                    }
                }
            }
        }

        clusters.push(cluster);
    }

    clusters
}

fn opening_scan_schedule(n: usize, p: f64) -> Vec<Scan> {
    let mut scans = Vec::new();
    let mid = n / 2;
    if p <= 0.25 {
        scans.push(Scan { r1: 0, c1: 0, r2: mid - 1, c2: mid - 1, count: 0, kind: ScanKind::TopLeft });
        scans.push(Scan { r1: 0, c1: mid, r2: mid - 1, c2: n - 1, count: 0, kind: ScanKind::Other });
        scans.push(Scan { r1: mid, c1: 0, r2: n - 1, c2: mid - 1, count: 0, kind: ScanKind::Other });
        scans.push(Scan { r1: mid, c1: mid, r2: n - 1, c2: n - 1, count: 0, kind: ScanKind::Other });
    } else {
        scans.push(Scan { r1: 0, c1: 0, r2: mid - 1, c2: n - 1, count: 0, kind: ScanKind::TopHalf });
        scans.push(Scan { r1: 0, c1: 0, r2: n - 1, c2: mid - 1, count: 0, kind: ScanKind::LeftHalf });
    }
    scans
}

fn apply_zero_scan(grid: &mut [Vec<Cell>], scan: Scan) {
    if scan.count != 0 {
        return;
    }
    for r in scan.r1..=scan.r2 {
        for c in scan.c1..=scan.c2 {
            if grid[r][c] == Cell::Unknown {
                grid[r][c] = Cell::Miss;
            }
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin).lines();
    let mut stdout = io::stdout();

    let n: usize = read_line(&mut reader);
    let s: usize = read_line(&mut reader);
    let l: usize = read_line(&mut reader);
    let p: f64 = read_line(&mut reader);

    let mut remaining = vec![0usize; l + 1];
    for len in 1..=l {
        remaining[len] = read_line(&mut reader);
    }

    let mut grid = vec![vec![Cell::Unknown; n]; n];
    let mut shot = vec![vec![false; n]; n];
    let mut active_clusters: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut branch_turn: usize = 0;
    diag_extra_line(&format!("GAME n={} p={:.9} remaining={:?}", n, p, remaining));
    let opening_scans = opening_scan_schedule(n, p);
    let mut next_opening_scan = 0usize;
    let mut scans: Vec<Scan> = Vec::new();
    let mut remaining_cells: usize = (1..remaining.len()).map(|len| len * remaining[len]).sum();
    let mut alive = s;
    let mut rng = 0x9E37_79B9_7F4A_7C15u64 ^ ((n as u64) << 32) ^ s as u64;

    while alive > 0 {
        if active_clusters.is_empty() && next_opening_scan < opening_scans.len() {
            let mut scan = opening_scans[next_opening_scan];
            next_opening_scan += 1;
            println!("SCAN {} {} {} {}", scan.r1, scan.c1, scan.r2, scan.c2);
            stdout.flush().unwrap();
            scan.count = read_line(&mut reader);
            apply_zero_scan(&mut grid, scan);
            scans.push(scan);
            let _elapsed_time: i32 = read_line(&mut reader);
            continue;
        }

        let target = best_chase_cell(n, &grid, &shot, &active_clusters, &remaining, &mut rng)
            .map(|(cell, cluster_idx)| (cell, Some(cluster_idx)))
            .or_else(|| best_hunt_cell(n, &grid, &shot, &remaining, &scans, remaining_cells, &mut rng).map(|cell| (cell, None)))
            .expect("No legal shot left");

        let ((mut r, mut c), target_cluster) = target;
        shot[r][c] = true;

        branch_turn += 1;
        if target_cluster.is_none() {
            if let (Ok(force_turn), Ok(force_r), Ok(force_c)) = (std::env::var("BS_FORCE_TURN"), std::env::var("BS_FORCE_R"), std::env::var("BS_FORCE_C")) {
                if force_turn.parse::<usize>().ok() == Some(branch_turn) {
                    if let (Ok(fr), Ok(fc)) = (force_r.parse::<usize>(), force_c.parse::<usize>()) {
                        if fr < n && fc < n && !shot[fr][fc] && grid[fr][fc] == Cell::Unknown {
                            diag_extra_line(&format!("FORCE branch_turn={} from={},{} to={},{}", branch_turn, r, c, fr, fc));
                            r = fr;
                            c = fc;
                        }
                    }
                }
            }
        }
        diag_extra_line(&format!("DECISION branch_turn={} cell={},{} target_cluster={:?}", branch_turn, r, c, target_cluster));
        println!("SHOOT {} {}", r, c);
        stdout.flush().unwrap();

        let result: String = read_line(&mut reader);
        diag_extra_line(&format!("RESULT branch_turn={} cell={},{} result={} remaining_cells={} target_cluster={:?}", branch_turn, r, c, result, remaining_cells, target_cluster));
        match result.as_str() {
            "MISS" => {
                grid[r][c] = Cell::Miss;
            }
            "HIT" => {
                remaining_cells = remaining_cells.saturating_sub(1);
                grid[r][c] = Cell::Hit;
                if let Some(cluster_idx) = target_cluster {
                    if cluster_idx < active_clusters.len() {
                        active_clusters[cluster_idx].push((r, c));
                    } else {
                        active_clusters.push(vec![(r, c)]);
                    }
                } else {
                    active_clusters.push(vec![(r, c)]);
                }
            }
            "KILL" => {
                remaining_cells = remaining_cells.saturating_sub(1);
                let inferred_len = if let Some(cluster_idx) = target_cluster {
                    if cluster_idx < active_clusters.len() {
                        let killed_cluster = active_clusters.swap_remove(cluster_idx);
                        let (killed_hits, inferred_len) = infer_killed_hits(n, &remaining, &killed_cluster, (r, c));
                        mark_active_dead(&mut grid, &killed_hits, (r, c));

                        let mut killed = vec![vec![false; n]; n];
                        for &(kr, kc) in &killed_hits {
                            killed[kr][kc] = true;
                        }

                        let leftovers: Vec<(usize, usize)> = killed_cluster
                            .into_iter()
                            .filter(|&(hr, hc)| !killed[hr][hc])
                            .collect();
                        active_clusters.extend(split_hit_clusters(n, leftovers));

                        inferred_len
                    } else {
                        grid[r][c] = Cell::Dead;
                        1
                    }
                } else {
                    grid[r][c] = Cell::Dead;
                    1
                };
                decrement_ship_count(&mut remaining, inferred_len);
                alive -= 1;
            }
            _ => panic!("Unexpected result: {}", result),
        }

        let _elapsed_time: i32 = read_line(&mut reader);
    }
}
