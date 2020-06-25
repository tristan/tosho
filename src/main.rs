#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate postgres;

use clap::{Arg, App, SubCommand};

use std::env;

mod curl;
mod utils;
mod rss;
mod database;
mod models;
mod commands;
mod tosho;
mod sabnzbd;

fn main() {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|e| {
        panic!("Missing DATABASE_URL: {}", e)
    });
    let db = database::connect(&database_url);

    let sabnzbd_url = env::var("SABNZBD_URL").unwrap_or_else(|e| {
        panic!("Missing SABNZBD_URL: {}", e)
    });
    let sabnzbd_apikey = env::var("SABNZBD_APIKEY").unwrap_or_else(|e| {
        panic!("Missing SABNZBD_APIKEY: {}", e)
    });
    let sabnzbd = sabnzbd::SabnzbdClient::new(&sabnzbd_url, &sabnzbd_apikey);

    let app = App::new("Tosho")
        .subcommand(SubCommand::with_name("add")
                    .arg(Arg::with_name("group")
                         .help("The group name")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("name")
                         .help("The show name")
                         .required(true)
                         .multiple(true))
                    .arg(Arg::with_name("quality")
                         .help("The show quality")
                         .short("q")
                         .long("quality")
                         .possible_values(&["480p", "480", "LOW", "low", "Low", "LQ", "lq", "Lq",
                                            "720p", "720", "MID", "mid", "Mid",
                                            "1080p", "1080", "HD", "hd", "Hd"]))
                    .arg(Arg::with_name("start")
                         .help("The episode number to start from")
                         .short("s")
                         .long("start")
                         .default_value("1")))
        .subcommand(SubCommand::with_name("queue"))
        .subcommand(SubCommand::with_name("check"))
        .subcommand(SubCommand::with_name("recheck")
                    .arg(Arg::with_name("page")
                         .help("The rss page to recheck")
                         .required(true)
                         .index(1)));
    let matches = app.get_matches();

    match matches.subcommand() {
        ("add", Some(add_matches)) => {
            let group = add_matches.value_of("group").unwrap();
            let group = if group.starts_with("[") {
                &group[1..group.len()-1]
            } else {
                group
            };
            let name = add_matches.values_of("name").unwrap()
                .collect::<Vec<&str>>()
                .join(" ");
            let start = value_t!(add_matches.value_of("start"), i32)
                .unwrap_or_else(|e| e.exit());
            let quality = add_matches.value_of("quality")
                .map(|q| q.parse::<models::Quality>().unwrap());
            commands::add(&db, &group, &name, start, &quality)
                .unwrap_or_else(|e| e.exit());
        },
        ("check", Some(_check_matches)) => {
            commands::check_missing(&db)
                .unwrap_or_else(|e| e.exit());
            commands::check(&db)
                .unwrap_or_else(|e| e.exit());
        },
        ("recheck", Some(recheck_matches)) => {
            let page = value_t!(recheck_matches.value_of("page"), u8)
                .unwrap_or_else(|e| e.exit());
            commands::recheck(&db, page)
                .unwrap_or_else(|e| e.exit());
        },
        ("queue", Some(_queue_matches)) => {
            commands::queue(&db, &sabnzbd)
                .unwrap_or_else(|e| e.exit());
        },
        _ => {
            println!("{}", matches.usage());
        }
    }
}
