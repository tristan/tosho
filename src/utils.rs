use regex::Regex;
use crate::models::Episode;

pub fn match_title_re(title: &str) -> Option<Episode> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"^\[(?P<group>[^\]]+)\]\s(?P<name>.+?)\s-\s(?P<episode>\d+)(\s?v(?P<version>\d+))?\s(?:\[(?P<quality>[^\]]+)\])?.*?(?:\.(?P<ext>mkv|mp4|avi))?$"
        ).unwrap();
    }
    let caps = RE.captures(&title)?;
    Episode::from(&caps)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::models::Quality;

    #[test]
    fn test_matching() {

        let ep = match_title_re("[Judas] Dorohedoro - 06 [1080p][HEVC x265 10bit][Eng-Subs].mkv")
            .expect("failed to match 1st example");

        assert_eq!(ep.name, "Dorohedoro");
        assert_eq!(ep.group, "Judas");
        assert_eq!(ep.episode, 6);
        assert_eq!(ep.quality, Some(Quality::HD_1080p));
        assert_eq!(ep.version, 1);

        let ep = match_title_re("[Judas] Dorohedoro - 08 v2 [1080p][HEVC x265 10bit][Eng-Subs].mkv")
            .expect("failed to match 2nd example");

        assert_eq!(ep.name, "Dorohedoro");
        assert_eq!(ep.group, "Judas");
        assert_eq!(ep.episode, 8);
        assert_eq!(ep.quality, Some(Quality::HD_1080p));
        assert_eq!(ep.version, 2);

        let ep = match_title_re("[Judas] Dorohedoro - 09 [1080p][HEVC x265 10bit][Eng-Subs]")
            .expect("failed to match 3rd example");

        assert_eq!(ep.name, "Dorohedoro");
        assert_eq!(ep.group, "Judas");
        assert_eq!(ep.episode, 9);
        assert_eq!(ep.quality, Some(Quality::HD_1080p));
        assert_eq!(ep.version, 1);

        let ep = match_title_re("[Die Hot 14 - My Hot Will Go On (Director's Cut)] Great Pretender - 14v2 [57023320].mkv")
            .expect("Failed to match 4th example");

        assert_eq!(ep.name, "Great Pretender");
        assert_eq!(ep.group, "Die Hot 14 - My Hot Will Go On (Director's Cut)");
        assert_eq!(ep.episode, 14);
        assert_eq!(ep.quality, None);
        assert_eq!(ep.version, 2);
    }
}
