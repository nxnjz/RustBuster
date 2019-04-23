# RustBuster

RustBuster is a multithreaded CLI tool for bruteforcing files and/or directories on HTTP(s) servers, similar to GoBuster, DirBuster, and Dirb.

** RustBuster is still a newborn, but works well enough in most cases. **

Features:

* Auto-appends file extensions.
* Custom status code filtering.
* Custom User-Agents, cookies, and basic authorization.
* Custom timeouts and redirection limits.
* Proxy support (with and without authentication).
* Nice Progress Bar

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
        --unsafe-https    Set this option to ignore invalid hostnames/certificates
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
    -R, --retry-count <Retry Count>          Set the maximum number of requests for a single target (in case of
                                             timeouts, or other errors). Default is 0.
    -T, --timeout <Timeout>...               Total timeout for a request. Default: 30 seconds
        --user-agent <User Agent>            Custom User Agent
    -b, --basic-auth <basic auth>            set credentials for http basic authentication in the format
                                             username:password


```






