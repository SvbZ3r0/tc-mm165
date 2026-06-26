use std::io::{self, BufRead, Write};

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

fn next_rand(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed
}

fn shuffled_indices(indices: &mut [usize], seed: &mut u64) {
    for i in (1..indices.len()).rev() {
        let j = (next_rand(seed) as usize) % (i + 1);
        indices.swap(i, j);
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

    for _ in 1..=l {
        let _: usize = read_line(&mut reader);
    }

    let mut seed = 0xA24B_6C8D_1357_9BDFu64 ^ ((n as u64) << 32) ^ s as u64 ^ p.to_bits();
    let parity = (next_rand(&mut seed) as usize) & 1;
    let mut first = Vec::new();
    let mut second = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let idx = r * n + c;
            if (r + c) & 1 == parity {
                first.push(idx);
            } else {
                second.push(idx);
            }
        }
    }
    shuffled_indices(&mut first, &mut seed);
    shuffled_indices(&mut second, &mut seed);
    first.extend(second);

    let mut alive = s;
    for idx in first {
        if alive == 0 {
            break;
        }

        let r = idx / n;
        let c = idx % n;
        println!("SHOOT {} {}", r, c);
        stdout.flush().unwrap();

        let result: String = read_line(&mut reader);
        if result == "KILL" {
            alive -= 1;
        }

        let _: i32 = read_line(&mut reader);
    }
}
