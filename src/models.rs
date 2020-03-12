use std::str::FromStr;
use std::string::ToString;
use std::convert::From;

#[allow(non_camel_case_types)]
#[derive(Debug,PartialEq)]
pub enum Quality {
    Low_480p,
    Mid_720p,
    HD_1080p
}

impl FromStr for Quality {
    type Err = ();

    fn from_str(s: &str) -> Result<Quality, ()> {
        match s {
            "480p" | "480" | "LOW" | "low" | "Low" | "LQ" | "Lq" | "lq" =>
                Ok(Quality::Low_480p),
            "720p" | "720" | "MID" | "mid" | "Mid" =>
                Ok(Quality::Mid_720p),
            "1080p" | "1080" | "HD" | "hd" | "Hd" =>
                Ok(Quality::HD_1080p),
            _ => Err(()),
        }
    }
}

impl ToString for Quality {
    fn to_string(&self) -> String {
        match *self {
            Quality::Low_480p => String::from("480p"),
            Quality::Mid_720p => String::from("720p"),
            Quality::HD_1080p => String::from("1080p")
        }
    }
}

#[derive(Debug)]
pub struct Episode {
    pub group: String,
    pub name: String,
    pub quality: Quality,
    pub episode: i32,
    pub version: i32,
    pub extension: Option<String>
}

impl Episode {
    pub fn from(caps: &regex::Captures) -> Option<Episode> {
        let group = caps.name("group")?
            .as_str().to_string();
        let name = caps.name("name")?
            .as_str().to_string();
        let episode = caps.name("episode")?
            .as_str()
            .parse::<i32>()
            .unwrap_or(0);
        let version = caps.name("version")
            .map(|v| {
                v.as_str()
                    .parse::<i32>()
                    .unwrap()
            })
            .unwrap_or(1);
        let quality = caps.name("quality")?
            .as_str()
            .parse::<Quality>()
            .ok()?;
        let extension = caps.name("ext")
            .map(|e| e.as_str().to_string());
        Some(Episode {
            group, name, quality, episode, version, extension
        })
    }
}
