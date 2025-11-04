use crate::device::{
    EdifierClient, EqualizerPreset, GameMode, LdacMode, NoiseCancellationMode, MAX_AMBIENT_VOLUME,
    MAX_PROMPT_VOLUME,
};
use crate::utils::join_str;
use argh::FromArgs;
use std::env;
use std::io::stdin;
use std::str::FromStr;

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

    #[argh(option, description = "set prompt volume [0-15]", arg_name = "0-15")]
    prompt_volume: Option<u8>,

    #[argh(
        option,
        description = "set ambient sound volume (when NC set to 'ambient') [0-12]",
        arg_name = "0-12"
    )]
    ambient_volume: Option<u8>,

    #[argh(option, description = "set game mode [on|off]", arg_name = "on|off")]
    game: Option<GameMode>,

    #[argh(
        option,
        description = "set LDAC mode [48K|96K|off]",
        arg_name = "48K|96K|off"
    )]
    ldac: Option<LdacMode>,

    #[argh(
        option,
        description = "set noise cancellation mode [on|off|ambient]",
        arg_name = "on|off|ambient"
    )]
    noise_cancel: Option<NoiseCancellationMode>,

    #[argh(
        option,
        description = "set equalizer preset [default|pop|classical|rock]",
        arg_name = "default|pop|classical|rock"
    )]
    equalizer: Option<EqualizerPreset>,

    #[argh(
        option,
        description = "set device button noise cancellation control set [[on]|[off]|[ambient]]",
        arg_name = "[on]|[off]|[ambient]"
    )]
    button: Option<String>,

    #[argh(switch, description = "disconnect device")]
    disconnect: bool,

    #[argh(switch, description = "power off device")]
    power_off: bool,

    #[argh(switch, description = "re-pair device")]
    re_pair: bool,

    #[argh(switch, description = "reset device to factory defaults")]
    reset: bool,

    #[argh(
        switch,
        short = 'y',
        description = "disable confirmation for unsafe operations"
    )]
    no_confirm: bool,
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

    if let Some(option) = args.name {
        client.set_device_name(option.as_str()).unwrap();
        println!("Device name set to: {}.", option);
    }

    if let Some(option) = args.prompt_volume {
        if option > MAX_PROMPT_VOLUME {
            println!("Prompt volume must be from 0 to {}.", MAX_PROMPT_VOLUME);
        } else {
            client.set_prompt_volume(option).unwrap();
            println!("Prompt volume set to: {}.", option);
        }
    }

    if let Some(option) = args.game {
        client.set_game_mode(option).unwrap();
        println!("Game mode set to: {}.", option);
    }

    if let Some(option) = args.ldac {
        if args.no_confirm || confirm_disconnect() {
            client.set_ldac_mode(option).unwrap();
            println!("LDAC mode set to: {}.", option);
        }
    }

    if let Some(option) = args.noise_cancel {
        client.set_noise_mode(option).unwrap();
        println!("Noise cancellation mode set to: {}.", option);
    }

    if let Some(option) = args.ambient_volume {
        if option > MAX_AMBIENT_VOLUME {
            println!("Ambient volume must be from 0 to {}.", MAX_AMBIENT_VOLUME);
        } else {
            client.set_ambient_volume(option).unwrap();
            println!("Ambient volume set to: {}.", option);
        }
    }

    if let Some(option) = args.equalizer {
        client.set_equalizer_preset(option).unwrap();
        println!("Equalizer set to: {}.", option);
    }

    if let Some(option) = args.button {
        match option
            .split('-')
            .map(|s| NoiseCancellationMode::from_str(s))
            .collect()
        {
            Ok(set) => {
                client.set_button_control_set(&set).unwrap();
                println!("Button actions set to: [{}].", join_str(set, ", "));
            }
            Err(_) => {
                println!("Invalid control set: '{option}'");
            }
        }
    }

    if args.disconnect {
        if args.no_confirm || confirm_disconnect() {
            client.disconnect_bluetooth().unwrap();
            println!("Device disconnected.");
        }
    }

    if args.re_pair {
        if args.no_confirm || confirm_disconnect() {
            client.re_pair().unwrap();
            println!("Re-pairing device.");
        }
    }

    if args.power_off {
        if args.no_confirm || confirm_disconnect() {
            client.power_off().unwrap();
            println!("Device powered off.");
        }
    }

    if args.reset {
        if args.no_confirm || confirm_disconnect() {
            client.reset_factory_defaults().unwrap();
            println!("Device settings reset to factory defaults.");
        }
    }

    if args.info {
        print_info(client).unwrap()
    }
}

fn confirm_disconnect() -> bool {
    println!("The device will be disconnected. Continue? (y/n)");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).unwrap();
    buffer.to_ascii_lowercase().trim() == "y"
}

fn print_info(client: EdifierClient) -> Result<(), String> {
    println!("Device name: {}", client.get_device_name()?);
    println!("Mac address: {}", client.get_mac_address()?);
    println!("Battery level: {}%", client.get_battery_level()?);
    println!("Firmware version: {}", client.get_firmware_version()?);
    println!("Fingerprint: {}", client.get_fingerprint()?);
    println!(
        "Prompt volume: {} of {}",
        client.get_prompt_volume()?,
        MAX_PROMPT_VOLUME
    );
    println!("Game mode: {}", client.get_game_mode()?);
    println!("LDAC mode: {}", client.get_ldac_mode()?);
    println!("Noise cancellation mode: {}", client.get_noise_mode()?);
    println!(
        "Ambient volume: {} of {}",
        client.get_ambient_volume()?,
        MAX_AMBIENT_VOLUME
    );
    println!("Equalizer preset: {}", client.get_equalizer_preset()?);
    println!(
        "Button actions: [{}]",
        join_str(client.get_button_control_set()?, ", ")
    );

    Ok(())
}
