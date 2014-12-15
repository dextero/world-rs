extern crate getopts;

use getopts::{optopt,optflag,getopts,OptGroup};
use std::os;
use std::fmt;
use std::str::FromStr;

include!("macros.rs")

fn get_block(data: &[u8],
             block_index: uint) -> u32 {
      data[block_index * 4 + 0] as u32 << 24
    | data[block_index * 4 + 1] as u32 << 16
    | data[block_index * 4 + 2] as u32 << 8
    | data[block_index * 4 + 3] as u32
}

fn rotl(num: u32,
        by: uint) -> u32 {
    num << by
    | num >> (32 - by)
}

fn murmur_step(key1: &mut u32,
               c1: u32,
               c2: u32,
               hash1: &mut u32,
               hash2: u32,
               rot1: uint,
               rot2: uint,
               magic: u32) {
    *key1 *= c1;
    *key1 = rotl(*key1, rot1);
    *key1 *= c2;

    *hash1 ^= *key1;
    *hash1 = rotl(*hash1, rot2);
    *hash1 += hash2;
    *hash1 = *hash1 * 5 + magic;
}

fn murmur_tail(tail: &[u8],
               hash: &mut [u32, ..4],
               c: &[u32, ..4]) {
    let mut k: [u32, ..4] = [0, 0, 0, 0];

    k[0] = if tail.len() >= 15 { tail[14] as u32 << 16 } else { 0 }
         | if tail.len() >= 14 { tail[13] as u32 << 8  } else { 0 }
         | if tail.len() >= 13 { tail[12] as u32 << 0  } else { 0 };
    k[1] = if tail.len() >= 12 { tail[11] as u32 << 24 } else { 0 }
         | if tail.len() >= 11 { tail[10] as u32 << 16 } else { 0 }
         | if tail.len() >= 10 { tail[ 9] as u32 << 8  } else { 0 }
         | if tail.len() >=  9 { tail[ 8] as u32 << 0  } else { 0 };
    k[2] = if tail.len() >=  8 { tail[ 7] as u32 << 24 } else { 0 }
         | if tail.len() >=  7 { tail[ 6] as u32 << 16 } else { 0 }
         | if tail.len() >=  6 { tail[ 5] as u32 << 8  } else { 0 }
         | if tail.len() >=  5 { tail[ 4] as u32 << 0  } else { 0 };
    k[3] = if tail.len() >=  4 { tail[ 3] as u32 << 24 } else { 0 }
         | if tail.len() >=  3 { tail[ 2] as u32 << 16 } else { 0 }
         | if tail.len() >=  2 { tail[ 1] as u32 << 8  } else { 0 }
         | if tail.len() >=  1 { tail[ 0] as u32 << 0  } else { 0 };

    for i in range(0u, 4u) {
        if k[i] != 0 {
            k[i] *= c[i];
            k[i] = rotl(k[i], 15 + i);
            k[i] *= c[(i + 1) % 4];
            hash[i] ^= k[i];
        }
    }
}

fn fmix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x *= 0x85ebca6b;
    x ^= x >> 13;
    x *= 0xc2b2ae35;
    x ^= x >> 16;

    x
}

fn murmur_hash3(text: &[u8],
                hash: &mut [u32, ..4]) {
    let num_blocks = text.len() / 16u;

    for i in range(0u, 4u) {
        hash[i] = 0;
    }

    let c = [0x239b961bu32, 0xab0e9789u32, 0x38b34ae5u32, 0xa1e38b93u32];

    for i in range(0u, num_blocks) {
        let mut k = [get_block(text, i * 4 + 0),
                     get_block(text, i * 4 + 1),
                     get_block(text, i * 4 + 2),
                     get_block(text, i * 4 + 3)];

        let mut h_tmp: u32;

        h_tmp = hash[1];
        murmur_step(&mut k[0], c[0], c[1], &mut hash[0], h_tmp, 15, 19, 0x561ccd1b);
        h_tmp = hash[2];
        murmur_step(&mut k[1], c[1], c[2], &mut hash[1], h_tmp, 16, 17, 0x0bcaa747);
        h_tmp = hash[3];
        murmur_step(&mut k[2], c[2], c[3], &mut hash[2], h_tmp, 17, 15, 0x96cd1c35);
        h_tmp = hash[0];
        murmur_step(&mut k[3], c[3], c[0], &mut hash[3], h_tmp, 18, 13, 0x32ac3b17);
    }

    let tail = text[num_blocks..];
    murmur_tail(tail, hash, &c);

    for i in range(0u, 4u) {
        hash[i] ^= text.len() as u32;
    }

    hash[0] += hash[1] + hash[2] + hash[3];
    hash[1] += hash[0];
    hash[2] += hash[0];
    hash[3] += hash[0];

    for i in range(0u, 4u) {
        hash[i] = fmix(hash[i]);
    }

    hash[0] += hash[1] + hash[2] + hash[3];
    hash[1] += hash[0];
    hash[2] += hash[0];
    hash[3] += hash[0];
}

