use postgres::{Connection, TlsMode, Error as PostgresError};
use std::fs;
use chrono::NaiveDateTime;
use wildmatch::WildMatch;
use crate::models::Quality;

#[derive(Debug)]
pub enum Error {
    DbError(PostgresError)
}

impl From<PostgresError> for Error {
    fn from(err: PostgresError) -> Error {
        Error::DbError(err)
    }
}

pub struct Database {
    conn: Connection
}

pub fn connect(database_url: &str) -> Database {
    let conn = Connection::connect(database_url, TlsMode::None).unwrap_or_else(|e| {
        panic!("Error connecting to database: {}", e);
    });
    let migrations = fs::read_to_string("sql/migrate_00001.sql").unwrap_or_else(|e| {
        panic!("Unable to read migration file: {}", e);
    });
    for migration in migrations.split(";") {
        conn.execute(&migration, &[]).unwrap_or_else(|e| {
            panic!("Unable to prepare database: {}", e);
        });
    }

    Database {
        conn
    }
}

impl Database {

    pub fn get_last_pub_date(&self) -> NaiveDateTime {
        let q = self.conn.query("SELECT last_pub_date FROM tosho", &[]);
        match q {
            Ok(rows) => {
                if rows.is_empty() {
                    panic!("No inital last_pub_date set");
                }
                let row = rows.get(0);
                row.get(0)
            }
            Err(err) => {
                panic!("Error connecting to database: {}", err);
            }
        }
    }

    pub fn set_last_pub_date(&self, pub_date: &NaiveDateTime) -> Result<(), Error> {
        self.conn.execute(r#"UPDATE tosho SET last_pub_date = $1"#,
                          &[pub_date])?;
        Ok(())
    }


    pub fn add_show_and_episodes(
        &self, group: &str, name: &str, quality: &Option<Quality>, episodes: &Vec<(i32, i32, String, bool)>
    ) -> Result<(), Error> {
        let trans = self.conn.transaction()?;
        let q = trans.query(r#"INSERT INTO shows
                               ("group", name, quality)
                               VALUES ($1, $2, $3)
                               RETURNING show_id"#,
                            &[&group, &name, &quality])?;
        let show_id: i64 = q.get(0).get(0);
        for ep in episodes {
            trans.execute(r#"INSERT INTO episodes
                             (show_id, episode, version, link, grabbed)
                             VALUES ($1, $2, $3, $4, $5)
                             ON CONFLICT (show_id, episode)
                             DO NOTHING"#,
                          &[&show_id, &ep.0, &ep.1, &ep.2, &ep.3])?;
        }
        trans.commit()?;
        Ok(())
    }

    pub fn add_episodes(&self, episodes: &Vec<(i64, i32, i32, String)>) -> Result<(), Error> {
        let trans = self.conn.transaction()?;
        for (show_id, ep, version, link) in episodes {
            trans.execute(r#"INSERT INTO episodes
                             (show_id, episode, version, link, grabbed)
                             VALUES ($1, $2, $3, $4, FALSE)
                             ON CONFLICT (show_id, episode)
                             DO UPDATE
                             SET link = EXCLUDED.link, version = EXCLUDED.version"#,
                          &[&show_id, &ep, &version, &link])?;
        }
        trans.commit()?;
        Ok(())
    }

    pub fn get_show_id(&self, group: &str, name: &str, quality: &Option<Quality>) -> Result<Option<i64>, Error> {
        let rows = self.conn.query(
            r#"SELECT show_id, "group", quality FROM shows WHERE
                      LOWER(name) = LOWER($1)"#,
            &[&name])?;
        if rows.is_empty() {
            Ok(None)
        } else {
            for row in rows.into_iter() {
                let db_group: String = row.get(1);
                if db_group.contains("*") {
                    if !WildMatch::new(&db_group).is_match(group) {
                        continue;
                    }
                } else if &db_group != group {
                    continue;
                }
                let db_quality: Option<Quality> = row.get(2);
                if &db_quality == quality {
                    return Ok(Some(row.get(0)));
                }
            }
            return Ok(None);
        }
    }

    pub fn get_episode(
        &self, show_id: &i64, episode: &i32, version: &i32
    ) -> Result<Option<(i64, String, String, Option<Quality>, i32, i32)>, Error> {
        let rows = self.conn.query(
            r#"SELECT e.show_id, s.name, s.group, s.quality, e.episode, e.version
               FROM episodes e
               JOIN shows s
               ON e.show_id = s.show_id
               WHERE s.show_id = $1 AND e.episode = $2 AND e.version = $3"#,
            &[show_id, episode, version])?;
        if rows.is_empty() {
            Ok(None)
        } else {
            let row = rows.get(0);
            Ok(Some((row.get(0), row.get(1), row.get(2),
                     row.get(3), row.get(4), row.get(5))))
        }
    }

    pub fn list_episodes_missing_nzb(&self) -> Result<Vec<(i64, String, String, Option<Quality>, i32, i32)>, Error> {
        let rows = self.conn.query(
            r#"SELECT e.show_id, s.name, s.group, s.quality, e.episode, e.version
               FROM episodes e
               JOIN shows s
               ON e.show_id = s.show_id
               WHERE e.link = ''"#,
            &[])?;
        let results = rows.iter().map(|row| {
            Ok((row.get(0), row.get(1), row.get(2),
                row.get(3), row.get(4), row.get(5)))
        }).collect::<Result<Vec<_>, Error>>()?;
        Ok(results)
    }

    pub fn list_ungrapped_nzbs(&self) -> Result<Vec<(i64, i32, String)>, Error> {
        let rows = self.conn.query(
            "SELECT show_id, episode, link FROM episodes WHERE grabbed IS FALSE",
            &[])?;
        let results = rows.iter().map(|row| {
            (row.get(0), row.get(1), row.get(2))
        }).collect();
        Ok(results)
    }

    pub fn mark_grabbed(&self, show_id: i64, episode: i32) -> Result<(), Error> {
        self.conn.execute(
            r#"UPDATE episodes SET
               grabbed = TRUE,
               grabbed_on = (NOW() AT TIME ZONE 'UTC')
               WHERE show_id = $1 AND episode = $2"#,
            &[&show_id, &episode])?;
        Ok(())
    }
}
