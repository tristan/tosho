use std::io::BufRead;
use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::events::attributes::Attributes;
use quick_xml::Error as XmlError;
use chrono::{NaiveDate, NaiveDateTime, DateTime, ParseError as ChronoParseError};

#[derive(Debug)]
pub enum Error {
    Eof,
    Xml(XmlError),
    MissingExpectedValue,
    ChronoParseError(ChronoParseError),
}

impl From<XmlError> for Error {
    fn from(err: XmlError) -> Error {
        Error::Xml(err)
    }
}

impl From<ChronoParseError> for Error {
    fn from(err: ChronoParseError) -> Error {
        Error::ChronoParseError(err)
    }
}

#[derive(Debug)]
pub struct Item {
    pub title: String,
    pub link: String,
    pub nzb_link: String,
    pub torrent_link: String,
    pub pub_date: NaiveDateTime,
    pub guid: String,
}

impl Default for Item {
    fn default() -> Item {
        Item {
            title: String::default(),
            link: String::default(),
            nzb_link: String::default(),
            torrent_link: String::default(),
            pub_date: NaiveDate::from_ymd(2000, 1, 1).and_hms(0, 0, 0),
            guid: String::default()
        }
    }
}

#[derive(Default)]
struct Enclosure {
    url: String,
    length: String,
    mime_type: String,
}

pub fn read_from<R: BufRead>(reader: R) -> Result<Vec<Item>, Error> {
    let mut reader = Reader::from_reader(reader);
    reader.trim_text(true).expand_empty_elements(true);

    let mut buf = Vec::new();
    let mut items: Vec<Item> = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"item" => {
                        let item = Item::from_xml(&mut reader, e.attributes())?;
                        items.push(item);
                    }
                    _ => (),
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
                    b"enclosure" => {
                        let enclosure = Enclosure::from_xml(reader, e.attributes())?;
                        match enclosure.mime_type.as_ref() {
                            "application/x-nzb" => {
                                item.nzb_link = enclosure.url;
                            }
                            "application/x-bittorrent" => {
                                item.torrent_link = enclosure.url;
                            },
                            _ => {}
                        }
                    }
                    b"title" => item.title = element_text(reader)?,
                    b"link" => item.link = element_text(reader)?,
                    b"pubDate" => item.pub_date = DateTime::parse_from_rfc2822(&element_text(reader)?)?
                        .naive_utc(),
                    b"guid" => {
                        let guid = element_text(reader)?;
                        if guid.starts_with("https://mirror.animetosho.org/view/") {
                            item.guid = guid[35..].to_string();
                        } else if guid.starts_with("https://animetosho.org/view/") {
                            item.guid = guid[28..].to_string();
                        }
                    }
                    n => {
                        reader.read_to_end(n, &mut Vec::new())?;
                    }
                }
                Event::End(_) => break,
                Event::Eof => return Err(Error::Eof),
                _ => {}
            }
            buf.clear();
        }
        Ok(item)
    }
}

impl Enclosure {
    fn from_xml<R: BufRead>(reader: &mut Reader<R>, mut attributes: Attributes) -> Result<Self, Error> {
        let mut enclosure = Enclosure::default();
        for attr in attributes.with_checks(false) {
            if let Ok(attr) = attr {
                match attr.key {
                    b"url" => {
                        enclosure.url = attr.unescape_and_decode_value(reader)?;
                    }
                    b"length" => {
                        enclosure.length = attr.unescape_and_decode_value(reader)?;
                    }
                    b"type" => {
                        enclosure.mime_type = attr.unescape_and_decode_value(reader)?;
                    }
                    _ => {}
                }
            }
        }
        reader.read_to_end(b"enclosure", &mut Vec::new())?;
        Ok(enclosure)
    }
}

