macro_rules! println_err(
    ($($arg:tt)*) => ({
        let mut stderr = ::std::io::stdio::stderr();
        match format_args!(|args| stderr.write_fmt(args), $($arg)*) {
            Err(_) => { panic!("println_err failed"); },
            _ => {}
        }
        match stderr.write_str("\n") {
            Err(_) => { panic!("println_err failed"); },
            _ => {}
        }
    })
)

macro_rules! time_it(
    ($name:expr, $limit:expr, $expr:block) => ({
        println!("{}: start", $name);
        let __start_time = time::precise_time_s();
        let __ret = $expr;
        let __end_time = time::precise_time_s();
        println!("{}: {}s", $name, __end_time - __start_time);
        if ($limit) as f64 > 0.0 && __end_time - __start_time > ($limit) as f64 {
            panic!("time limit ({}s) exceeded", $limit);
        }
        __ret
    });
)

