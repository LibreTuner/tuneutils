#[macro_use]
extern crate conrod;
extern crate find_folder;

extern crate tuneutils;
extern crate rustyline;
extern crate clap;
extern crate directories;

use conrod::backend::glium::glium::{self, Surface};

use tuneutils::protocols::can;
use tuneutils::protocols::isotp;
use tuneutils::protocols::uds::{UdsInterface, UdsIsotp};
use tuneutils::download;
use tuneutils::download::{DownloadCallback, Downloader};
use tuneutils::definition;
use tuneutils::link;
use tuneutils::definition::Definitions;
use tuneutils::error::{Result};
use tuneutils::rom;
use tuneutils::datalog;

use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::cell::RefCell;
use std::sync::{Mutex, Arc};
use std::thread;

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
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub avail_links: RefCell<Vec<Box<link::DataLinkEntry>>>,
    pub definitions: Definitions,
    pub roms: RefCell<rom::RomManager>,
    pub tunes: rom::tune::TuneManager,
}

struct LogEntry {
    id: u32,
    name: String,
    data: f64,
    unit: String,
}

impl TuneUtils {
    fn new() -> TuneUtils {
        let proj_dirs = ProjectDirs::from("org", "LibreTuner",  "TuneUtils").unwrap();
        let config_dir = proj_dirs.config_dir().to_path_buf();
        let data_dir = proj_dirs.data_dir().to_path_buf();
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&data_dir).unwrap();

        let mut definitions = Definitions::default();
        definitions.load(&config_dir.join("definitions")).unwrap();

        let rom_dir = data_dir.join("roms");
        fs::create_dir_all(&rom_dir).unwrap();
        let mut roms = rom::RomManager::new(rom_dir);
        roms.load(&definitions).unwrap();

        let tune_dir = data_dir.join("tunes");
        fs::create_dir_all(&tune_dir).unwrap();
        let tunes = rom::tune::TuneManager::load(tune_dir).unwrap();


