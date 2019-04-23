# RustBuster

RustBuster is a simple multithreaded CLI tool for bruteforcing files and/or directories on HTTP(s) servers, similar to GoBuster, DirBuster, and Dirb.

**RustBuster is still a newborn, but works well enough in most cases.**

Features:

* Auto-appends file extensions.
* Custom status code filtering.
* Custom User-Agents, cookies, and basic authorization.
* Custom timeouts and redirection limits.
* Proxy support (with and without authentication).
* Nice Progress Bar!

# Installation

## Compiling from source:

Installing from source requires Cargo. Refer to https://doc.rust-lang.org/cargo/getting-started/installation.html for installing Cargo.

First, clone this repo:

`git clone https://gitlab.com/nxnjz/rustbuster.git` 

Build and Install using Cargo:

`cargo install --path rustbuster/`

# Usage

#### Bare minimum (no file extensions, for bruteforcing directories): 

`rustbuster -u https://yoursite.net/ -w /usr/share/wordlists/dirb/small.txt`

#### With file extensions:

`rustbuster -u https://yoursite.net/ -w /usr/share/wordlists/dirb/small.txt -x .html,.php,.txt`

#### With file extensions and blank extension (for bruteforcing directories):

`rustbuster -u https://yoursite.net/ -w /usr/share/wordlists/dirb/small.txt -x .html,.php,.txt,`


#### From rustbuster --help:

```
USAGE:
    rustbuster [FLAGS] [OPTIONS] --url <Base URL> --wordlist <dictionary>

FLAGS:
    -U, --unsafe-https    Set this option to ignore invalid hostnames and certificate errors
    -v                    Verbosity level: -v or -vv or -vvv.
    -h, --help            Prints help information
    -V, --version         Prints version information

OPTIONS:
    -u, --url <Base URL>                     Base URL on which items are appended.
    -w, --wordlist <dictionary>              Dictionary file, items separated by newlines and/or spaces
    -x, --ext <Extensions>
            Comma separated list of extensions to use.
            If the provided value ends with a comma, a blank extension will also be used (no extension).
            Examples: .html,.php,.txt
                      .html,.php,.txt,
                      .php.bak,.php.old,.php,.PHP,.php5,
    -t, --threads <Threads>
            Number of threads. Default: 12.
            Please keep OS limits in mind, setting a high number may cause some threads to crash.
    -s, --status-codes <Status Codes>
            Comma separated list of status codes which should be considered success. Dashes can be used to specify
            ranges.
             Default: 200-299,301,302,403
    -c, --cookie <Cookie List>               Optional cookie list in the form of "name=value; name2=value2;
                                             name3=value3;"
    -p, --proxy <Proxy>                      Use a proxy for http and https in one of the following formats:
                                             http(s)://myproxy.net:port
                                             user:pass@http(s)://myproxy.tld:port
    -r, --redirect-limit <Redirect Limit>    Set the maximum number of redirects to follow. Default: 0
    -R, --retry-count <Retry Count>          Set the maximum number of tries for a single request (applies in case of
                                             timeouts, or other errors). Default is 0.
    -T, --timeout <Timeout>...               Total timeout for a request. Default: 30 seconds
    -a, --user-agent <User Agent>            Custom User Agent
    -b, --basic-auth <basic auth>            set credentials for http basic authentication in the format
                                             username:password

```

# NOTES

RustBuster started as a way for me to learn rust, expect inconsistencies and inefficiencies in the code. 

Constructive criticism is very welcome. 

Thanks to [reqwest]("https://github.com/seanmonstar/reqwest"), [clap]("https://github.com/clap-rs/clap"), [indicatif]("https://github.com/mitsuhiko/indicatif"), [base64]("https://docs.rs/base64/0.10.1/base64/").


# BENCHMARKS

The following are some basic, single-iteration tests comparing RustBuster and GoBuster. All tests were performed on a 1vCPU 1GB Debian 9 VM with a rather stable 1Gbps connection. The wordlist used contained 20469 words. Timeout was set to 60 seconds on both tools. 

## Single thread
```
CMD: gobuster -w big1.txt -u https://google.com -t 1 -to 60s
TIME: 0m42.487s

CMD: rustbuster -w big1.txt -u https://google.com -t 1 -T 60
TIME: 0m35.608s
```
## 10 threads
```
CMD: gobuster -w big1.txt -u https://google.com -t 10 -to 60s
TIME: 0m20.222s

CMD: rustbuster -w big1.txt -u https://google.com -t 10 -T 60
TIME: 0m5.130s
```
## 20 threads
```
CMD: gobuster -w big1.txt -u https://google.com -t 20 -to 60s
TIME: 0m25.641s

CMD: rustbuster -w big1.txt -u https://google.com -t 20 -T 60
TIME: 0m4.447s
```
## 50 threads
```
CMD: gobuster -w big1.txt -u https://google.com -t 50 -to 60s
TIME: 0m27.564s

CMD: rustbuster -w big1.txt -u https://google.com -t 50 -T 60
TIME: 0m3.435s
```
## 100 threads
```
CMD: gobuster -w big1.txt -u https://google.com -t 100 -to 60s
TIME: 0m28.535s

CMD: rustbuster -w big1.txt -u https://google.com -t 100 -T 60
TIME: 0m5.243s
```

