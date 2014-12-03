// Copyright (c) 2014 Dawid Ciężarkiewicz
// See LICENSE file for details

use hyper::mime;
use url::Url;
use hyper::client::Request;
use hyper::client::Response;
use hyper::HttpResult;
use hyper::HttpError;
use hyper::http::RawStatus;
use hyper::header::common::location::Location;
use hyper::header::HeaderFormatter;
use hyper::header::common::Connection;
use hyper::header::common::connection::ConnectionOption;
use hyper::header::common::user_agent::UserAgent;
use hyper::header::common::accept::Accept;

pub type FetcherResult = HttpResult<Response>;

pub trait HttpFetcher {
    fn get(&self, url : Url) -> FetcherResult;
}

pub struct BasicFetcher;

impl BasicFetcher {
    pub fn new() -> BasicFetcher {
        BasicFetcher
    }
}

impl HttpFetcher for BasicFetcher {
    fn get(&self, url : Url) -> FetcherResult
    {
        let mut req = try!(Request::get(url));

        // Setting a header.
        {
            let hm = req.headers_mut();
            let mime = mime::Mime(mime::TopLevel::Star, mime::SubLevel::Star, vec!());
            hm.set(Accept(vec![mime]));
            hm.set(Connection(vec![ConnectionOption::KeepAlive]));
            hm.set(UserAgent("Mozilla/5.0 (X11; CrOS x86_64 6158.70.0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/38.0.2125.110 Safari/537.36".to_string()));
        }

        let req = try!(req.start());

        req.send()
    }
}

pub struct FollowingFetcher<'a> {
    fetcher : &'a (HttpFetcher + 'a)
}

impl<'a> FollowingFetcher<'a> {
    pub fn new(fetcher : &'a (HttpFetcher + 'a)) -> FollowingFetcher<'a> {
        FollowingFetcher { fetcher: fetcher }
    }
}

impl<'a> HttpFetcher for FollowingFetcher<'a> {

    fn get(&self, url : Url) -> FetcherResult {
        let mut url = url;

        for _ in range(0u, 10) {
            let orig_url = url.clone();
            let res = try!(self.fetcher.get(url));
            {
                let &RawStatus(ref code, _) = res.status_raw();

                // if 301 Location contains the new address
                if *code == 301 {
                    let loc = res.headers.get::<Location>().unwrap();

                    let url_res = Url::parse(format!("{}", HeaderFormatter(loc)).as_slice());
                    match url_res {
                        Err(_) => return Err(HttpError::HttpUriError),
                        Ok(location_url) => {
                            println!("301: {} -> {}", orig_url, location_url);
                            url = location_url;
                        }
                    }
                    continue;
                }
            }
            return Ok(res);
        }
        Err(HttpError::HttpUriError)
    }
}

pub struct RetryingFetcher<'a> {
    retries : uint,
    fetcher : &'a (HttpFetcher + 'a)
}


impl<'a> RetryingFetcher<'a> {
    pub fn new(fetcher : &'a (HttpFetcher + 'a), retries : uint) -> RetryingFetcher<'a> {
        RetryingFetcher{ fetcher: fetcher, retries: retries }
    }
}

impl<'a> HttpFetcher for RetryingFetcher<'a> {

    fn get(&self, url : Url) -> FetcherResult {

        let mut attempt = 0u;
        loop {
            let res = self.fetcher.get(url.clone());

            match res {
                Ok(res) => return Ok(res),
                Err(err) => {
                    attempt += 1;
                    if attempt == self.retries {
                        return Err(err);
                    }
                }
            }
        }
    }
}
