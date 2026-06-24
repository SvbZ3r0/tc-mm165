use std::io::{self, BufRead, Write};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Unknown,
    Miss,
    Hit,
    Dead,
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

fn build_probabilities(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
) -> Vec<Vec<f64>> {
    let mut prob = vec![vec![0.0f64; n]; n];

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
                    if grid[r][c + k] != Cell::Unknown || shot[r][c + k] {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    for k in 0..len {
                        prob[r][c + k] += weight;
                    }
                }
            }
        }

        for r in 0..=n - len {
            for c in 0..n {
                let mut ok = true;
                for k in 0..len {
                    if grid[r + k][c] != Cell::Unknown || shot[r + k][c] {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    for k in 0..len {
                        prob[r + k][c] += weight;
                    }
                }
            }
        }
    }

    prob
}

fn best_hunt_cell(
    n: usize,
    grid: &[Vec<Cell>],
    shot: &[Vec<bool>],
    remaining: &[usize],
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

            *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let jitter = ((*rng >> 32) as f64) * 1e-12;
            let score = prob[r][c] + center_bias + jitter;
            if score > best_score {
                best_score = score;
                best = Some((r, c));
            }
        }
    }

    best
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
                    candidates.push((r, c, 10000.0 + prob[r][c]));
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
                    candidates.push((r, c, 10000.0 + prob[r][c]));
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
                        candidates.push((rr, cc, 5000.0 + prob[rr][cc]));
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

fn main() {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin).lines();
    let mut stdout = io::stdout();

    let n: usize = read_line(&mut reader);
    let s: usize = read_line(&mut reader);
    let l: usize = read_line(&mut reader);
    let _p: f64 = read_line(&mut reader);

    let mut remaining = vec![0usize; l + 1];
    for len in 1..=l {
        remaining[len] = read_line(&mut reader);
    }

    let mut grid = vec![vec![Cell::Unknown; n]; n];
    let mut shot = vec![vec![false; n]; n];
    let mut active_clusters: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut alive = s;
    let mut rng = 0x9E37_79B9_7F4A_7C15u64 ^ ((n as u64) << 32) ^ s as u64;

    while alive > 0 {
        let target = best_chase_cell(n, &grid, &shot, &active_clusters, &remaining, &mut rng)
            .map(|(cell, cluster_idx)| (cell, Some(cluster_idx)))
            .or_else(|| best_hunt_cell(n, &grid, &shot, &remaining, &mut rng).map(|cell| (cell, None)))
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
                let inferred_len = if let Some(cluster_idx) = target_cluster {
                    if cluster_idx < active_clusters.len() {
                        let killed_cluster = active_clusters.swap_remove(cluster_idx);
                        let inferred_len = killed_cluster.len() + 1;
                        mark_active_dead(&mut grid, &killed_cluster, (r, c));
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
