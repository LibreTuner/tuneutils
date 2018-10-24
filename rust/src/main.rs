extern crate tuneutils;
extern crate rustyline;
extern crate clap;
extern crate directories;

use tuneutils::protocols::can;
use tuneutils::protocols::isotp;
use tuneutils::protocols::uds::{UdsInterface, UdsIsotp};
use tuneutils::download;
use tuneutils::download::Downloader;
use tuneutils::definition;
use tuneutils::link;

use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::cell::RefCell;

use directories::{BaseDirs, UserDirs, ProjectDirs};

use can::{CanInterface, CanInterfaceIterator};
use isotp::IsotpInterface;
#[cfg(feature = "socketcan")]
use can::SocketCan;

use rustyline::error::ReadlineError;
use rustyline::Editor;

struct Command<'a> {
    callback: Box<FnMut(&[&str]) + 'a>,
    description: String,
}

impl<'a> Command<'a> {
    pub fn new(callback: Box<FnMut(&[&str]) + 'a>, description: &str) -> Command<'a> {
        Command {
            callback: callback,
            description: description.to_string(),
        }
    }
}

struct Commands<'a> {
    pub commands: HashMap<String, Command<'a>>,
}

impl<'a> Commands<'a> {
    pub fn new() -> Commands<'a> {
        Commands {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, command: Command<'a>) {
        self.commands.insert(name.to_string(), command);
    }

    pub fn call(&mut self, command: &str, args: &[&str]) {
        if command == "help" {
            println!("help - Displays this message");
            for (name, cmd) in self.commands.iter() {
                println!("{} - {}", name, cmd.description);
            }
        }
        else if let Some(cmd) = self.commands.get_mut(command) {
            (cmd.callback)(args);
        } else {
            println!("No such command: {}", command);
        }
    }
}

struct TuneUtils {
    config_dir: PathBuf,

    avail_links: RefCell<Vec<Box<link::DataLinkEntry>>>,
}

impl TuneUtils {
    fn new() -> TuneUtils {
        TuneUtils {
            config_dir: PathBuf::new(),
            avail_links: RefCell::new(link::discover_datalinks()),
        }
    }

    fn run(&mut self) {
        let mut commands = Commands::new();
        commands.register("add_link", Command::new(Box::new(|args| {
            if args.is_empty() {
                println!("Usage: add_link <type> [params]");
                return;
            }

            match args[0] {
                #[cfg(feature = "socketcan")]
                "socketcan" => {
                    if args.len() < 2 {
                        println!("Usage: add_link socketcan <interface>");
                        return;
                    }
                    self.avail_links.borrow_mut().push(Box::new(link::SocketCanDataLinkEntry { interface: args[1].to_string(), }));
                },
                _ => println!("Unsupported link type"),
            }
        }), "Add Link"));

        commands.register("links", Command::new(Box::new(|args| {
            println!("Type\t\tDescription\t\t\t\tLoaded");
            for link in self.avail_links.borrow().iter() {
                println!("{}\t{}\tNo", link.typename(), link.description());
            }
        }), "Lists available links"));

        commands.register("definitions", Command::new(Box::new(|args| {
            // Stub
        }), "Lists installed definitions"));

        println!("LibreTuner  Copyright (C) 2018  The LibreTuner Team
This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
under certain conditions; type `show c' for details.");
        let mut rl = Editor::<()>::new();
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_ref());
                    
                    let parts: Vec<&str> = line.split(' ').collect();
                    if parts.len() == 0 {
                        continue;
                    }

                    commands.call(parts[0], &parts[1..]);
                },
                Err(ReadlineError::Interrupted) => {
                    println!("Terminated");
                    break
                },
                Err(ReadlineError::Eof) => {
                    // println!("CTRL-D");
                    break
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                    break
                }
            }
        }
    }

    fn setup_dirs(&mut self) {
        if let Some(proj_dirs) = ProjectDirs::from("org", "LibreTuner",  "TuneUtils") {
            self.config_dir = proj_dirs.config_dir().to_path_buf();

            fs::create_dir_all(&self.config_dir).unwrap();
        }
    }


}

fn main() {
    let mut utils = TuneUtils::new();
    utils.setup_dirs();
    utils.run();
}

/*
#[cfg(feature = "j2534")]
extern crate j2534;
#[cfg(feature = "j2534")]
fn main() {
    for listing in j2534::list().unwrap() {
        println!("Loading {}", listing.name);
        let interface = j2534::Interface::new(&listing.path).unwrap();
        let device = interface.open_any().unwrap();
        let can = can::J2534Can::connect(&device, 500000).unwrap();
        can.apply_blank_filter().unwrap();
        let iface = isotp::IsotpCan::new(&can, isotp::Options::default());
        let uds = UdsIsotp::new(&iface);
        
        println!("{:?}", uds.request(0x01, &[0x05]));
    }
}

#[cfg(feature = "socketcan")]
fn main() {
    let mut defs = definition::Definitions::default();
    defs.load(&Path::new("/home/altenius/projects/LibreTuner/src/resources/definitions")).unwrap();
    println!("Definitions: {:?}", defs.definitions);
    /*
    let can = SocketCan::open("slcan0").expect("Failed to find slcan0");
    
    let iface = isotp::IsotpCan::new(&can, isotp::Options::default());
    let uds = UdsIsotp::new(&iface);

    let downloader = download::mazda::Mazda1Downloader::new(&uds, "MazdA", 1024*5);
    println!("Downloading");
    let data = downloader.download().unwrap();

    println!("Got data: {:?}", data.data);*/
}*/