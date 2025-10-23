use clap::{CommandFactory, Parser};
use std::env;
use crate::device::EdifierClient;

mod bluetooth;
mod device;

#[derive(Parser)]
#[command(
    name = "edifier",
    version = "1.0",
    about = "Tool to control Edifier devices"
)]
struct Args {
    #[arg(short, long, num_args = 0..=1, help = "Set device name. Omit value to print current one")]
    name: Option<Option<String>>,

    #[arg(short, long, help = "Print device MAC address")]
    mac: bool,

    #[arg(short, long, help = "Power off device")]
    power_off: bool,
}

fn main() -> Result<(), String> {
    if env::args_os().len() <= 1 {
        Args::command().print_help().unwrap();
        return Ok(());
    }

    let mut client = EdifierClient::default();
    if let Err(e) = client.connect(){
        println!("{}", e);
        return Ok(());
    }

    let args = Args::parse();
    if let Some(name) = args.name {
        match name {
            Some(value) => {
                client.set_device_name(value)?;
                println!("Device name set")
            },
            None => println!("Device name: {}", client.get_device_name()?),
        }
    }

    if args.mac {
        println!("Mac address: {}", client.get_mac_address()?);
    }

    if args.power_off {
        client.power_off_device()?;
        println!("Device power off");
    }

    Ok(())
}
