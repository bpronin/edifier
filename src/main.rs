use crate::device::{EdifierClient, GameMode, PlaybackStatus};
use argh::FromArgs;
use std::env;
use std::io::Write;

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

    /// set prompt volume
    #[argh(option)]
    set_prompt_volume: Option<u8>,

    /// set game mode
    #[argh(option)]
    set_game_mode: Option<GameMode>,

    /// disconnect device
    #[argh(switch)]
    disconnect: bool,

    /// power off device
    #[argh(switch)]
    power_off: bool,

    /// re-pair device
    #[argh(switch)]
    re_pair: bool,

    /// reset device factory defaults
    #[argh(switch)]
    reset: bool,
}

fn main() {
    // print_discardable!("Connecting...");

    let client = EdifierClient::new().unwrap_or_else(|e| {
        print_discard!();
        println!("{}", e);
        std::process::exit(1);
    });

    // print_discard!();

    /* no args */
    if env::args().count() <= 1 {
        print_info(client).unwrap();
        return;
    }

    let args: Args = argh::from_env();

    if let Some(name) = args.set_name {
        client.set_device_name(name.as_str()).unwrap();
        println!("Device name set to: {}.", name);
    }

    if let Some(volume) = args.set_prompt_volume {
        client.set_prompt_volume(volume).unwrap();
        println!("Prompt volume set to: {}.", volume);
    }

    if let Some(mode) = args.set_game_mode {
        client.set_game_mode(mode).unwrap();
        println!("Game mode set to: {}.", mode);
    }

    if args.disconnect {
        client.disconnect_bluetooth().unwrap();
        println!("Device disconnected.");
    }

    if args.re_pair {
        client.re_pair().unwrap();
        println!("Re-pairing device.");
    }

    if args.power_off {
        client.power_off().unwrap();
        println!("Device powered off.");
    }

    if args.reset {
        client.reset_factory_defaults().unwrap();
        println!("Device settings reset to factory defaults.");
    }

    if args.info {
        print_info(client).unwrap()
    }
}

fn print_info(client: EdifierClient) -> Result<(), String> {
    println!("Device name: {}", client.get_device_name()?);
    println!("Mac address: {}", client.get_mac_address()?);
    println!("Battery level: {}%", client.get_battery_level()?);
    println!("Firmware version: {}", client.get_firmware_version()?);
    println!("Fingerprint: {}", client.get_fingerprint()?);
    println!("Prompt volume: {}", client.get_prompt_volume()?);
    println!("Game mode: {}", client.get_game_mode()?);

    Ok(())
}
