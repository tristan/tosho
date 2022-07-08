use crate::curl;
use crate::rss;

const ANIMETOSHO_RSS_URL: &str = "https://feed.animetosho.org/rss2";

#[derive(Debug)]
pub enum Error {
    Io(::std::io::Error),
    CurlError(curl::Error),
    RssError(rss::Error),
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<curl::Error> for Error {
    fn from(err: curl::Error) -> Error {
        Error::CurlError(err)
    }
}

impl From<rss::Error> for Error {
    fn from(err: rss::Error) -> Error {
        Error::RssError(err)
    }
}

pub fn search(terms: &str, page: Option<u8>) -> Result<Vec<rss::Item>, Error> {
    let query = terms.split(' ')
        .collect::<Vec<&str>>()
        .join("+");
    let page = page.unwrap_or(1);
    let url = &[ANIMETOSHO_RSS_URL, "?q=", &query, "&page=", &page.to_string()].join("");
    results_from_url(url)
}

pub fn feed(page: &u8) -> Result<Vec<rss::Item>, Error> {
    results_from_url(&[ANIMETOSHO_RSS_URL, "?page=", &page.to_string()].join(""))
}

fn results_from_url(url: &str) -> Result<Vec<rss::Item>, Error> {
    let response = curl::get(url)?;
    let items = rss::read_from(&response.body[..])?;
    Ok(items)
}
