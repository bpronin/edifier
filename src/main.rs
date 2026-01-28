use crate::device::{
    ButtonControlSet, DenoiseMode, EdifierClient, EqualizerPreset, GameMode, LdacMode,
    MAX_PROMPT_VOLUME,
};
use argh::FromArgs;
use std::env;
use std::io::{stdin, stdout, Write};

mod bluetooth;
mod device;
mod message;
mod utils;

#[derive(FromArgs)]
#[argh(description = "Tool to control Edifier devices")]
struct Args {
    #[argh(switch, short = 'i', description = "print device current status")]
    info: bool,

    #[argh(option, short = 'a', description = "set device name")]
    name: Option<String>,

    #[argh(
        option,
        short = 'v',
        description = "set prompt volume [0-15]",
        arg_name = "0-15"
    )]
    prompt_vol: Option<u8>,

    #[argh(
        option,
        short = 'g',
        description = "set game mode [on|off]",
        arg_name = "on|off"
    )]
    game: Option<GameMode>,

    #[argh(
        option,
        short = 'l',
        description = "set LDAC mode [48K|96K|off]",
        arg_name = "48K|96K|off"
    )]
    ldac: Option<LdacMode>,

    #[argh(
        option,
        short = 'n',
        description = "set noise cancellation mode [on|off|ambient[-<volume>]]",
        arg_name = "on|off|ambient[-<volume>]"
    )]
    denoise: Option<DenoiseMode>,

    #[argh(
        option,
        short = 'e',
        description = "set equalizer preset [default|pop|classical|rock]",
        arg_name = "default|pop|classical|rock"
    )]
    equalizer: Option<EqualizerPreset>,

    #[argh(
        option,
        short = 'b',
        description = "set device round button noise cancellation control set [on-off|off-on|on-ambient|off-ambient|on-off-ambient|off-on-ambient]",
        arg_name = "on-off|off-on|on-ambient|off-ambient|on-off-ambient|off-on-ambient"
    )]
    button: Option<ButtonControlSet>,

    #[argh(switch, short = 'd', description = "disconnect device")]
    disconnect: bool,

    #[argh(switch, short = 'p', description = "power off device")]
    power_off: bool,

    #[argh(switch, short = 'x', description = "re-pair device")]
    re_pair: bool,

    #[argh(switch, short = 'r', description = "reset device to factory defaults")]
    reset: bool,

    #[argh(
        switch,
        short = 'y',
        description = "skip confirmation for unsafe operations"
    )]
    no_confirm: bool,
}

fn main() {
    let client = match EdifierClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    /* no args */
    if env::args().count() <= 1 {
        run_safe_action(|| print_info(&client), "");
        return;
    }

    let args: Args = argh::from_env();

    if args.info {
        run_safe_action(|| print_info(&client), "");
    }

    if let Some(option) = args.denoise {
        run_safe_action(
            || client.set_denoise_mode(option),
            &format!("Noise cancellation mode set to: {option}."),
        );
    }

    if let Some(option) = args.name {
        run_safe_action(
            || client.set_device_name(option.as_str()),
            &format!("Device name set to: {option}."),
        );
    }

    if let Some(option) = args.prompt_vol {
        run_safe_action(
            || client.set_prompt_volume(option),
            &format!("Prompt volume set to: {option}."),
        );
    }

    if let Some(option) = args.game {
        run_safe_action(
            || client.set_game_mode(option),
            &format!("Game mode set to: {option}."),
        )
    }

    if let Some(option) = args.equalizer {
        run_safe_action(
            || client.set_equalizer_preset(option),
            &format!("Equalizer set to: {option}."),
        );
    }

    if let Some(option) = args.button {
        run_safe_action(
            || client.set_button_control_set(option),
            &format!("Button actions set to: {option}."),
        );
    }

    /* Actions that require device disconnection. */

    if let Some(option) = args.ldac {
        run_unsafe_action(
            args.no_confirm,
            || client.set_ldac_mode(option),
            &format!("LDAC mode set to: {option}."),
        )
    }

    if args.disconnect {
        run_unsafe_action(
            args.no_confirm,
            || client.disconnect_bluetooth(),
            "Device disconnected.",
        )
    };

    if args.re_pair {
        run_unsafe_action(args.no_confirm, || client.re_pair(), "Device re-paired.")
    }

    if args.power_off {
        run_unsafe_action(
            args.no_confirm,
            || client.power_off(),
            "Device powered off.",
        )
    }

    if args.reset {
        run_unsafe_action(
            args.no_confirm,
            || client.reset_factory_defaults(),
            "Device settings reset to factory defaults.",
        )
    }
}

fn run_safe_action<F>(action: F, success_message: &str)
where
    F: FnOnce() -> Result<(), String>,
{
    action()
        .and_then(|_| Ok(println!("{success_message}")))
        .unwrap_or_else(|e| eprintln!("{e}"));
}

fn run_unsafe_action<F>(skip_confirmation: bool, action: F, success_message: &str)
where
    F: FnOnce() -> Result<(), String>,
{
    if skip_confirmation || confirm_disconnect() {
        run_safe_action(action, success_message);
    } else {
        println!("Operation cancelled.");
    }
}

fn confirm_disconnect() -> bool {
    print!("The device will be disconnected. Continue? (y/n): ");
    stdout().flush().ok();

    let mut buffer = String::new();
    if stdin().read_line(&mut buffer).is_err() {
        return false;
    }
    let answer = buffer.trim();
    answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes")
}

fn print_info(client: &EdifierClient) -> Result<(), String> {
    println!("Device name: {}", client.get_device_name()?);
    println!("LDAC mode: {}", client.get_ldac_mode()?);
    println!("Battery level: {}%", client.get_battery_level()?);
    println!("Noise cancellation mode: {}", client.get_denoise_mode()?);
    println!(
        "Prompt voice volume: {} of {}",
        client.get_prompt_volume()?,
        MAX_PROMPT_VOLUME
    );
    println!(
        "Control button actions: {}",
        client.get_button_control_set()?
    );
    println!("Game mode: {}", client.get_game_mode()?);
    println!("Equalizer preset: {}", client.get_equalizer_preset()?);
    println!("Mac address: {}", client.get_mac_address()?);
    println!("Firmware version: {}", client.get_firmware_version()?);
    println!("Fingerprint: {}", client.get_fingerprint()?);

    Ok(())
}
