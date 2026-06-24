use std::io::{self, BufRead, Write};

// Helper function to read the next line and parse it into the requested type
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

#[allow(non_snake_case)]		//so it doesn't complaing about variable names
fn main() {
	let stdin = io::stdin();
	let mut reader = io::BufReader::new(stdin).lines();
	let mut stdout = io::stdout();

	// Read initial variables
	let N: usize = read_line(&mut reader);
	let S: usize = read_line(&mut reader);
	let L: usize = read_line(&mut reader);
	let _P: f64 = read_line(&mut reader);

	// Initialize and populate ship lengths
	let mut shipLengths = vec![0; L + 1];
	for i in 1..=L {
		shipLengths[i] = read_line(&mut reader);
	}

	// 2D grid for tracking visited coordinates
	let mut seen = vec![vec![false; N]; N];

	let mut seed: u32 = 42;
	let mut alive = S;

	while alive > 0 {
		seed = (seed * 8009 + 104729) % (1 << 16);
		let num = (seed as usize) % (N * N);
		let r = num / N;
		let c = num % N;

		if seen[r][c] {
				continue;
		}

		seen[r][c] = true;

		// Print the action and flush stdout immediately
		println!("SHOOT {} {}", r, c);
		stdout.flush().unwrap();

		// Read the game results for this turn
		let result: String = read_line(&mut reader);
		if result == "KILL"{
			alive -= 1;
		}

		let _elapsed_time: i32 = read_line(&mut reader);
	}
}