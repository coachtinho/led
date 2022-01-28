mod magic_home;

use clap::Parser;
use magic_home::{Actions, MagicHomeAPI, Status};
use std::process;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// Adress of controller
    #[clap(short, long)]
    address: String,

    /// Port to access on the controller (default: 5577)
    #[clap(short, long)]
    port: Option<String>,

    #[clap(subcommand)]
    action: Actions,
}

fn print_status(status: Status) {
    if status.power {
        println!("Power: on");
    } else {
        println!("Power: off");
    }
    println!("Color: {:?}", status.color);
    println!("Mode: {}", status.mode);
    if let Some(speed) = status.speed {
        println!("Speed: {}", speed);
    }
}

fn main() {
    let args = Args::parse();

    let action = args.action;

    let address = args.address;

    let port = args.port.as_deref();

    let mut magic_api = MagicHomeAPI::new(&address, port).unwrap_or_else(|err| {
        eprintln!("Failed Creating api: {}", err);
        process::exit(1);
    });
    println!("Connection successful");

    let status = magic_api.perform_action(&action).unwrap_or_else(|err| {
        eprintln!("Failed performing action: {}", err);
        process::exit(1);
    });

    if let Some(s) = status {
        print_status(s);
    }

    println!("Performed action: {:?}", action);
}
