use clap::Parser;
use led::magic_home::{Actions, MagicHomeAPI};
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
        println!("{s}");
    }

    println!("Performed action: {:?}", action);
}
