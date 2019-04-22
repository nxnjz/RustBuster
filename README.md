# RustBuster

RustBuster is a multithreaded CLI tool for bruteforcing files and/or directories on HTTP(s) servers, similar to GoBuster, DirBuster, and Dirb.

Features:

* Auto-appends file extensions.
* Custom status code filtering.
* Custom User-Agents, cookies, and basic authorization.
* Custom timeouts and redirection limits.
* Proxy support (with and without authentication).

# Installation

## Compiling from source:

Installing from source requires Cargo. Refer to https://doc.rust-lang.org/cargo/getting-started/installation.html

First, clone this gitlab repo:

`git clone https://gitlab.com/nxnjz/rustbuster.git` 

Build using Cargo:

`cargo build --release`



