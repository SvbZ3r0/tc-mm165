use std::io::{self, BufRead, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Unknown,
    Miss,
    Hit,
    Dead,
}

#[derive(Clone, Copy)]
struct Scan {
    r1: usize,
    c1: usize,
    r2: usize,
    c2: usize,
    count: usize,
}

impl Scan {
    fn contains(&self, r: usize, c: usize) -> bool {
        self.r1 <= r && r <= self.r2 && self.c1 <= c && c <= self.c2
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

fn build_probabilities(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
) -> Vec<Vec<f64>> {
    let mut raw = vec![vec![0.0f64; n]; n];
    let mut norm = vec![vec![0.0f64; n]; n];
    let mut legal_counts = vec![0usize; remaining.len()];

    for len in 1..remaining.len() {
        if remaining[len] == 0 {
            continue;
        }

        for r in 0..n {
            for c in 0..=n - len {
                if placement_is_legal(grid, shot, r, c, 0, 1, len) {
                    legal_counts[len] += 1;
                }
            }
        }

        for r in 0..=n - len {
            for c in 0..n {
                if placement_is_legal(grid, shot, r, c, 1, 0, len) {
                    legal_counts[len] += 1;
                }
            }
        }
    }

    for len in 1..remaining.len() {
        let ships = remaining[len];
        if ships == 0 || legal_counts[len] == 0 {
            continue;
        }
        let raw_weight = ships as f64 * len as f64;
        let norm_weight = raw_weight / legal_counts[len] as f64;

        for r in 0..n {
            for c in 0..=n - len {
                if placement_is_legal(grid, shot, r, c, 0, 1, len) {
                    for k in 0..len {
                        raw[r][c + k] += raw_weight;
                        norm[r][c + k] += norm_weight;
                    }
                }
            }
        }

        for r in 0..=n - len {
            for c in 0..n {
                if placement_is_legal(grid, shot, r, c, 1, 0, len) {
                    for k in 0..len {
                        raw[r + k][c] += raw_weight;
                        norm[r + k][c] += norm_weight;
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

fn best_hunt_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
    scans: &[Scan],
    remaining_cells: usize,
    rng: &mut u64,
) -> Option<(usize, usize)> {
    let prob = build_probabilities(n, grid, shot, remaining);
    let mut best = None;
    let mut best_score = -1.0f64;

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
                for scan in scans {
                    if !scan.contains(r, c) {
                        continue;
                    }
                    let mut scan_unknown = 0usize;
                    let mut scan_known_ship = 0usize;
                    for rr in scan.r1..=scan.r2 {
                        for cc in scan.c1..=scan.c2 {
                            if grid[rr][cc] == Cell::Dead || grid[rr][cc] == Cell::Hit {
                                scan_known_ship += 1;
                            } else if grid[rr][cc] == Cell::Unknown && !shot[rr][cc] {
                                scan_unknown += 1;
                            }
                        }
                    }
                    if scan_unknown == 0 {
                        continue;
                    }
                    let scan_remaining = scan.count.saturating_sub(scan_known_ship);
                    let scan_density = scan_remaining as f64 / scan_unknown as f64;
                    let expected = global_density * scan_unknown as f64;
                    let ratio = if global_density > 0.0 { scan_density / global_density } else { 1.0 };
                    let surprise = (scan_remaining as f64 - expected).abs();
                    let strength = 0.5 + 0.5 * (surprise / (expected + 1.0).sqrt()).min(1.0);
                    density_scale *= ratio.clamp(0.25, 2.50).powf(strength);
                }
            }
            density_scale = density_scale.clamp(0.20, 3.00);

            *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let jitter = ((*rng >> 32) as f64) * 1e-12;
            let score = prob[r][c] * density_scale + center_bias + jitter;
            if score > best_score {
                best_score = score;
                best = Some((r, c));
            }
        }
    }

    best
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

    let prob = build_probabilities(n, grid, shot, remaining);
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
    let prob = build_probabilities(n, grid, shot, remaining);
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

fn infer_killed_hits(active_hits: &[(usize, usize)], last: (usize, usize)) -> Vec<(usize, usize)> {
    let mut hit_set = vec![false; 400];
    for &(r, c) in active_hits {
        hit_set[r * 20 + c] = true;
    }

    let mut best: Vec<(usize, usize)> = Vec::new();

    let mut row_segment = Vec::new();
    let mut c = last.1 as isize - 1;
    while c >= 0 && hit_set[last.0 * 20 + c as usize] {
        row_segment.push((last.0, c as usize));
        c -= 1;
    }
    c = last.1 as isize + 1;
    while c < 20 && hit_set[last.0 * 20 + c as usize] {
        row_segment.push((last.0, c as usize));
        c += 1;
    }
    if row_segment.len() > best.len() {
        best = row_segment;
    }

    let mut col_segment = Vec::new();
    let mut r = last.0 as isize - 1;
    while r >= 0 && hit_set[r as usize * 20 + last.1] {
        col_segment.push((r as usize, last.1));
        r -= 1;
    }
    r = last.0 as isize + 1;
    while r < 20 && hit_set[r as usize * 20 + last.1] {
        col_segment.push((r as usize, last.1));
        r += 1;
    }
    if col_segment.len() > best.len() {
        best = col_segment;
    }

    best
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
    if p <= 0.30 {
        scans.push(Scan { r1: 0, c1: 0, r2: mid - 1, c2: n - 1, count: 0 });
        scans.push(Scan { r1: 0, c1: 0, r2: n - 1, c2: mid - 1, count: 0 });
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

        let ((r, c), target_cluster) = target;
        shot[r][c] = true;
        println!("SHOOT {} {}", r, c);
        stdout.flush().unwrap();

        let result: String = read_line(&mut reader);
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
                        let killed_hits = infer_killed_hits(&killed_cluster, (r, c));
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

                        killed_hits.len() + 1
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
