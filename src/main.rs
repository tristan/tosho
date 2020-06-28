use gumdrop::Options;

mod config;
mod options;
mod curl;
mod utils;
mod rss;
mod database;
mod models;
mod commands;
mod tosho;
mod sabnzbd;

fn main() {
    let config = config::Config::load();

    let mut db = database::connect().unwrap();
    let sabnzbd = sabnzbd::SabnzbdClient::new(
        &config.sabnzbd.url,
        &config.sabnzbd.apikey);

    let opts = options::ToshoOptions::parse_args_default_or_exit();

    match opts.command {
        Some(options::Command::Add(opts)) => {
            let group = if opts.group.starts_with("[") {
                &opts.group[1..opts.group.len()-1]
            } else {
                &opts.group
            };
            let start = opts.start.unwrap_or(1);
            commands::add(&mut db, group, &opts.show, start, &opts.quality)
                .unwrap_or_else(|e| e.exit());
        },
        Some(options::Command::Queue(_)) => {
            commands::queue(&db, &sabnzbd)
                .unwrap_or_else(|e| e.exit());
        },
        Some(options::Command::Check(_)) => {
            commands::check_missing(&mut db)
                .unwrap_or_else(|e| e.exit());
            commands::check(&mut db)
                .unwrap_or_else(|e| e.exit());
        },
        Some(options::Command::Recheck(opts)) => {
            commands::recheck(&mut db, opts.page.unwrap_or(1))
                .unwrap_or_else(|e| e.exit());
        },
        None => {
            unreachable!();
        }
    }
}
