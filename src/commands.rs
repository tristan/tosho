use std::process;

use wildmatch::WildMatch;

use crate::models::Quality;
use crate::database::{Database, Error as DatabaseError};
use crate::sabnzbd::{SabnzbdClient, Error as SabnzbdError};
use crate::tosho;
use crate::utils;

#[derive(Debug)]
pub enum Error {
    ToshoError(tosho::Error),
    DatabaseError(DatabaseError),
    SabnzbdError(SabnzbdError),
}

impl Error {
    pub fn exit(&self) -> ! {
        eprintln!("{:?}", self);
        process::exit(1);
    }
}

impl From<tosho::Error> for Error {
    fn from(err: tosho::Error) -> Error {
        Error::ToshoError(err)
    }
}

impl From<DatabaseError> for Error {
    fn from(err: DatabaseError) -> Error {
        Error::DatabaseError(err)
    }
}

impl From<SabnzbdError> for Error {
    fn from(err: SabnzbdError) -> Error {
        Error::SabnzbdError(err)
    }
}

pub fn add(db: &Database, group: &str, name: &str, start: i32, quality: &Option<Quality>) -> Result<(), Error> {
    println!("[{}] {} - {} [{:?}]", group, name, start, quality);
    let items = tosho::search(&[
        group, " ", name, " ",
        &match quality {
            Some(q) => q.to_string(),
            None => "".to_string()
        }
    ].join(" "))?;
    let mut filtered: Vec<(i32, i32, String, bool)> = Vec::new();
    for item in items {
        if let Some(ep) = utils::match_title(&item.title) {
            if group.contains("*") {
                if !WildMatch::new(&group).is_match(&ep.group) {
                    continue;
                }
            } else if ep.group != group {
                continue;
            }
            if ep.name != name || &ep.quality != quality {
                continue;
            }
            println!("{} {} {} v{} {:?} {:?}", ep.group, ep.name, ep.episode, ep.version, ep.quality, ep.extension);
            // TODO: what am i actually doing with the * here?
            filtered.push((ep.episode, ep.version,
                           item.nzb_link.to_string(),
                           ep.episode < start));
        }
    }
    db.add_show_and_episodes(&group, &name, &quality, &filtered)?;
    Ok(())
}

pub fn check(db: &Database) -> Result<(), Error> {
    let last_pub_date = db.get_last_pub_date();
    let mut newest_pub_date = last_pub_date.clone();
    let mut page = 1;
    let mut new_episodes: Vec<(i64, i32, i32, String)> = Vec::new();
    'outer: loop {
        println!("getting feed page: {}", page);
        let items = tosho::feed(&page)?;
        if items.len() == 0 {
            break 'outer;
        }
        for item in items {
            if item.pub_date < last_pub_date {
                break 'outer;
            }
            if item.pub_date > newest_pub_date {
                newest_pub_date = item.pub_date;
            }
            if let Some(ep) = utils::match_title(&item.title) {
                if let Some(show_id) = db.get_show_id(&ep.group, &ep.name, &ep.quality)? {
                    print!("Found [{}] {} - {} v{} [{}]",
                           ep.group, ep.name, ep.episode, ep.version,
                           ep.quality.map(|q| q.to_string())
                           .unwrap_or_else(|| "".to_string()));
                    if item.nzb_link == "" {
                        println!(" -- MISSING NZB LINK")
                    } else {
                        println!("");
                    }
                    new_episodes.push((
                        show_id, ep.episode, ep.version, item.nzb_link.to_string()));
                }
            }
        }
        page += 1;
    }
    if ! new_episodes.is_empty() {
        db.add_episodes(&new_episodes)?;
    }
    db.set_last_pub_date(&newest_pub_date)?;
    Ok(())
}

pub fn recheck(db: &Database, page: u8) -> Result<(), Error> {
    let mut new_episodes: Vec<(i64, i32, i32, String)> = Vec::new();
    println!("getting feed page: {}", page);
    let items = tosho::feed(&page)?;
    for item in items {
        if let Some(ep) = utils::match_title(&item.title) {
            //dbg!(&ep);
            if let Some(show_id) = db.get_show_id(&ep.group, &ep.name, &ep.quality)? {
                if let None = db.get_episode(&show_id, &ep.episode, &ep.version)? {
                    print!("Found [{}] {} - {} v{} [{}]",
                           ep.group, ep.name, ep.episode, ep.version,
                           ep.quality.map(|q| q.to_string())
                           .unwrap_or_else(|| "".to_string()));
                    if item.nzb_link == "" {
                        println!(" -- MISSING NZB LINK")
                    } else {
                        println!("");
                    }
                    new_episodes.push((
                        show_id, ep.episode, ep.version, item.nzb_link.to_string()));
                } else {
                    println!("Skipping existing [{}] {} - {} v{} [{}]",
                             ep.group, ep.name, ep.episode, ep.version,
                             ep.quality.map(|q| q.to_string())
                             .unwrap_or_else(|| "".to_string()));
                }
            }
        }
    }
    if ! new_episodes.is_empty() {
        db.add_episodes(&new_episodes)?;
    }
    Ok(())
}

pub fn check_missing(db: &Database) -> Result<(), Error> {
    let missing_episodes = db.list_episodes_missing_nzb()?;
    let mut new_episodes: Vec<(i64, i32, i32, String)> = Vec::new();
    for episode in missing_episodes {
        let show_id = episode.0;
        let name = episode.1;
        let group = episode.2;
        let quality = episode.3;
        let episode_no = episode.4;
        let version = episode.5;
        println!("Checking for: [{}] {} - {} v{} [{}]",
                 group, name, episode_no, version,
                 quality.as_ref().map(|q| q.to_string())
                 .unwrap_or_else(|| "".to_string()));

        let terms = {
            let arr: [&str; 4] = [&group, &name,
                                  &format!("{:02}", episode_no),
                                  &quality.as_ref().map(|q| q.to_string())
                                  .unwrap_or_else(|| "".to_string())];
            arr.join(" ")
        };
        let results = tosho::search(&terms)?;
        for item in results {
            if item.nzb_link == "" {
                continue;
            }
            if let Some(ep) = utils::match_title(&item.title) {
                if group.contains("*") {
                    if !WildMatch::new(&group).is_match(&ep.group) {
                        continue;
                    }
                } else if ep.group != group {
                    continue;
                }
                if ep.name == name && ep.episode == episode_no && ep.quality == quality {
                    println!("Found [{}] {} - {} v{} [{}]",
                             ep.group, ep.name, ep.episode, ep.version,
                             ep.quality.map(|q| q.to_string())
                             .unwrap_or_else(|| "".to_string()));
                    new_episodes.push((
                        show_id, ep.episode, ep.version, item.nzb_link.to_string()));
                }
            }
        }
    }
    if ! new_episodes.is_empty() {
        db.add_episodes(&new_episodes)?;
    }
    Ok(())
}


pub fn queue(db: &Database, sabnzbd: &SabnzbdClient) -> Result<(), Error> {
    for (show_id, ep_no, url) in db.list_ungrapped_nzbs()? {
        if url == "" {
            continue;
        }
        println!("Grabbing: {}", url);
        sabnzbd.addurl(&url, &"anime")?;
        db.mark_grabbed(show_id, ep_no)?;
    }
    Ok(())
}
