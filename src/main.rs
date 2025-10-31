use crate::device::EdifierClient;
use argh::FromArgs;
use std::io::Write;
use std::{env, io};

mod bluetooth;
mod device;
mod message;
mod utils;

#[derive(FromArgs)]
/// Tool to control Edifier devices
///
struct Args {
    /// print device current status
    #[argh(switch, short = 'i')]
    info: bool,

    /// set device name
    #[argh(option)]
    set_name: Option<String>,

    /// power off device
    #[argh(switch, short = 'p')]
    power_off: bool,
}

fn main() {
    print_discardable!("Connecting...");

    let client = EdifierClient::new().unwrap_or_else(|e| {
        print_discard!();
        println!("{}", e);
        std::process::exit(1);
    });

    print_discard!();

    /* no args */
    if env::args().count() <= 1 {
        print_info(client);
        return;
    }

    let args: Args = argh::from_env();
    if let Some(name) = args.set_name {
        client.set_device_name(name.as_str()).unwrap();
        println!("Device name set to: {}", name);
    }

    if args.power_off {
        client.power_off_device().unwrap();
        println!("Device power off");
    }

    if args.info {
        print_info(client)
    }
}

fn print_info(client: EdifierClient) {
    println!("Device name: {}", client.get_device_name().unwrap());
    println!("Mac address: {}", client.get_mac_address().unwrap());
}
