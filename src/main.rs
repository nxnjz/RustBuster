use clap::{App, Arg, SubCommand};
use reqwest::Client;
use std::fs;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct Config {
    verbosity: u64,
    codes: Vec<usize>,
    timeout: Option<Duration>,
    proxy: Option<String>,
    customua: Option<String>,
    bauser: Option<String>,
    bapass: Option<String>,
}

fn main() {
    let args = App::new("RustBuster")
        .version("0.10")
        .author("Karl W. <karl@nxnjz.net>")
        .about("Directory/File Buster")
        .arg(
            Arg::with_name("dictionary")
                .short("d")
                .long("dic")
                .help("Dictionary file, items separated by newlines and/or spaces")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("Base URL")
                .short("u")
                .long("url")
                .help("Base URL on which items are appended.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("Extensions")
                .short("x")
                .long("ext")
                .help("Comma separated list of extensions to use.\nIf the provided value ends with a comma, a blank extension will also be used.\nExamples: .html,.php,.txt\n          .html,.php,.txt,\n          .php.bak,.php.old,.php,.PHP,.php5,")
                .takes_value(true)
                .required(false)
                .next_line_help(true),
        )
        .arg(
            Arg::with_name("Threads")
                .short("t")
                .long("threads")
                .help("Number of threads. Default: 12 or number of targets, whichever is smaller.")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("Status Codes")
                .short("s")
                .long("status-codes")
                .help("Comma separated list of status codes which should be considered success. Dashes can be used to specify ranges.\n Default: 200-299,301,302,403")
                .takes_value(true)
                .required(false)
                .next_line_help(true),
        ).arg(
            Arg::with_name("Verbosity")
                .short("v")
                .help("Verbosity level: -v or -vv or -vvv")
                .multiple(true)
                .takes_value(false)
                .required(false),
        ).arg(
            Arg::with_name("Timeout")
                .short("T")
                .long("timeout")
                .help("Total timeout for a request. Default: 30 seconds")
                .multiple(true)
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    //base url
    let mut base_url = args.value_of("Base URL").unwrap().to_string();
    if !base_url.ends_with("/") {
        base_url.push('/');
    }

    //threads
    let mut t_num: usize = args
        .value_of("Threads")
        .unwrap_or("12")
        .parse::<usize>()
        .unwrap_or(12);

    //input file
    let dic_filename = args.value_of("dictionary").unwrap_or("");
    let dic_str = fs::read_to_string(dic_filename)
        .expect(format!("Could not read {}", dic_filename).as_str());

    //extensions
    let ext_str = args.value_of("Extensions").unwrap_or("");

    //verbosity
    let verbosity = match args.occurrences_of("Verbosity") {
        0 => 1,
        _ => args.occurrences_of("Verbosity"),
    };

    //timeout
    let timeout_input = args
        .value_of("Timeout")
        .unwrap_or("30")
        .parse()
        .unwrap_or(30);
    let timeout: Option<Duration> = match timeout_input {
        0 => None,
        _ => Some(Duration::from_secs(timeout_input)),
    };

    //status codes
    let stat_codes = args
        .value_of("Status Codes")
        .unwrap_or("200-299,301,302,403");
    let mut codes: Vec<usize> = Vec::new();
    for code in stat_codes.split(',') {
        if code.contains('-') {
            let mut code_range = code.split('-');
            let start = code_range
                .next()
                .expect("Unable to parse status code range")
                .parse()
                .expect("Unable to parse status code range");
            let end = code_range
                .next()
                .expect("Unable to parse status code range")
                .parse()
                .expect("Unable to parse status code range");
            codes.append(&mut (start..=end).collect::<Vec<usize>>());
        } else {
            codes.push(code.parse().unwrap());
        }
    }

    //create urls
    let mut urls: Vec<String> = Vec::new();
    for i in dic_str.split_whitespace() {
        for j in ext_str.split(',') {
            urls.push(base_url.to_owned() + i + j);
        }
    }

    //reduce threads if not enough urls
    if urls.len() < t_num {
        t_num = urls.len();
    }

    println!("Will check {} URLs", urls.len());
    println!("Starting {} threads. \n\n\n", t_num);

    //generate vector of number of urls per thread, for splitting targets
    //tries to distribute them as equally as possible
    let mut url_per_thread = Vec::new();
    for _ in 0..(urls.len() % t_num) {
        url_per_thread.push(urls.len() / t_num + 1);
    }
    for _ in 0..(t_num - (urls.len() % t_num)) {
        url_per_thread.push(urls.len() / t_num);
    }

    //split urls to vec of vecs of strings
    let mut url_map = Vec::new();
    let mut start = 0;
    let mut end = 0;
    for i in 0..t_num {
        let current_num = url_per_thread[i];
        end = end + current_num;
        url_map.push(urls[start..end].to_vec());
        start = end;
    }

    let config = Config {
        verbosity: verbosity,
        codes: codes,
        timeout: timeout,
        proxy: None,
        customua: None,
        bauser: None,
        bapass: None,
    };

    //create config Arc
    let config = Arc::new(config);

    //create shared counter
    let mut progress = Arc::new(AtomicU64::new(0));
    //start threads
    let mut threads = Vec::new();
    for i in 0..t_num {
        let url_map_i = url_map[i as usize].clone();
        //clone config pointer and pass it
        let config = Arc::clone(&config);
        let progress = Arc::clone(&progress);
        threads.push(thread::spawn(move || {
            tjob(i, &url_map_i, &config, &progress);
        }));
    }

    //wait for threads
    for t in threads {
        let _ = t.join();
    }
}

fn tjob(i: usize, urllist: &[String], config: &Config, progress: &AtomicU64) {
    let client = Client::new();
    let client = Client::builder().timeout(config.timeout).build().unwrap();
    for url in urllist.iter() {
        print!("{} \r", progress.fetch_add(1, Ordering::Relaxed) + 1);
        let resp = client.head(url).send().expect("error");
        let resp_code: usize = resp
            .status()
            .to_string()
            .split_whitespace()
            .next()
            .unwrap_or(resp.status().as_str())
            .parse()
            .expect("Error parsing response code");
        if config.codes.contains(&resp_code) {
            output(format!("{} {}", url, resp.status()), 1, &config.verbosity);
        } else {
            output(format!("{} {}", url, resp.status()), 3, &config.verbosity);
        }
    }
}

fn output<T>(msg: T, msg_level: u64, verbosity_conf: &u64) -> ()
where
    T: std::fmt::Display,
{
    if msg_level <= *verbosity_conf {
        println!("{}", msg);
    }
}