pub fn element_text<R: BufRead>(reader: &mut Reader<R>) -> Result<String, Error> {
    let mut content: Option<String> = None;
    let mut buf = Vec::new();
    let mut skip_buf = Vec::new();

    loop {
        match reader.read_event(&mut buf)? {
            Event::Start(element) => {
                reader.read_to_end(element.name(), &mut skip_buf)?;
            }
            Event::CData(element) => {
                let text = reader.decode(&*element)?.to_string();
                content = Some(text);
            }
            Event::Text(element) => {
                let text = element.unescape_and_decode(reader)?;
                content = Some(text);
            }
            Event::End(_) | Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    match content {
        Some(c) => Ok(c),
        None => Err(Error::MissingExpectedValue)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_from_xml() {
        let data = r#"<?xml version="1.0" encoding="utf-8"?>
<rss xmlns:atom="http://www.w3.org/2005/Atom" version="2.0">
  <channel>
    <atom:link href="https://feed.animetosho.org/rss2" rel="self" type="application/rss+xml"/>
    <title>Anime Tosho</title>
    <link>https://animetosho.org/</link>
    <description>Latest releases feed</description>
    <language>en-gb</language>
    <ttl>30</ttl>
    <lastBuildDate>Sun, 01 Mar 2020 18:02:01 +0000</lastBuildDate>
    <item>
      <title>[Judas] Dorohedoro - 08 v2 [1080p][HEVC x265 10bit][Eng-Subs].mkv</title>
      <description><![CDATA[<strong>Total Size</strong>: 247.8 MB<br/><strong>Download Links</strong>: <a href="https://animetosho.org/storage/torrent/f0ca3d5cef269b6f2811da9196527dee47b79712/%5BJudas%5D%20Dorohedoro%20-%2008%20v2%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.torrent">Torrent</a>/<a href="magnet:?xt=urn:btih:6DFD2XHPE2NW6KAR3KIZMUT55ZD3PFYS&amp;tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&amp;dn=%5BJudas%5D%20Dorohedoro%20-%2008%20v2%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.mkv">Magnet</a>, <a href="https://animetosho.org/storage/nzbs/00053c86/%5BJudas%5D%20Dorohedoro%20-%2008%20v2%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.nzb.gz">NZB</a> | <a href="https://clicknupload.org/b0kp2jf3bs4p">ClickNUpload</a> | <a href="https://dropapk.to/p57jffj5e4k6">DropAPK</a> | <a href="https://download.jheberg.net/jplpcn1ghc6a">Jheberg</a> | <a href="https://multiup.org/download/51f9313f5ea9354cf99da76d08bc1bd2/_Judas__Dorohedoro_-_08_v2__1080p__HEVC_x265_10bit__Eng-Subs_.mkv">MultiUp</a> | <a href="https://www.sendspace.com/file/hjqshh">Sendspace</a> | <a href="https://www.solidfiles.com/v/555YZe4pNAwzB">SolidFiles</a> | <a href="https://www14.zippyshare.com/v/ygsiQL6w/file.html">ZippyShare</a>]]></description>
      <link>https://animetosho.org/view/judas-dorohedoro-08-v2-1080p-hevc-x265-10bit.1432917</link>
      <comments>https://animetosho.org/view/judas-dorohedoro-08-v2-1080p-hevc-x265-10bit.1432917</comments>
      <enclosure url="http://animetosho.org/storage/torrent/f0ca3d5cef269b6f2811da9196527dee47b79712/%5BJudas%5D%20Dorohedoro%20-%2008%20v2%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.torrent" type="application/x-bittorrent" length="0"/>
      <enclosure url="http://animetosho.org/storage/nzbs/00053c86/%5BJudas%5D%20Dorohedoro%20-%2008%20v2%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.nzb" type="application/x-nzb" length="0"/>
      <source url="https://www.tokyotosho.info/details.php?id=1432917">TokyoTosho</source>
      <pubDate>Sun, 01 Mar 2020 17:46:01 +0000</pubDate>
      <guid isPermaLink="true">https://animetosho.org/view/a343174</guid>
    </item>
    <item>
      <title>[Judas] Dorohedoro - 09 [1080p][HEVC x265 10bit][Eng-Subs]</title>
      <description><![CDATA[<strong>Total Size</strong>: 283.5 MB<br/><strong>Download Links</strong>: <a href="https://animetosho.org/storage/torrent/c9132ccb1d4d0bc5f29e1064143a96e468cf2870/%5BJudas%5D%20Dorohedoro%20-%2009%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.torrent">Torrent</a>/<a href="magnet:?xt=urn:btih:ZEJSZSY5JUF4L4U6CBSBIOUW4RUM6KDQ&amp;tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&amp;dn=%5BJudas%5D%20Dorohedoro%20-%2009%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D">Magnet</a>, <a href="https://animetosho.org/storage/nzbs/00053fd0/%5BJudas%5D%20Dorohedoro%20-%2009%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.nzb.gz">NZB</a> | <a href="https://clicknupload.org/o4sbe4uz8r3e">ClickNUpload</a> | <a href="https://dropapk.to/zj82ua2hopax">DropAPK</a> | <a href="https://multiup.org/download/2e2c82ceb65c4e28ddbe0f383892784b/_Judas__Dorohedoro_-_09__1080p__HEVC_x265_10bit__Eng-Subs_.mkv">MultiUp</a> | <a href="https://www.sendspace.com/file/lijzla">Sendspace</a> | <a href="https://www.solidfiles.com/v/keXDqVqBzXx72">SolidFiles</a> | <a href="https://www81.zippyshare.com/v/slMMiJ5h/file.html">ZippyShare</a>]]></description>
      <link>https://animetosho.org/view/judas-dorohedoro-09-1080p-hevc-x265-10bit-eng-subs.n1227240</link>
      <comments>https://animetosho.org/view/judas-dorohedoro-09-1080p-hevc-x265-10bit-eng-subs.n1227240</comments>
      <enclosure url="http://animetosho.org/storage/torrent/c9132ccb1d4d0bc5f29e1064143a96e468cf2870/%5BJudas%5D%20Dorohedoro%20-%2009%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.torrent" type="application/x-bittorrent" length="0"/>
      <enclosure url="http://animetosho.org/storage/nzbs/00053fd0/%5BJudas%5D%20Dorohedoro%20-%2009%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.nzb" type="application/x-nzb" length="0"/>
      <source url="https://nyaa.si/view/1227240">Nyaa</source>
      <pubDate>Sun, 08 Mar 2020 08:56:16 +0000</pubDate>
      <guid isPermaLink="true">https://animetosho.org/view/a344016</guid>
    </item>
  </channel>
</rss>
"#;

        //let mut reader = quick_xml::Reader::from_str(&data);
        // let attrs = Attributes::new(&[], 0);
        // let item = Item::from_xml(&mut reader, attrs).unwrap();
        let reader = Cursor::new(data.as_bytes());
        let items = read_from(reader).unwrap();
        let item = &items[0];
        assert_eq!(item.title, "[Judas] Dorohedoro - 08 v2 [1080p][HEVC x265 10bit][Eng-Subs].mkv");
        assert_eq!(item.nzb_link, "http://animetosho.org/storage/nzbs/00053c86/%5BJudas%5D%20Dorohedoro%20-%2008%20v2%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.nzb");

        let item = &items[1];
        assert_eq!(item.title, "[Judas] Dorohedoro - 09 [1080p][HEVC x265 10bit][Eng-Subs]");
        assert_eq!(item.nzb_link, "http://animetosho.org/storage/nzbs/00053fd0/%5BJudas%5D%20Dorohedoro%20-%2009%20%5B1080p%5D%5BHEVC%20x265%2010bit%5D%5BEng-Subs%5D.nzb");
    }
}