        TuneUtils {
            config_dir,
            data_dir,
            avail_links: RefCell::new(link::discover_datalinks()),
            definitions,
            roms: RefCell::new(roms),
            tunes,
        }
    }

    fn reload_definitions(&mut self) -> Result<()> {
        self.definitions.load(&self.config_dir.join("definitions"))
    }

    fn run(&mut self) {
        let tu = RefCell::new(self);

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
                    let s = tu.borrow_mut();
                    s.avail_links.borrow_mut().push(Box::new(link::SocketCanDataLinkEntry { interface: args[1].to_string(), }));
                },
                _ => println!("Unsupported link type"),
            }
        }), "Add Link"));



        commands.register("links", Command::new(Box::new(|args| {
            let s = tu.borrow_mut();
            println!("Id\tType\t\tDescription\t\t\t\tLoaded");
            for (i, link) in s.avail_links.borrow().iter().enumerate() {
                println!("{}    {}    {}    No", i, link.typename(), link.description());
            }
        }), "Lists available links"));



        commands.register("platforms", Command::new(Box::new(|args| {
            let s = tu.borrow_mut();
            println!("Id\t\tName");
            for definition in s.definitions.definitions.iter() {
                println!("{}\t{}", definition.id, definition.name);
            }
        }), "Lists installed platform definitions"));



        commands.register("download", Command::new(Box::new(|args| {
            let mut s = tu.borrow_mut();
            if args.len() < 2 {
                println!("Usage: download <datalink id> <platform id>");
                return;
            }
            let datalink_id = match args[0].parse::<usize>() {
                Ok(id) => id,
                Err(err) => { println!("invalid datalink id"); return; },
            };
            let platform_id = args[1];

            // Find the datalink
            if datalink_id >= s.avail_links.borrow().len() {
                println!("Datalink id out of scope");
                return;
            }

            let datalink = match s.avail_links.borrow()[datalink_id].create() {
                Ok(link) => link,
                Err(err) => { println!("Failed to load datalink: {}", err); return; },
            };

            // Find the platform
            let platform = match s.definitions.find(platform_id) {
                Some(def) => def,
                None => { println!("Invalid platform id"); return; },
            };

            // Create the platform link
            let link = link::PlatformLink::new(datalink, platform.clone());
            let downloader = match link.downloader() {
                Some(dl) => dl,
                None => { println!("Downloading is unsupported on this platform or datalink"); return; },
            };

            let mut rl = Editor::<()>::new();

            let name = rl.readline("ROM name: ").unwrap();
            let id = rl.readline("ROM id: ").unwrap();
            // Begin downloading
            let data = match downloader.download(&DownloadCallback::with(|progress| {
                println!("Progress: {:.*}%", 2, progress * 100.0);
            })) {
                Ok(res) => res.data,
                Err(err) => { println!("Failed to download: {}", err); return; },
            };

            let model = match platform.identify(&data) {
                Some(model) => model,
                None => { println!("Failed to identify ROM model"); return; },
            };

            let rom = s.roms.borrow_mut().new_rom(name, id, platform.clone(), model.clone(), data);
            s.roms.borrow_mut().save_meta().unwrap();
            rom.save().unwrap();
        }), "Downloads firmware"));



        commands.register("pids", Command::new(Box::new(|args| {
            let mut s = tu.borrow_mut();
            if args.is_empty() {
                println!("Usage: pids <platform id>");
                return;
            }

            // Find the platform
            let platform = match s.definitions.find(args[0]) {
                Some(def) => def,
                None => { println!("Invalid platform id"); return; },
            };

            println!("Id\tName\tDescription");
            for pid in platform.pids.iter() {
                println!("{}\t{}\t{}", pid.id, pid.name, pid.description);
            }
        }), "Lists the PIDs of a platform"));



        commands.register("roms", Command::new(Box::new(|args| {
            let s = tu.borrow();
            println!("Id\tName\tPlatform\t\t\t\t\tModel");
            for rom in s.roms.borrow().roms.iter() {
                println!("{}\t{}\t{}\t{}", rom.id, rom.name, rom.platform.name, rom.model.name);
            }
        }), "Lists ROMs"));



        commands.register("tunes", Command::new(Box::new(|args| {
            let s = tu.borrow();
            println!("Id\tName\tROM Id");
            for tune in s.tunes.tunes.iter() {
                println!("{}\t{}\t{}", tune.id, tune.name, tune.rom_id);
            }
        }), "Lists tunes"));



        commands.register("create_tune", Command::new(Box::new(|args| {
            let mut s = tu.borrow_mut();

            if args.len() < 3 {
                println!("Usage: create_tune <id> <name> <rom id>");
                return;
            }

            let id = args[0];
            let name = args[1];

            {
                // Just check that the ROM exists
                let roms = s.roms.borrow();
                match roms.search(args[2]) {
                    Some(meta) => meta,
                    None => { println!("Invalid ROM id"); return; },
                };
            }

            // Add to tunes
            s.tunes.add_meta(name.to_string(), id.to_string(), args[2].to_string());
            s.tunes.save().unwrap();

        }), "Creates a new tune from a ROM"));



        commands.register("log", Command::new(Box::new(|args| {
            let mut s = tu.borrow_mut();
            if args.len() < 3 {
                println!("Usage: download <datalink id> <platform id> <pids>");
                return;
            }
            let datalink_id = match args[0].parse::<usize>() {
                Ok(id) => id,
                Err(err) => { println!("invalid datalink id"); return; },
            };
            let platform_id = args[1];
            let pids: Vec<&str> = args[2].split(",").collect();

            // Find the datalink
            if datalink_id >= s.avail_links.borrow().len() {
                println!("Datalink id out of scope");
                return;
            }

            let datalink = match s.avail_links.borrow()[datalink_id].create() {
                Ok(link) => link,
                Err(err) => { println!("Failed to load datalink: {}", err); return; },
            };

            // Find the platform
            let platform = match s.definitions.find(platform_id) {
                Some(def) => def,
                None => { println!("Invalid platform id"); return; },
            };

            let link = link::PlatformLink::new(datalink, platform.clone());
            let mut logger = match link.datalogger() {
                Some(dl) => dl,
                None => { println!("Datalogging is unsupported on this platform or datalink"); return; },
            };

            let entries = Arc::new(Mutex::new(Vec::new()));

            let mut log = datalog::Log::new(platform.clone());
            // Add all PIDs for now
            for pid in platform.pids.iter() {
                //if pid.id  == 3 || pid.id == 5 || pid.id == 0 {
                if pids.iter().find(|p| pid.id.to_string() == **p) != None {
                    log.add_entry(pid);
                    logger.add_entry(pid);
                    entries.lock().unwrap().push(LogEntry {
                        id: pid.id,
                        name: pid.name.to_owned(),
                        data: 0.0,
                        unit: pid.unit.to_owned(),
                    })
                }
            }

            {
                let entries = entries.clone();
                log.register(move |entry, num| {
                    if let Some(e) = entries.lock().unwrap().iter_mut().find(|ref x| x.id == entry.pid_id) {
                        e.data = f64::from(num);
                    }
                });
            }

            let handle = thread::spawn(move || {
                run_ui(entries);
            });

            if let Err(err) = logger.run(&mut log) {
                println!("Datalogger failed: {}", err);
            }
            handle.join().unwrap();
        }), "Datalog"));


        commands.register("scan", Command::new(Box::new(|args| {
            let mut s = tu.borrow_mut();
            if args.len() < 2 {
                println!("Usage: scan <datalink id> <platform id>");
                return;
            }
            let datalink_id = match args[0].parse::<usize>() {
                Ok(id) => id,
                Err(err) => { println!("invalid datalink id"); return; },
            };
            let platform_id = args[1];

            // Find the datalink
            if datalink_id >= s.avail_links.borrow().len() {
                println!("Datalink id out of scope");
                return;
            }

            let datalink = match s.avail_links.borrow()[datalink_id].create() {
                Ok(link) => link,
                Err(err) => { println!("Failed to load datalink: {}", err); return; },
            };

            // Find the platform
            let platform = match s.definitions.find(platform_id) {
                Some(def) => def,
                None => { println!("Invalid platform id"); return; },
            };

            // Create the platform link
            let link = link::PlatformLink::new(datalink, platform.clone());
            

            let codes = link.uds().unwrap().request(3, &[]).unwrap();
            println!("{:?}", codes);
            let c = match (codes[0] & 0xC0) >> 6 {
                0 => 'P',
                1 => 'C',
                0x10 => 'B',
                0x11 => 'U',
                _ => '?',
            };

            println!("Code: {}", c);

        }), "Scans OBD-II codes"));


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
}

