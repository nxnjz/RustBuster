mod blib;

use base64::encode as b64;
use blib::*;
use clap::{App, Arg, SubCommand};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Client, RedirectPolicy};
use std::sync::{atomic::AtomicU64, atomic::Ordering, Arc};
use std::{fs, thread, time::Duration};

struct Config {
    verbosity: u64,
    codes: Vec<usize>,
    timeout: Option<Duration>,
    ignore_cert: bool,
    redirect: usize,
    proxy_url: Option<String>,
    proxy_auth: Option<String>,
}

fn main() {
    let app_ver = "0.11";
    let app_name = "RustBuster";

    let args = App::new(app_name)
        .version(app_ver)
        .author("Karl W. <karl@nxnjz.net>")
        .about("Multithreaded Directory/File Buster")
        .arg(
            Arg::with_name("dictionary")
                .short("d")
                .long("dic")
                .help("Dictionary file, items separated by newlines and/or spaces")
                .takes_value(true)
                .required(true)
                .display_order(2),
        )
        .arg(
            Arg::with_name("Base URL")
                .short("u")
                .long("url")
                .help("Base URL on which items are appended.")
                .takes_value(true)
                .required(true)
                .display_order(1),
        )
        .arg(
            Arg::with_name("Extensions")
                .short("x")
                .long("ext")
                .help("Comma separated list of extensions to use.\nIf the provided value ends with a comma, a blank extension will also be used (no extension).\nExamples: .html,.php,.txt\n          .html,.php,.txt,\n          .php.bak,.php.old,.php,.PHP,.php5,")
                .takes_value(true)
                .required(false)
                .display_order(3)
                .next_line_help(true),
        )
        .arg(
            Arg::with_name("Threads")
                .short("t")
                .long("threads")
                .help("Number of threads. Default: 12.\nPlease keep OS limits in mind, setting a high number may cause some threads to crash.")
                .next_line_help(true)
                .takes_value(true)
                .display_order(4)
                .required(false),
        )
        .arg(
            Arg::with_name("Status Codes")
                .short("s")
                .long("status-codes")
                .help("Comma separated list of status codes which should be considered success. Dashes can be used to specify ranges.\n Default: 200-299,301,302,403")
                .takes_value(true)
                .required(false)
                .display_order(5)
                .next_line_help(true),
        ).arg(
            Arg::with_name("Verbosity")
                .short("v")
                .help("Verbosity level: -v or -vv or -vvv. ")
                .multiple(true)
                .takes_value(false)
                .required(false),
        ).arg(
            Arg::with_name("Timeout")
                .short("T")
                .long("timeout")
                .help("Total timeout for a request. Default: 30 seconds")
                .multiple(true)
                .takes_value(true)
                .required(false),
        ).arg(
            Arg::with_name("Cookie List")
                .short("c")
                .long("cookie")
                .help("Optional cookie list in the form of \"name=value; name2=value2; name3=value3;\"")
                .multiple(false)
                .takes_value(true)
                .required(false),
        ).arg(
            Arg::with_name("User Agent")
                .long("user-agent")
                .help("Custom User Agent")
                .multiple(false)
                .takes_value(true)
                .required(false),
        ).arg(
            Arg::with_name("Ignore HTTPS Certificate Errors")
                .long("unsafe-https")
                .help("Set this option to ignore invalid hostnames/certificates")
                .multiple(false)
                .takes_value(false)
                .required(false)
        ).arg(
            Arg::with_name("Redirect Limit")
                .short("r")
                .long("redirect-limit")
                .help("Set the maximum number of redirects to follow. Default: 0")
                .multiple(false)
                .takes_value(true)
                .required(false)
        ).arg(
            Arg::with_name("Proxy")
                .short("p")
                .long("proxy")
                .help("Use a proxy for http and https in one of the following formats:\n http(s)://myproxy.net:port\nuser:pass@http(s)://myproxy.tld:port")
                .multiple(false)
                .takes_value(true)
                .required(false)
            ).arg(
            Arg::with_name("Basic Auth")
                .short("b")
                .long("basic-auth")
                .help("Set credentials for HTTP basic authentication in the format username:password")
                .multiple(false)
                .takes_value(true)
                .required(false)
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
    if dic_str.is_empty() {
        println!("{} seems empty, exiting...", dic_filename);
        panic!("Intentional panic");
    }

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

    //UA
    let client_ua = (args
        .value_of("User Agent")
        .unwrap_or(&(app_name.to_string() + "/" + app_ver)))
    .to_string();

    //cookies
    let cookies = if args.is_present("Cookie List") {
        Some(
            args.value_of("Cookie List")
                .expect("error parsing cookie list"),
        )
    } else {
        None
    };

    //ignore cert errors
    let ignore_cert = args.is_present("Ignore HTTPS Certificate Errors");

    //max redirects
    let redir_limit: usize = args
        .value_of("Redirect Limit")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    //proxy parsing
    let proxy_input = args.value_of("Proxy");
    #[allow(unused_assignments)]
    let mut proxy_url: Option<String> = None;
    #[allow(unused_assignments)]
    let mut proxy_auth: Option<String> = None;
    if proxy_input.is_some() && proxy_input.unwrap().contains('@') {
        proxy_auth = proxy_input
            .unwrap()
            .split('@')
            .nth(0)
            .map(|x| x.to_string());
        proxy_url = proxy_input
            .unwrap()
            .split('@')
            .nth(1)
            .map(|x| x.to_string());
    } else if proxy_input.is_some() {
        proxy_auth = None;
        proxy_url = proxy_input.map(|x| x.to_string());
    } else {
        proxy_url = None;
        proxy_auth = None;
    }
    println!("{:?}{:?}", proxy_auth, proxy_url);

    //basic auth credentials
    let basic_auth = args
        .value_of("Basic Auth")
        .map(|x| "Basic ".to_string() + &b64(x));
    println!("{:?}", basic_auth);

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
                .expect("[err 11]Unable to parse status code range")
                .parse()
                .expect("[err 12]Unable to parse status code range");
            let end = code_range
                .next()
                .expect("[err 13]Unable to parse status code range")
                .parse()
                .expect("[err 14]Unable to parse status code range");
            codes.append(&mut (start..=end).collect::<Vec<usize>>());
        } else {
            codes.push(code.parse().expect("[err 15]Unable to parse status code"));
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
        ignore_cert: ignore_cert,
        redirect: redir_limit,
        proxy_url: proxy_url,
        proxy_auth: proxy_auth,
    };

    //setup headers
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, client_ua.parse().unwrap());
    headers.insert(header::CONNECTION, "keep-alive".parse().unwrap());
    if cookies.is_some() {
        headers.insert(header::COOKIE, cookies.unwrap().parse().unwrap());
    }
    if basic_auth.is_some() {
        headers.insert(header::AUTHORIZATION, basic_auth.unwrap().parse().unwrap());
    }

    //create headers Arc
    let headers = Arc::new(headers);
    //create config Arc
    let config = Arc::new(config);

    //create shared counter
    let progress = Arc::new(AtomicU64::new(0));
    //create bar and put it in Arc
    let bar = ProgressBar::new(urls.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar().template("[{elapsed_precise}] {wide_bar} {pos}/{len}"),
    );
    bar.tick();
    let bar = Arc::new(bar);
    //vec for storing threads
    let mut threads = Vec::new();
    for i in 0..t_num {
        let url_map_i = url_map[i as usize].clone();
        //clone pointers
        let config = Arc::clone(&config);
        let progress = Arc::clone(&progress);
        let headers = Arc::clone(&headers);
        let bar = Arc::clone(&bar);
        //spawn threads
        threads.push(thread::spawn(move || {
            tjob(i, &url_map_i, &config, &progress, &headers, &bar);
        }));
    }

    //wait for threads to finish
    for t in threads {
        let _ = t.join();
    }
}

