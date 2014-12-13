macro_rules! panic_bt(
    ($($arg:tt)*) => ({
        let _ = ::std::rt::backtrace::write(&mut ::std::io::stdio::stderr());
        panic!($($arg)*);
    })
)

macro_rules! println_err(
    ($($arg:tt)*) => ({
        let mut stderr = ::std::io::stdio::stderr();
        match format_args!(|args| stderr.write_fmt(args), $($arg)*) {
            Err(_) => { panic_bt!("println_err failed"); },
            _ => {}
        }
        match stderr.write_str("\n") {
            Err(_) => { panic_bt!("println_err failed"); },
            _ => {}
        }
    })
)

macro_rules! time_it(
    ($name:expr, $limit:expr, $expr:block) => ({
        if ($limit) as f64 == 0.0 {
            println_err!("{}: start", $name);
        };
        let __start_time = time::precise_time_s();
        let __ret = $expr;
        let __end_time = time::precise_time_s();
        let __elapsed = __end_time - __start_time;

        if ($limit) as f64 == 0.0 {
            println_err!("{}: {}s", $name, __end_time - __start_time);
        } else if __elapsed > (($limit) as f64) * 3.0 {
            panic_bt!("{}: time limit ({}s) exceeded over 3x: {}s", $name, $limit, __elapsed);
        } else if __elapsed > ($limit) as f64 {
            println_err!("warning: {}: time limit ({}s) exceeded: {}s", $name, $limit, __elapsed);
        }
        __ret
    })
)

