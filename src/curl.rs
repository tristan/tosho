use curl::easy::Easy;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub enum Error {
    CurlError(curl::Error),
    JsonError(serde_json::Error),
    Utf8Error,
}

impl From<curl::Error> for Error {
    fn from(err: curl::Error) -> Error {
        Error::CurlError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Error {
        Error::Utf8Error
    }
}

pub fn get(url: impl AsRef<str>) -> Result<Response, Error> {
    let mut easy = Easy::new();
    let mut body = Vec::new();
    {
        easy.url(url.as_ref())?;
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            body.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    let code = easy.response_code()?;

    Ok(Response { code, body })
}

pub struct Response {
    pub code: u32,
    pub body: Vec<u8>,
}

impl Response {
    pub fn json<T: DeserializeOwned>(self) -> Result<T, Error> {
        let data = String::from_utf8(self.body)?;
        let res: T = serde_json::from_str(&data)?;
        Ok(res)
    }
}