fn tjob(
    i: usize,
    urllist: &[String],
    config: &Config,
    progress: &AtomicU64,
    headers: &header::HeaderMap,
    bar: &ProgressBar,
) {
    output(format!("Thread {} started", i), 2, &config.verbosity);
    //let mut proxy_u = "";
    //let mut proxy_p = "";
    let mut proxy_url = String::new();
    let mut clientbuild = Client::builder();
    if config.proxy_auth.is_some() {
        let proxy_auth = config.proxy_auth.clone().unwrap();
        let proxy_u = proxy_auth.split(':').nth(0).unwrap();
        let proxy_p = proxy_auth.split(':').nth(1).unwrap();
        proxy_url = config.proxy_url.clone().unwrap();
        clientbuild = clientbuild.proxy(
            reqwest::Proxy::all(&proxy_url)
                .unwrap()
                .basic_auth(proxy_u, proxy_p),
        );
    } else if config.proxy_url.is_some() {
        proxy_url = config.proxy_url.clone().unwrap();
        clientbuild = clientbuild.proxy(reqwest::Proxy::all(&proxy_url).unwrap());
    }

    let redir_limit = config.redirect.clone();
    let redir_pol = RedirectPolicy::custom(move |attempt| {
        if attempt.previous().len() > redir_limit {
            attempt.stop()
        } else {
            attempt.follow()
        }
    });
    let client = clientbuild
        .timeout(config.timeout)
        .default_headers(headers.to_owned())
        .redirect(redir_pol)
        .danger_accept_invalid_hostnames(config.ignore_cert)
        .danger_accept_invalid_certs(config.ignore_cert)
        .build()
        .expect("[Err 51]Error configuring HTTP client");
    for url in urllist.iter() {
        let resp = client
            .head(url)
            .send()
            .expect("[Err 41]Error sending HTTP request");
        let resp_code: usize = resp
            .status()
            .to_string()
            .split_whitespace()
            .next()
            .expect("[Err 31]Error parsing response code")
            .parse()
            .expect("[Err 32]Error parsing response code");
        if config.codes.contains(&resp_code) {
            bar_output(
                format!("{} {}", url, resp.status()),
                1,
                &config.verbosity,
                bar,
            );
        } else {
            bar_output(
                format!("{} {}", url, resp.status()),
                3,
                &config.verbosity,
                bar,
            );
        }
        bar.inc(1);
    }
}