pub struct Args {
    pub rng_seed: String,
    pub rng_seed_hash: [u32, ..4],
    pub resolution: [u32, ..2],
    pub world_detail_level: uint,
    pub plate_sim_detail_level: uint,
    pub plate_sim_steps: uint,
    pub plate_sim_plates: uint,
}

impl fmt::Show for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "Configuration:"));
        try!(writeln!(f, "- rng_seed = {}", self.rng_seed));
        try!(writeln!(f, "- rng_seed_hash = {}", self.rng_seed_hash));
        try!(writeln!(f, "- resolution = {} x {}", self.resolution[0], self.resolution[1]));
        try!(writeln!(f, "- world_detail_level = {}", self.world_detail_level));
        try!(writeln!(f, "- plate_sim_detail_level = {}", self.plate_sim_detail_level));
        try!(writeln!(f, "- plate_sim_steps = {}", self.plate_sim_steps));
        writeln!(f, "- plate_sim_plates = {}", self.plate_sim_plates)
    }
}

fn from_str_or_panic<T: FromStr>(text: &str) -> T {
    match from_str::<T>(text) {
        Some(val) => val,
        None => {
            panic_bt!("invalid value: {}, use -h for help", text);
        }
    }
}

fn parse_resolution(arg: &str,
                    resolution: &mut [u32, ..2]) {
    let split: Vec<&str> = arg.split_str(",").collect();

    if split.len() != 2 {
        panic_bt!("invalid resolution format, got: {}, expected: WIDTH,HEIGHT", arg);
    }

    for i in range(0u, 2u) {
        resolution[i] = from_str_or_panic(split[i]);
    }
}

fn print_help(program_name: &str,
              opts: &[OptGroup]) {
    println!("usage: {} [ options... ]", program_name);

    for opt in opts.iter() {
        println!("    -{}, --{:<12} {:<20} - {}",
                 opt.short_name, opt.long_name, opt.hint, opt.desc);
    }
}

impl Args {
    pub fn parse() -> Result<Args, int> {
        let args = os::args();
        let opts = &[
             optopt("s", "rng-seed",     "random number generator seed",     "NUM,NUM,NUM,NUM"),
             optopt("r", "resolution",   "window size/resolution",           "NUM,NUM"),
             optopt("w", "world-detail", "world model detail level",         "NUM"),
             optopt("p", "plate-detail", "plate simulation detail level",    "NUM"),
             optopt("P", "plate-steps",  "number of plate simulation steps", "NUM"),
             optopt("n", "plate-count",  "number of plates to generate",     "NUM"),
            optflag("h", "help",         "print this message and exit"),
        ];

        let matches = match getopts(args.tail(), opts) {
            Ok(m) => m,
            Err(reason) => {
                println_err!("{}", reason.to_string());
                return Err(-1);
            }
        };

        if matches.opt_present("h") {
            print_help(args[0].as_slice(), opts);
            return Err(0);
        }

        let mut ret: Args = Args {
            rng_seed: String::from_str("asd"),
            rng_seed_hash: [1, 2, 3, 4],
            resolution: [1000, 1000],
            world_detail_level: 4,
            plate_sim_detail_level: 2,
            plate_sim_steps: 10,
            plate_sim_plates: 25,
        };

        match matches.opt_str("s") {
            Some(arg) => ret.rng_seed = arg,
            None => {}
        }
        murmur_hash3(ret.rng_seed.as_slice().as_bytes(), &mut ret.rng_seed_hash);

        match matches.opt_str("r") {
            Some(arg) => parse_resolution(arg.as_slice(), &mut ret.resolution),
            None => {}
        }

        match matches.opt_str("w") {
            Some(arg) => ret.world_detail_level = from_str_or_panic(arg.as_slice()),
            None => {}
        }

        match matches.opt_str("p") {
            Some(arg) => ret.plate_sim_detail_level = from_str_or_panic(arg.as_slice()),
            None => {}
        }
        match matches.opt_str("P") {
            Some(arg) => ret.plate_sim_steps = from_str_or_panic(arg.as_slice()),
            None => {}
        }
        match matches.opt_str("n") {
            Some(arg) => ret.plate_sim_plates = from_str_or_panic(arg.as_slice()),
            None => {}
        }

        Ok(ret)
    }
}


