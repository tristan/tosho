use crate::models::Quality;
use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Error as RusqliteError, OptionalExtension, Result};
use std::env;
use std::path::PathBuf;
use wildmatch::WildMatch;

#[derive(Debug)]
pub enum Error {
    DbError(RusqliteError),
}

impl From<RusqliteError> for Error {
    fn from(err: RusqliteError) -> Error {
        Error::DbError(err)
    }
}

pub struct Database {
    conn: Connection,
}

const MIGRATE_V1: &str = std::include_str!("../sql/migrate_00001.sql");

pub fn connect() -> Result<Database, Error> {
    let mut home_dir: PathBuf = env::var_os("HOME").map(PathBuf::from).unwrap();
    home_dir.push(".config");
    home_dir.push("tosho");
    home_dir.push("database.sqlite");
    let conn = Connection::open(&home_dir)?;
    for migration in MIGRATE_V1
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        conn.execute(migration, [])?;
    }
    Ok(Database { conn })
}

impl Database {
    pub fn get_last_pub_date(&self) -> Result<NaiveDateTime, Error> {
        self.conn
            .query_row("SELECT last_pub_date FROM tosho", [], |row| row.get(0))
            .map_err(Error::DbError)
    }

    pub fn set_last_pub_date(&self, pub_date: &NaiveDateTime) -> Result<(), Error> {
        self.conn
            .execute(r#"UPDATE tosho SET last_pub_date = $1"#, params![pub_date])?;
        Ok(())
    }

    pub fn add_show_and_episodes(
        &mut self,
        group: &str,
        name: &str,
        quality: &Option<Quality>,
        episodes: &Vec<(i32, i32, String, bool)>,
    ) -> Result<(), Error> {
        let trans = self.conn.transaction()?;
        let show_id: i64 = trans.query_row(r#"SELECT MAX(show_id) + 1 FROM shows"#, [], |row| {
            row.get(0)
        })?;
        trans.execute(
            r#"INSERT INTO shows
               (show_id, "group", name, quality)
               VALUES ($1, $2, $3, $4)"#,
            params![&show_id, &group, &name, &quality],
        )?;
        for ep in episodes {
            trans.execute(
                r#"INSERT INTO episodes
                             (show_id, episode, version, link, grabbed)
                             VALUES ($1, $2, $3, $4, $5)
                             ON CONFLICT (show_id, episode)
                             DO NOTHING"#,
                params![&show_id, &ep.0, &ep.1, &ep.2, &ep.3],
            )?;
        }
        trans.commit()?;
        Ok(())
    }

    pub fn add_episodes(&mut self, episodes: &Vec<(i64, i32, i32, String)>) -> Result<(), Error> {
        let trans = self.conn.transaction()?;
        for (show_id, ep, version, link) in episodes {
            trans.execute(
                r#"INSERT INTO episodes
                             (show_id, episode, version, link, grabbed)
                             VALUES ($1, $2, $3, $4, FALSE)
                             ON CONFLICT (show_id, episode)
                             DO UPDATE
                             SET link = EXCLUDED.link, version = EXCLUDED.version"#,
                params![&show_id, &ep, &version, &link],
            )?;
        }
        trans.commit()?;
        Ok(())
    }

    pub fn get_show_id(
        &self,
        group: &str,
        name: &str,
        quality: &Option<Quality>,
    ) -> Result<Option<i64>, Error> {
        let mut stmt = self.conn.prepare(
            r#"SELECT show_id, "group", quality FROM shows WHERE
               LOWER(name) = LOWER($1)"#,
        )?;
        let show_iter = stmt.query_map(params![&name], |row| {
            let show_id: i64 = row.get(0)?;
            let db_group: String = row.get(1)?;
            let db_quality: Option<Quality> = row.get(2)?;

            Ok((show_id, db_group, db_quality))
        })?;

        for row in show_iter {
            let (show_id, db_group, db_quality) = row?;
            if db_group.contains('*') {
                if !WildMatch::new(&db_group).is_match(group) {
                    continue;
                }
            } else if db_group != group {
                continue;
            }

            if &db_quality == quality {
                return Ok(Some(show_id));
            }
        }
        Ok(None)
    }

    pub fn get_episode(
        &self,
        show_id: &i64,
        episode: &i32,
        version: &i32,
    ) -> Result<Option<(i64, String, String, Option<Quality>, i32, i32)>, Error> {
        self.conn
            .query_row(
                r#"SELECT e.show_id, s.name, s."group", s.quality, e.episode, e.version
               FROM episodes e
               JOIN shows s
               ON e.show_id = s.show_id
               WHERE s.show_id = $1 AND e.episode = $2 AND e.version = $3"#,
                params![show_id, episode, version],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )
            .optional()
            .map_err(Error::DbError)
    }

    pub fn list_episodes_missing_nzb(
        &self,
    ) -> Result<Vec<(i64, String, String, Option<Quality>, i32, i32)>, Error> {
        let mut stmt = self.conn.prepare(
            r#"SELECT e.show_id, s.name, s."group", s.quality, e.episode, e.version
               FROM episodes e
               JOIN shows s
               ON e.show_id = s.show_id
               WHERE e.link = ''"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;
        rows.collect::<Result<_, RusqliteError>>()
            .map_err(Error::DbError)
    }

    pub fn list_ungrapped_nzbs(&self) -> Result<Vec<(i64, i32, String)>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT show_id, episode, link FROM episodes WHERE grabbed IS FALSE")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        rows.collect::<Result<_, RusqliteError>>()
            .map_err(Error::DbError)
    }

    pub fn mark_grabbed(&self, show_id: i64, episode: i32) -> Result<(), Error> {
        self.conn.execute(
            r#"UPDATE episodes SET
               grabbed = TRUE,
               grabbed_on = (datetime('now'))
               WHERE show_id = $1 AND episode = $2"#,
            params![&show_id, &episode],
        )?;
        Ok(())
    }
}
