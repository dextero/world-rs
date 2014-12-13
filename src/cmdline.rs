extern crate getopts;

use getopts::{optopt,optflag,getopts,OptGroup};
use std::os;
use std::cmp::min;
use std::str::FromStr;

include!("macros.rs")

pub struct Args {
    pub rng_seed: [u32, ..4],
    pub resolution: [u32, ..2],
    pub world_detail_level: uint,
    pub plate_sim_detail_level: uint,
    pub plate_sim_steps: uint,
    pub plate_sim_plates: uint,
}

fn from_str_or_panic<T: FromStr>(text: &str) -> T {
    match from_str::<T>(text) {
        Some(val) => val,
        None => {
            panic_bt!("invalid value: {}, use -h for help", text);
        }
    }
}

fn parse_rng_seed(arg: &str,
                  seed: &mut [u32, ..4]) {
    let split: Vec<&str> = arg.split_str(",").collect();

    if split.len() > seed.len() {
        panic_bt!("excess RNG seed initializer elements: got {}, expected no more than {}",
                  split.len(), seed.len());
    }

    for i in range(0u, min(seed.len(), split.len())) {
        seed[i] = from_str_or_panic(split[i]);
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
            rng_seed: [1, 2, 3, 4],
            resolution: [1000, 1000],
            world_detail_level: 4,
            plate_sim_detail_level: 2,
            plate_sim_steps: 10,
            plate_sim_plates: 25,
        };

        match matches.opt_str("s") {
            Some(arg) => parse_rng_seed(arg.as_slice(), &mut ret.rng_seed),
            None => {}
        }

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


