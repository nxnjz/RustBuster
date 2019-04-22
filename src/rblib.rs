/* Copyright (C) 2019 nxnjz.
This file is part of RustBuster.
RustBuster is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.
RustBuster is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
You should have received a copy of the GNU General Public License
along with RustBuster. If not, see <http://www.gnu.org/licenses/>. */

use indicatif::ProgressBar;
use reqwest::{header, Client, RedirectPolicy};
use std::time::Duration;

pub struct Config {
    pub verbosity: u64,
    pub codes: Vec<usize>,
    pub timeout: Option<Duration>,
    pub ignore_cert: bool,
    pub redirect: usize,
    pub proxy_url: Option<String>,
    pub proxy_auth: Option<String>,
    pub retry_limit: u64,
}

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

pub fn tjob(
    i: usize,
    urllist: &[String],
    config: &Config,
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
        let mut attempt = 0;
        let mut resp = client.head(url).send();
        while resp.is_err() && attempt < config.retry_limit {
            bar_output(
                format!("Retrying for {}, [attempt {}]", url, attempt),
                3,
                &config.verbosity,
                bar,
            );
            resp = client.head(url).send();
            attempt += 1;
        }
        if resp.is_err() {
            bar_output(
                format!(
                    "[Retry Limit Reached] Gave up on {} after {} total attempts",
                    url,
                    attempt + 1
                ),
                2,
                &config.verbosity,
                bar,
            );
            bar.inc(1);
            continue;
        }

        let resp = resp.unwrap();
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
