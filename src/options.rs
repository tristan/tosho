use crate::models::Quality;
use gumdrop::Options;

#[derive(Debug, Options)]
pub struct ToshoOptions {
    #[options(help = "print help message")]
    help: bool,
    #[options(command, required)]
    pub command: Option<Command>,
}

#[derive(Debug, Options)]
pub enum Command {
    #[options(help = "add show to the check list")]
    Add(AddOpts),
    #[options(help = "queue unfetched shows for download")]
    Queue(QueueOpts),
    #[options(help = "check for new shows")]
    Check(CheckOpts),
    #[options(help = "recheck the whole rss page")]
    Recheck(RecheckOpts),
    #[options(help = "check for new bookmarks on dognzb")]
    Dog(CheckOpts),
}

#[derive(Debug, Options)]
pub struct AddOpts {
    #[options(help = "print help message")]
    help: bool,
    #[options(free, help = "The group name", required)]
    pub group: String,
    #[options(free, help = "The show name", required)]
    pub show: String,
    #[options(help = "The show quality")]
    pub quality: Option<Quality>,
    #[options(help = "The episode number to start from")]
    pub start: Option<i32>,
}

#[derive(Debug, Options)]
pub struct RecheckOpts {
    #[options(free, help = "Which rss page to recheck")]
    pub page: Option<u8>,
}

#[derive(Debug, Options)]
pub struct QueueOpts {
    #[options(help = "print help message")]
    help: bool,
}

#[derive(Debug, Options)]
pub struct CheckOpts {
    #[options(help = "print help message")]
    help: bool,
}
