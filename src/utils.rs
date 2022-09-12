use crate::models::{Episode, Quality};
use std::str::FromStr;

fn match_name_ep_version(text: &str) -> Option<(String, i32, i32)> {
    let mut rsn = text.rsplitn(2, " - ").map(str::trim);
    match rsn.next() {
        Some(epv) => {
            let epv = if epv.ends_with(" END") {
                &epv[..epv.len() - 3]
            } else {
                epv
            };
            let mut sn = epv.splitn(2, 'v').map(str::trim);
            let ep: i32 = match sn.next() {
                Some(ep) => match ep.parse::<i32>() {
                    Ok(ep) => ep,
                    Err(_) => return None,
                },
                None => return None,
            };
            let v: i32 = match sn.next() {
                Some(v) => match v.parse::<i32>() {
                    Ok(v) => v,
                    Err(_) => return None,
                },
                None => 1,
            };
            rsn.next().map(|name| (name.to_string(), ep, v))
        }
        None => None,
    }
}

fn match_name_sxxexx(text: &str) -> Option<(String, i32, i32)> {
    let mut rsn = text.trim().rsplitn(2, ' ').map(str::trim);
    match rsn.next() {
        Some(part) if part.len() == 6 => {
            let mut chars = part.chars();
            match (
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
            ) {
                ('S', s1, s2, 'E', e1, e2)
                    if s1.is_numeric() && s2.is_numeric() && e1.is_numeric() && e2.is_numeric() =>
                {
                    let ep = match part[4..].parse() {
                        Ok(ep) => ep,
                        Err(_) => return None,
                    };
                    rsn.next().map(|name| (name.to_string(), ep, 1))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn match_title(title: &str) -> Option<Episode> {
    if !title.starts_with('[') {
        return None;
    }
    if let Some(idx) = title.find(']') {
        let group = title[1..idx].to_string();
        let (sqc, eqc, eqc_char) = if &group == "SubsPlease" || &group == "PAS" {
            ("(", ")", ')')
        } else {
            ("[", "]", ']')
        };

        if let Some(qidx) = title[idx + 1..].find(sqc) {
            let qidx = qidx + idx + 1;
            let (name, episode, version) = match match_name_ep_version(&title[idx + 1..qidx]) {
                Some((name, episode, version)) => (name, episode, version),
                None => match match_name_sxxexx(&title[idx + 1..qidx]) {
                    Some((name, episode, version)) => (name, episode, version),
                    None => return None,
                },
            };
            let (qidx, version) = if version == 1 && &title[qidx + 1..qidx + 2] == "v" {
                match title[qidx + 1..].find(eqc) {
                    Some(veidx) => match title[qidx + 2..qidx + 1 + veidx].parse::<i32>() {
                        Ok(v) => {
                            if let Some(qsidx) = title[qidx + 2..].find(sqc) {
                                (qidx + 2 + qsidx, v)
                            } else {
                                return None;
                            }
                        }
                        Err(_) => return None,
                    },
                    None => return None,
                }
            } else if title.len() < qidx + 5 {
                (qidx, version)
            } else if &title[qidx + 1..qidx + 5] == "VRV]" {
                (qidx + 5, version)
            } else if &title[qidx + 1..qidx + 5] == "WEB " {
                (qidx + 4, version)
            } else {
                (qidx, version)
            };
            let quality = match title[qidx + 1..].find(|c| c == eqc_char || c == ' ') {
                Some(qeidx) => Quality::from_str(&title[qidx + 1..qidx + 1 + qeidx]).ok(),
                None => return None,
            };
            // let extension = if let Some(ext_idx) = title[qidx..].rfind('.') {
            //     Some(title[qidx + ext_idx + 1..].to_string())
            // } else {
            //     None
            // };
            let extension = title[qidx..]
                .rfind('.')
                .map(|ext_idx| title[qidx + ext_idx + 1..].to_string());
            Some(Episode {
                group,
                name,
                episode,
                version,
                quality,
                extension,
            })
        } else if let Some(ext_idx) = title[idx..].rfind('.') {
            let ext_idx = ext_idx + idx;
            let (name, episode, version) = match match_name_ep_version(&title[idx..ext_idx]) {
                Some((name, episode, version)) => (name, episode, version),
                None => return None,
            };
            Some(Episode {
                group,
                name,
                episode,
                version,
                quality: None,
                extension: Some(title[ext_idx + 1..].to_string()),
            })
        } else {
            let (name, episode, version) = match match_name_ep_version(&title[idx + 1..]) {
                Some((name, episode, version)) => (name, episode, version),
                None => return None,
            };
            Some(Episode {
                group,
                name,
                episode,
                version,
                quality: None,
                extension: None,
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

        let ep = match_title("[Erai-raws] Majo no Tabitabi - 02 [v0][720p].mkv")
            .expect("Failed to match 6th example");

        assert_eq!(ep.name, "Majo no Tabitabi");
        assert_eq!(ep.group, "Erai-raws");
        assert_eq!(ep.episode, 2);
        assert_eq!(ep.quality, Some(Quality::Mid_720p));
        assert_eq!(ep.version, 0);

        let ep = match_title("[Erai-raws] Dungeon ni Deai wo Motomeru no wa Machigatteiru Darou ka III - 05v2 [VRV][480p][Multiple Subtitle].mkv")
            .expect("Failed to match 7th example");

        assert_eq!(
            ep.name,
            "Dungeon ni Deai wo Motomeru no wa Machigatteiru Darou ka III"
        );
        assert_eq!(ep.group, "Erai-raws");
        assert_eq!(ep.episode, 5);
        assert_eq!(ep.quality, Some(Quality::Low_480p));
        assert_eq!(ep.version, 2);

        let ep = match_title(
            "[EMBER] Isekai Ojisan S01E01 [1080p] [HEVC WEBRip DDP] (Uncle from Another World)",
        )
        .expect("Failed to match 8th example");

        assert_eq!(ep.name, "Isekai Ojisan");
        assert_eq!(ep.group, "EMBER");
        assert_eq!(ep.episode, 1);
        assert_eq!(ep.quality, Some(Quality::HD_1080p));
        assert_eq!(ep.version, 1);

        let ep = match_title("[SubsPlease] Jujutsu Kaisen - 08 (720p) [E2508E65].mkv")
            .expect("Failed to match 9th example");

        assert_eq!(ep.name, "Jujutsu Kaisen");
        assert_eq!(ep.group, "SubsPlease");
        assert_eq!(ep.episode, 8);
        assert_eq!(ep.quality, Some(Quality::Mid_720p));
        assert_eq!(ep.version, 1);

        let ep = match_title("[PAS] Beastars S2 - 13 (WEB 1080 AAC) [8CF487D4].mkv")
            .expect("Failed to match 10th example");

        assert_eq!(ep.name, "Beastars S2");
        assert_eq!(ep.group, "PAS");
        assert_eq!(ep.episode, 13);
        assert_eq!(ep.quality, Some(Quality::HD_1080p));
        assert_eq!(ep.version, 1);

        // TODO: fix this
        let ep = match_title(
            "[SubsPlease] Dragon Quest - Dai no Daibouken (2020) - 08 (720p) [2CB58E42].mkv",
        )
        .expect("Failed to match 11th example");

        assert_eq!(ep.name, "Dragon Quest - Dai no Daibouken (2020)");
        assert_eq!(ep.group, "SubsPlease");
        assert_eq!(ep.episode, 8);
        assert_eq!(ep.quality, Some(Quality::Mid_720p));
        assert_eq!(ep.version, 1);
    }
}
