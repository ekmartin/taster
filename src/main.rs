#[macro_use]
extern crate log;
extern crate env_logger;

extern crate afterparty;
extern crate clap;
extern crate git2;
extern crate hyper;

mod taste;
mod repo;

use afterparty::{Delivery, Event, Hub};
use hyper::Server;
use std::path::Path;
use std::sync::Mutex;

#[cfg_attr(rustfmt, rustfmt_skip)]
const TASTER_USAGE: &'static str = "\
EXAMPLES:
  taste -w /path/to/workdir
  taste -l 0.0.0.0:1234 -w /path/to/workdir";

pub fn main() {
    use clap::{Arg, App};

    env_logger::init().unwrap();

    let args = App::new("taster")
        .version("0.0.1")
        .about("Tastes Soup commits.")
        .arg(Arg::with_name("listen_addr")
            .short("l")
            .long("listen_addr")
            .takes_value(true)
            .value_name("IP:PORT")
            .default_value("0.0.0.0:4567")
            .help("Listen address and port for webhook delivery"))
        .arg(Arg::with_name("github_repo")
            .short("r")
            .long("github_repo")
            .takes_value(true)
            .required(true)
            .value_name("GH_REPO")
            .default_value("https://github.com/ms705/taster")
            .help("GitHub repository to taste"))
        .arg(Arg::with_name("workdir")
            .short("w")
            .long("workdir")
            .takes_value(true)
            .required(true)
            .value_name("REPO_DIR")
            .help("Directory holding the workspace repo"))
        .after_help(TASTER_USAGE)
        .get_matches();

    let addr = args.value_of("listen_addr").unwrap();
    let repo = args.value_of("github_repo").unwrap();
    let workdir = Path::new(args.value_of("workdir").unwrap());

    let ws = Mutex::new(repo::Workspace::new(repo, workdir));

    let mut hub = Hub::new();
    hub.handle("push", move |delivery: &Delivery| {
        match delivery.payload {
            Event::Push { ref commits, ref pusher, .. } => {
                println!("Handling {} commits pushed by {}",
                         commits.len(),
                         pusher.name);
                for c in commits.iter() {
                    taste::taste_commit(&ws, &c.id);
                }
            }
            _ => (),
        }
    });

    let srvc = Server::http(&addr[..])
        .unwrap()
        .handle(hub);

    println!("Taster listening on {}", addr);
    srvc.unwrap();
}
