use std::str::FromStr;
use crate::models::{Episode, Quality};

fn match_name_ep_version(text: &str) -> Option<(String, i32, i32)> {
    let mut rsn = text.rsplitn(2, " - ").map(str::trim);
    match rsn.next() {
        Some(epv) => {
            let epv = if epv.ends_with("END") {
                &epv[..epv.len() - 3]
            } else {
                epv
            };
            let mut sn = epv.splitn(2, "v").map(str::trim);
            let ep: i32 = match sn.next() {
                Some(ep) => match ep.parse::<i32>() {
                    Ok(ep) => ep,
                    Err(_) => return None
                },
                None => return None
            };
            let v: i32 = match sn.next() {
                Some(v) => match v.parse::<i32>() {
                    Ok(v) => v,
                    Err(_) => return None
                },
                None => 1
            };
            match rsn.next() {
                Some(name) => Some((name.to_string(), ep, v)),
                None => None
            }
        },
        None => None
    }
}

pub fn match_title(title: &str) -> Option<Episode> {
    if !title.starts_with("[") {
        return None;
    }
    if let Some(idx) = title.find("]") {
        let group = title[1..idx].to_string();

        if let Some(qidx) = title[idx+1..].find("[") {
            let qidx = qidx + idx + 1;
            let (name, episode, version) = match match_name_ep_version(&title[idx+1..qidx]) {
                Some((name, episode, version)) => (name, episode, version),
                None => return None
            };
            let quality = match title[qidx+1..].find("]") {
                Some(qeidx) => Quality::from_str(&title[qidx+1..qidx+1+qeidx]).ok(),
                None => return None
            };
            let extension = if let Some(ext_idx) = title[qidx..].rfind(".") {
                Some(title[qidx+ext_idx+1..].to_string())
            } else {
                None
            };
            Some(Episode {
                group, name, episode, version,
                quality, extension
            })
        } else if let Some(ext_idx) = title[idx..].rfind(".") {
            let ext_idx = ext_idx + idx;
            let (name, episode, version) = match match_name_ep_version(&title[idx..ext_idx]) {
                Some((name, episode, version)) => (name, episode, version),
                None => return None
            };
            Some(Episode {
                group, name, episode, version,
                quality: None,
                extension: Some(title[ext_idx+1..].to_string())
            })
        } else {
            let (name, episode, version) = match match_name_ep_version(&title[idx+1..]) {
                Some((name, episode, version)) => (name, episode, version),
                None => return None
            };
            Some(Episode {
                group, name, episode, version,
                quality: None,
                extension: None
            })
        }

    } else {
        None
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::models::Quality;

    #[test]
    fn test_matching() {

        let ep = match_title("[Judas] Dorohedoro - 06 [1080p][HEVC x265 10bit][Eng-Subs].mkv")
            .expect("failed to match 1st example");

        assert_eq!(ep.name, "Dorohedoro");
        assert_eq!(ep.group, "Judas");
        assert_eq!(ep.episode, 6);
        assert_eq!(ep.quality, Some(Quality::HD_1080p));
        assert_eq!(ep.version, 1);

        let ep = match_title("[Judas] Dorohedoro - 08 v2 [1080p][HEVC x265 10bit][Eng-Subs].mkv")
            .expect("failed to match 2nd example");

        assert_eq!(ep.name, "Dorohedoro");
        assert_eq!(ep.group, "Judas");
        assert_eq!(ep.episode, 8);
        assert_eq!(ep.quality, Some(Quality::HD_1080p));
        assert_eq!(ep.version, 2);

        let ep = match_title("[Judas] Dorohedoro - 09 [1080p][HEVC x265 10bit][Eng-Subs]")
            .expect("failed to match 3rd example");

        assert_eq!(ep.name, "Dorohedoro");
        assert_eq!(ep.group, "Judas");
        assert_eq!(ep.episode, 9);
        assert_eq!(ep.quality, Some(Quality::HD_1080p));
        assert_eq!(ep.version, 1);

        let ep = match_title("[Die Hot 14 - My Hot Will Go On (Director's Cut)] Great Pretender - 14v2 [57023320].mkv")
            .expect("Failed to match 4th example");

        assert_eq!(ep.name, "Great Pretender");
        assert_eq!(ep.group, "Die Hot 14 - My Hot Will Go On (Director's Cut)");
        assert_eq!(ep.episode, 14);
        assert_eq!(ep.quality, None);
        assert_eq!(ep.version, 2);

        let ep = match_title("[Erai-raws] Shironeko Project - Zero Chronicle - 12 END [720p].mkv")
            .expect("Failed to match 5th example");

        assert_eq!(ep.name, "Shironeko Project - Zero Chronicle");
        assert_eq!(ep.group, "Erai-raws");
        assert_eq!(ep.episode, 12);
        assert_eq!(ep.quality, Some(Quality::Mid_720p));
        assert_eq!(ep.version, 1);
    }
}
