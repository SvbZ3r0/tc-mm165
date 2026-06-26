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

    let mut shot = vec![false; n * n];
    let mut alive = s;
    let mut seed = 0xD1B5_4A32_D192_ED03u64 ^ ((n as u64) << 32) ^ s as u64 ^ p.to_bits();

    while alive > 0 {
        let mut idx = (next_rand(&mut seed) as usize) % (n * n);
        while shot[idx] {
            idx += 1;
            if idx == n * n {
                idx = 0;
            }
        }
        shot[idx] = true;

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
