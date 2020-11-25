use std::process;
use led::MagicHomeAPI;
use clap::{
    Arg,
    crate_description,
    crate_authors,
    crate_version,
    crate_name,
    app_from_crate,
};

const ACTIONS: &str = "Possible actions:
on, off, chaos, rainbow, ambient,
red, green, blue, yellow, orange,
lime, purple, pink, cyan, white";

fn main() {
    let matches = app_from_crate!()
                        .arg(Arg::with_name("address")
                            .short("a")
                            .long("address")
                            .value_name("ADDRESS")
                            .help("Address of controller")
                            .takes_value(true)
                            .required(true))
                        .arg(Arg::with_name("ACTION")
                            .help(ACTIONS)
                            .required(true)
                            .index(1))
                        .arg(Arg::with_name("port")
                            .short("p")
                            .long("port")
                            .value_name("PORT")
                            .help("Port to access on controller (default: 5577)")
                            .takes_value(true))
                        .get_matches();

    let action = matches.value_of("ACTION").unwrap();

    let address = matches.value_of("address").unwrap();

    let port = matches.value_of("port");

    let mut magic_api = MagicHomeAPI::new(address, port).unwrap_or_else(|err| {
        eprintln!("Failed Creating api: {}", err);
        process::exit(1);
    });
    println!("Connection successful");

    match action {
        "on" => magic_api.turn_on(),
        "off" => magic_api.turn_off(),
        _ => if let Err(err) = magic_api.set_mode(action) {
                eprintln!("Failed changing mode: {}", err);
                process::exit(1);
            }
    }

    println!("Changed LEDs: {}", action);
}

