extern crate tuneutils;
extern crate rustyline;
extern crate clap;

use tuneutils::protocols::can;
use tuneutils::protocols::isotp;
use tuneutils::protocols::uds::{UdsInterface, UdsIsotp};
use tuneutils::download;
use tuneutils::download::Downloader;
use tuneutils::definition;

use std::path::Path;
use std::collections::HashMap;
use std::default::Default;

use can::{CanInterface, CanInterfaceIterator};
use isotp::IsotpInterface;
#[cfg(feature = "socketcan")]
use can::SocketCan;

use rustyline::error::ReadlineError;
use rustyline::Editor;

struct Command {
    callback: Box<FnMut(&[&str])>,
    description: String,
}

impl Command {
    pub fn new<CB: 'static + FnMut(&[&str])>(callback: CB, description: &str) -> Command {
        Command {
            callback: Box::new(callback),
            description: description.to_string(),
        }
    }
}

struct Commands {
    pub commands: HashMap<String, Command>,
}

impl Commands {
    pub fn new() -> Commands {
        Commands {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, command: Command) {
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

fn main() {
    let mut commands = Commands::new();
    commands.register("test", Command::new(|_args| {
        println!("TEST");
    }, "Test command"));

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