use indicatif::ProgressBar;

pub fn output<T>(msg: T, msg_level: u64, &verbosity_conf: &u64) -> ()
where
    T: std::fmt::Display,
{
    if msg_level <= verbosity_conf {
        println!("{}", msg);
    }
}

pub fn bar_output<T>(msg: T, msg_level: u64, &verbosity_conf: &u64, bar: &ProgressBar) -> ()
where
    T: Into<String>,
{
    if msg_level <= verbosity_conf {
        bar.println(msg);
    }
}
