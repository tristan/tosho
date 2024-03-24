use crate::curl;
use crate::rss::{self, element_text};
use chrono::{DateTime, NaiveDate, NaiveDateTime, ParseError as ChronoParseError};
use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use quick_xml::Error as XmlError;
use quick_xml::Reader;
use std::io::BufRead;

#[derive(Debug)]
pub enum Error {
    CurlError(curl::Error),
    RssError(rss::Error),
}

impl From<rss::Error> for Error {
    fn from(err: rss::Error) -> Self {
        Error::RssError(err)
    }
}

impl From<XmlError> for Error {
    fn from(err: XmlError) -> Error {
        Error::RssError(rss::Error::Xml(err))
    }
}

impl From<ChronoParseError> for Error {
    fn from(err: ChronoParseError) -> Error {
        Error::RssError(rss::Error::ChronoParseError(err))
    }
}

impl From<curl::Error> for Error {
    fn from(err: curl::Error) -> Error {
        Error::CurlError(err)
    }
}

#[derive(Debug)]
pub struct Item {
    pub title: String,
    pub link: String,
    pub category: String,
    pub pub_date: NaiveDateTime,
    pub description: String,
    pub guid: String,
}

impl Default for Item {
    fn default() -> Item {
        Item {
            title: String::default(),
            link: String::default(),
            category: String::default(),
            description: String::default(),
            pub_date: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            guid: String::default(),
        }
    }
}

fn read_from<R: BufRead>(reader: R) -> Result<Vec<Item>, Error> {
    let mut reader = Reader::from_reader(reader);
    reader.trim_text(true).expand_empty_elements(true);

    let mut buf = Vec::new();
    let mut items: Vec<Item> = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if let b"item" = e.name() {
                    let item = Item::from_xml(&mut reader, e.attributes())?;
                    items.push(item);
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    Ok(items)
}

impl Item {
    fn from_xml<R: BufRead>(reader: &mut Reader<R>, _: Attributes) -> Result<Self, Error> {
        let mut item = Item::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event(&mut buf)? {
                Event::Start(e) => match e.name() {
                    b"title" => item.title = element_text(reader)?,
                    b"link" => item.link = element_text(reader)?,
                    b"description" => item.description = element_text(reader)?,
                    b"pubDate" => {
                        item.pub_date =
                            DateTime::parse_from_rfc2822(&element_text(reader)?)?.naive_utc()
                    }
                    b"guid" => item.guid = element_text(reader)?,
                    n => {
                        reader.read_to_end(n, &mut Vec::new())?;
                    }
                },
                Event::End(_) => break,
                Event::Eof => return Err(Error::RssError(rss::Error::Eof)),
                _ => {}
            }
            buf.clear();
        }
        Ok(item)
    }
}

pub fn get_bookmarks(apikey: &str) -> Result<Vec<Item>, Error> {
    let response = curl::get(&format!("https://dognzb.cr/rss.cfm?r={apikey}&t=9000"))?;
    let links = read_from(&response.body[..])?;
    Ok(links)
}
