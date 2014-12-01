// Copyright (c) 2014 Dawid Ciężarkiewicz
// See LICENSE file for details

extern crate hyper;
extern crate url;
extern crate getopts;
use getopts::{optopt,optflag,getopts,OptGroup};
use std::os;

use std::error::Error;

use std::io::{stdout, stderr};
use std::io::util::copy;
use fetcher::{HttpFetcher,BasicFetcher,RetryingFetcher,FollowingFetcher};
use url::Url;

mod fetcher;

fn print_usage(program: &str, opts: &[OptGroup]) {
    println!("{}", getopts::short_usage(program, opts));
    println!("-L --follow\tFollow redirects");
    println!("-r --retry\tRetry");
    println!("-h --help\tUsage");
}


fn main() {

    // By default print everything to stderr;
    // only real output should go to stdout
    std::io::stdio::set_stdout(box stderr());

    let args: Vec<String> = os::args();

    let program = args[0].clone();

    let opts = &[
        optopt("r", "retry", "retry", "TIMES"),
        optflag("L", "location", "follow redirects"),
        optflag("h", "help", "print this help menu")
    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(program.as_slice(), opts);
        return;
    }


    let f = BasicFetcher::new();
    let mut rf = None;

    // TODO: Use `if let` + `opt_default` to handle this.
    if matches.opt_present("r") {
        let attempts_str = matches.opt_str("r").unwrap_or("3".into_string());
        let attempts = from_str::<uint>(attempts_str.as_slice());

        match attempts {
            None => panic!("Incorrect retry count: {}", attempts_str),
            Some(v) => rf = Some(RetryingFetcher::new(&f, v)),
        }
    }

    // TODO: There's got to be a better way
    let f = rf.as_ref().map(|f| f as &HttpFetcher).unwrap_or(&f as &HttpFetcher);


    let mut rf = None;

    if matches.opt_present("L") {
        rf = Some(FollowingFetcher::new(f));
    }

    let f = rf.as_ref().map(|f| f as &HttpFetcher).unwrap_or(f);

    if matches.free.is_empty() {
        print_usage(program.as_slice(), opts);
        std::os::set_exit_status(1);
        return;
    };


    // TODO: stop on error
    let mut urls = matches.free.iter().filter_map(
        |s| Url::parse(s.as_slice()).map_err(
            |err| {
                println!("Malformed URL: {} ({})", s, err);
                std::os::set_exit_status(1);
                err
            }
            ).ok()
        );

    for url in urls {
        match f.get(url) {
            Err(e) => {
                println!("{}: {}", e.detail(), e.description());
            },
            Ok(mut r) => {
                match copy(&mut r, &mut stdout()) {
                    Ok(..) => (),
                    Err(e) => panic!("Stream failure: {}", e)
                }
            }
        }
    }
}
