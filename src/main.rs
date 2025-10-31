use crate::device::{EdifierClient, EqualizerPreset, GameMode, LdacMode, NoiseMode};
use argh::FromArgs;
use std::env;

mod bluetooth;
mod device;
mod message;
mod utils;

#[derive(FromArgs)]
#[argh(description = "Tool to control Edifier devices")]
struct Args {
    #[argh(switch, description = "print device current status")]
    info: bool,

    #[argh(option, description = "set device name")]
    name: Option<String>,

    #[argh(option, description = "set prompt volume [0..15]", arg_name = "0..15")]
    prompt_volume: Option<u8>,

    #[argh(option, description = "set game mode [on|off]", arg_name = "on|off")]
    game: Option<GameMode>,

    #[argh(
        option,
        description = "set LDAC mode [k48|k96|off]",
        arg_name = "k48|k96|off"
    )]
    ldac: Option<LdacMode>,

    #[argh(
        option,
        description = "set noise cancellation mode [on|off|ambient]",
        arg_name = "on|off|ambient"
    )]
    noise_cancel: Option<NoiseMode>,

    #[argh(
        option,
        description = "set equalizer preset [default|pop|classical|rock]",
        arg_name = "default|pop|classical|rock"
    )]
    equalizer: Option<EqualizerPreset>,

    #[argh(switch, description = "disconnect device")]
    disconnect: bool,

    #[argh(switch, description = "power off device")]
    power_off: bool,

    #[argh(switch, description = "re-pair device")]
    re_pair: bool,

    #[argh(switch, description = "reset device to factory defaults")]
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

    if let Some(name) = args.name {
        client.set_device_name(name.as_str()).unwrap();
        println!("Device name set to: {}.", name);
    }

    if let Some(volume) = args.prompt_volume {
        client.set_prompt_volume(volume).unwrap();
        println!("Prompt volume set to: {}.", volume);
    }

    if let Some(mode) = args.game {
        client.set_game_mode(mode).unwrap();
        println!("Game mode set to: {}.", mode);
    }

    if let Some(mode) = args.ldac {
        client.set_ldac_mode(mode).unwrap();
        println!("LDAC mode set to: {}.", mode);
    }

    if let Some(mode) = args.noise_cancel {
        client.set_noise_mode(mode).unwrap();
        println!("Noise cancellation mode set to: {}.", mode);
    }

    if let Some(preset) = args.equalizer {
        client.set_equalizer_preset(preset).unwrap();
        println!("Equalizer set to: {}.", preset);
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
    println!("LDAC mode: {}", client.get_ldac_mode()?);
    println!("Noise cancellation mode: {}", client.get_noise_mode()?);
    println!("Equalizer preset: {}", client.get_equalizer_preset()?);

    Ok(())
}
