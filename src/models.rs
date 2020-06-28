use std::str::FromStr;
use std::string::ToString;
use std::convert::From;
use std::error::Error;
use postgres::types::{self, ToSql, FromSql};

#[allow(non_camel_case_types)]
#[derive(Debug,PartialEq)]
pub enum Quality {
    Low_480p,
    Mid_720p,
    HD_1080p
}

#[derive(Debug)]
pub struct BadQuality;
impl std::fmt::Display for BadQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for BadQuality {}

impl FromStr for Quality {
    type Err = BadQuality;

    fn from_str(s: &str) -> Result<Quality, BadQuality> {
        match s {
            "480p" | "480" | "LOW" | "low" | "Low" | "LQ" | "Lq" | "lq" =>
                Ok(Quality::Low_480p),
            "720p" | "720" | "MID" | "mid" | "Mid" =>
                Ok(Quality::Mid_720p),
            "1080p" | "1080" | "HD" | "hd" | "Hd" =>
                Ok(Quality::HD_1080p),
            _ => Err(BadQuality),
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

impl FromSql for Quality {
    fn from_sql(
        ty: &types::Type,
        raw: &[u8]
    ) -> Result<Self, Box<dyn Error + 'static + Send + Sync>> {
        let s: String = String::from_sql(ty, raw)?;
        Ok(Quality::from_str(&s)?)
    }

    fn accepts(ty: &types::Type) -> bool {
        <String as FromSql>::accepts(ty)
    }
}

impl ToSql for Quality {
    fn to_sql(
        &self,
        ty: &types::Type,
        out: &mut Vec<u8>
    ) -> Result<types::IsNull, Box<dyn Error + Sync + Send>> {
        (*self).to_string().to_sql(ty, out)
    }

    fn accepts(ty: &types::Type) -> bool {
        <String as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

#[derive(Debug)]
pub struct Episode {
    pub group: String,
    pub name: String,
    pub quality: Option<Quality>,
    pub episode: i32,
    pub version: i32,
    pub extension: Option<String>
}