/// This `Iterator`-like type simplifies some of the boilerplate involved in setting up a
/// glutin+glium event loop that works efficiently with conrod.
pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl EventLoop {

    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(&mut self, events_loop: &mut glium::glutin::EventsLoop) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events_loop.poll_events(|event| events.push(event));

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        /*if events.is_empty() && !self.ui_needs_update {
            events_loop.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            });
        }*/

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }

}

widget_ids!(struct Ids
    {
        text,
        canvas,
        list,
    });

fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids, entries: &Vec<LogEntry>) {
    use conrod::{widget, Positionable, Colorable, Widget, Sizeable};

    const MARGIN: conrod::Scalar = 30.0;
    const SHAPE_GAP: conrod::Scalar = 50.0;
    const TITLE_SIZE: conrod::FontSize = 42;
    const SUBTITLE_SIZE: conrod::FontSize = 32;

    // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
    // By default, its size is the size of the window. We'll use this as a background for the
    // following widgets, as well as a scrollable container for the children widgets.
    widget::Canvas::new().color(conrod::color::DARK_CHARCOAL).set(ids.canvas, ui);

    let (mut items, scrollbar) = widget::List::flow_down(entries.len())
            .item_size(20.0)
            .scrollbar_on_top()
            .middle_of(ids.canvas)
            .wh_of(ids.canvas)
    .set(ids.list, ui);

    while let Some(item) = items.next(ui) {
        let i = item.i;
        let label = format!("{}: {} {}", entries[i].name, entries[i].data, entries[i].unit);
        let lab = widget::Text::new(&label)
            .color(conrod::color::WHITE)
            .font_size(12);
        item.set(lab, ui);
        /*let toggle = widget::Toggle::new(list[i])
            .label(&label)
            .label_color(conrod::color::WHITE)
            .color(conrod::color::LIGHT_BLUE);
        for v in item.set(toggle, ui) {
            list[i] = v;
        }*/
    }

    if let Some(s) = scrollbar { s.set(ui) }
}

fn run_ui(log: Arc<Mutex<Vec<LogEntry>>>) {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 200;

    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
                    .with_title("Hello Conrod")
                    .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
                    .with_vsync(true)
                    .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    let assets = find_folder::Search::KidsThenParents(3,
    5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Build UI
    let ids = Ids::new(ui.widget_id_generator());

    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
    // Poll events from the window.
    let mut event_loop = EventLoop::new();
    'main: loop {
        // Handle all events.
        for event in event_loop.next(&mut events_loop) {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                ui.handle_event(event);
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested |
                    glium::glutin::WindowEvent::KeyboardInput {
                        input: glium::glutin::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        set_ui(ui.set_widgets(), &ids, &log.lock().unwrap());
        // Render the `Ui` and then display it on the screen.
        let primitives = ui.draw();
        {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.1, 0.1, 0.1, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}

fn main() {
    let mut utils = TuneUtils::new();
    utils.run();
    //run_ui();
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