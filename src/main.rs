use std::env;
use std::process;
use std::io::Read;
use std::ffi::OsString;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn cksum(slice: &[u8]) -> u64 {
	let mut state = DefaultHasher::new();
	slice.hash(&mut state);
	state.finish()
}

fn parse_args() -> Result<(usize, std::fs::File, std::fs::File), i32> {
	let args: Vec<OsString> = env::args_os().collect();
	if args.len() != 4 {
		print!("\
			blockcmp - blockwise file comparison\n\
			\n\
			Usage: blockcmp BLOCKSIZE FILE_A FILE_B\n\
			\n\
			Output: A list of differing blocks, consisting of\n\
			offsets (in blocks and bytes) and block checksums.\n\
		");
		return Err(1);
	}

	let lossy = args[1].to_string_lossy();
	let bs = lossy.parse::<usize>().map_err(|e| {
		eprintln!("{}: {}", lossy, e);
		1
	})?;

	let open_or_complain = |path: &OsString| {
		std::fs::File::open(&path).map_err(|e| {
			let lossy = path.to_string_lossy();
			eprintln!("{}: {}", lossy, e);
			2
		})
	};

	let file_a = open_or_complain(&args[2])?;
	let file_b = open_or_complain(&args[3])?;

	Ok((bs, file_a, file_b))
}

fn realmain() -> Result<(), i32> {
	let (bs, mut file_a, mut file_b) = parse_args()?;

	let mut buf_a = Vec::<u8>::with_capacity(bs);
	let mut buf_b = Vec::<u8>::with_capacity(bs);
	unsafe {
		buf_a.set_len(bs);
		buf_b.set_len(bs);
	}
	let mut off: usize = 0;
	let mut count: usize = 0;

	loop {
		let res_a = file_a.read_exact(&mut buf_a);
		let res_b = file_b.read_exact(&mut buf_b);

		match (res_a, res_b) {
			(Ok(_), Ok(_)) => {
				if buf_a != buf_b {
					println!(
						"block {} (offset {}): {:016x} <> {:016x}",
						count, off, cksum(&buf_a), cksum(&buf_b)
					);
				}
			}
			_ => break
		}
		off += bs;
		count += 1;
	}
	Ok(())
}

fn main() {
	if let Err(code) = realmain() {
		process::exit(code);
	}
}
