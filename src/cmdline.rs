extern crate getopts;

use getopts::{optopt,optflag,getopts,OptGroup};
use std::os;
use std::cmp::min;

include!("macros.rs")

pub struct Args {
    pub rng_seed: [u32, ..4]
}

fn parse_rng_seed(arg: &str,
                  seed: &mut [u32, ..4]) {
    let split: Vec<&str> = arg.split_str(",").collect();

    if split.len() > seed.len() {
        println_err!("warning: excess RNG seed initializer elements: got {}, expected no more than {}",
                     split.len(), seed.len());
    }

    for i in range(0u, min(seed.len(), split.len())) {
        match from_str::<u32>(split[i].as_slice()) {
            Some(val) => seed[i] = val,
            None => {
                println_err!("warning: {} is not a valid 32-bit unsigned integer in: {}",
                             split[i], arg);
            }
        }
    }
}

fn print_help(program_name: &str,
              opts: &[OptGroup]) {
    println!("usage: {} [ options... ]", program_name);

    for opt in opts.iter() {
        println!("    -{}, --{:<10} {:<20} - {}",
                 opt.short_name, opt.long_name, opt.hint, opt.desc);
    }
}

impl Args {
    pub fn parse() -> Result<Args, int> {
        let args = os::args();
        let opts = &[
             optopt("s", "rng-seed", "set random number generator seed", "NUM,NUM,NUM,NUM"),
            optflag("h", "help",     "print this message and exit"),
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

        let mut rng_seed: [u32, ..4] = [1, 2, 3, 4];
        match matches.opt_str("s") {
            Some(arg) => parse_rng_seed(arg.as_slice(), &mut rng_seed),
            None => {}
        }

        Ok(Args {
            rng_seed: rng_seed
        })
    }
}


