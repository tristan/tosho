use url::Url;
use serde::Deserialize;
use crate::curl;

#[derive(Debug)]
pub enum Error {
    UrlParseError(url::ParseError),
    CurlError(curl::Error),
    StatusFalse(String),
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::UrlParseError(err)
    }
}

impl From<curl::Error> for Error {
    fn from(err: curl::Error) -> Error {
        Error::CurlError(err)
    }
}

pub struct SabnzbdClient {
    host: String,
    apikey: String
}

#[derive(Debug,Deserialize)]
struct JsonResponse {
    status: bool,
    error: Option<String>,
    nzo_ids: Option<Vec<String>>
}

fn handle_response(json: JsonResponse) -> Result<(), Error> {
    if !json.status {
        if let Some(error) = json.error {
            Err(Error::StatusFalse(error))
        } else {
            Err(Error::StatusFalse("Unknown error".to_string()))
        }
    } else {
        Ok(())
    }
}

impl SabnzbdClient {

    pub fn new(host: &str, apikey: &str) -> SabnzbdClient {
        // TODO: sanitize host to start with http and not end with /
        SabnzbdClient {
            host: host.to_string(),
            apikey: apikey.to_string()
        }
    }

    fn base_url(&self) -> Result<Url, Error> {
        let url = Url::parse(&self.host)?;
        let mut url = url.join("api").unwrap();
        url.query_pairs_mut()
            .append_pair("apikey", &self.apikey)
            .append_pair("output", "json");
        Ok(url)
    }

    // https://sabnzbd.org/wiki/advanced/api#addurl
    pub fn addurl(&self, nzb_url: &str, cat: &str) -> Result<(), Error> {
        let mut url = self.base_url()?;
        url.query_pairs_mut()
            .append_pair("mode", "addurl")
            .append_pair("name", nzb_url)
            .append_pair("cat", cat);

        let response = curl::get(url)?.json()?;
        handle_response(response)
    }
}
