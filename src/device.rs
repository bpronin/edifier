//! The EdifierClient struct provides a client for establishing a Bluetooth connection
//! with an Edifier device using its Serial Port Profile (SPP) service.
//! It allows control over various device features such as game mode, LDAC mode, equalizer presets,
//! noise cancellation modes, and more.
use crate::message::EdifierMessage;
use crate::utils::join_str;
use crate::{bluetooth, err, utils};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum_macros::{Display, EnumString, FromRepr};
use utils::join_hex;
use windows::Win32::Networking::WinSock::SOCKET;
use windows_core::GUID;
use DenoiseMode::{Ambient, Off, On};

const CMD_GET_PROMPT_VOLUME: u8 = 0x05;
const CMD_SET_PROMPT_VOLUME: u8 = 0x06;
const CMD_RESET_FACTORY_DEFAULTS: u8 = 0x07;
const CMD_GET_GAME_MODE: u8 = 0x08;
const CMD_SET_GAME_MODE: u8 = 0x09;
const CMD_GET_LDAC_MODE: u8 = 0x48;
const CMD_SET_LDAC_MODE: u8 = 0x49;
const CMD_SET_NOISE_MODE: u8 = 0xC1;
// const CMD_GET_PLAYBACK_STATUS: u8 = 0xC3;
const CMD_SET_EQUALIZER_PRESET: u8 = 0xC4;
const CMD_GET_FIRMWARE_VERSION: u8 = 0xC6;
const CMD_GET_MAC_ADDRESS: u8 = 0xC8;
const CMD_GET_NAME: u8 = 0xC9;
const CMD_SET_NAME: u8 = 0xCA;
const CMD_GET_NOISE_MODE: u8 = 0xCC;
const CMD_DISCONNECT_BLUETOOTH: u8 = 0xCD;
const CMD_POWER_OFF: u8 = 0xCE;
const CMD_RE_PAIR: u8 = 0xCF;
const CMD_GET_BATTERY_LEVEL: u8 = 0xD0;
// const CMD_SET_AUTO_POWER_OFF_TIME: u8 = 0xD1;
// const CMD_DISABLE_AUTO_POWER_OFF: u8 = 0xD2;
// const CMD_GET_AUTO_POWER_OFF_TIME: u8 = 0xD3;
const CMD_GET_EQUALIZER_PRESET: u8 = 0xD5;
const CMD_GET_FINGERPRINT: u8 = 0xD8;
const CMD_GET_BUTTON_CONTROL_SET: u8 = 0xF0;
const CMD_SET_BUTTON_CONTROL_SET: u8 = 0xF1;

pub(crate) const MAX_PROMPT_VOLUME: u8 = 15;
pub(crate) const MAX_AMBIENT_VOLUME: u8 = 12;

const SPP_UUID: GUID = GUID::from_u128(0xEDF00000_EDFE_DFED_FEDF_EDFEDFEDFEDF);

/// Provides a Bluetooth client for controlling an Edifier device through its SPP service.
#[derive(Debug)]
pub struct EdifierClient {
    socket: SOCKET,
}

impl EdifierClient {

    /// Creates a new Edifier client connected through the device SPP Bluetooth service.
    pub(crate) fn new() -> Result<EdifierClient, String> {
        Ok(Self {
            socket: bluetooth::connect(&SPP_UUID)?,
        })
    }

    /// Resets Bluetooth pairing-related services for the Edifier device.
    pub(crate) fn pair() -> Result<(), String> {
        bluetooth::pair(&SPP_UUID)
    }

    /// Returns the current Bluetooth device name.
    pub(crate) fn get_device_name(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_NAME, None)?;
        let payload = response.payload().unwrap();
        let result = String::from_utf8_lossy(payload.as_ref()).to_string();

        Ok(result)
    }

    /// Sets the Bluetooth device name.
    pub(crate) fn set_device_name(&self, name: &str) -> Result<(), String> {
        self.send(CMD_SET_NAME, name.as_bytes().into())?;

        Ok(())
    }

    /// Returns the device MAC address formatted as hexadecimal bytes.
    pub(crate) fn get_mac_address(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_MAC_ADDRESS, None)?;
        let result = join_hex(response.payload().unwrap(), ":");

        Ok(result)
    }

    /// Returns the current battery level percentage reported by the device.
    pub(crate) fn get_battery_level(&self) -> Result<u8, String> {
        let response = self.send(CMD_GET_BATTERY_LEVEL, None)?;
        let result = response.payload().unwrap()[0];

        Ok(result)
    }

    /// Returns the firmware version reported by the device.
    pub(crate) fn get_firmware_version(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_FIRMWARE_VERSION, None)?;
        let result = join_str(response.payload().unwrap(), ".");

        Ok(result)
    }

    /// Returns the device fingerprint formatted as hexadecimal bytes.
    pub(crate) fn get_fingerprint(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_FINGERPRINT, None)?;
        let result = join_hex(response.payload().unwrap(), " ");

        Ok(result)
    }

    /// Returns the current game mode state.
    pub(crate) fn get_game_mode(&self) -> Result<GameMode, String> {
        let response = self.send(CMD_GET_GAME_MODE, None)?;
        let value = response.payload().unwrap()[0];
        let result = GameMode::from_repr(value).expect("Invalid game mode");

        Ok(result)
    }

    /// Enables or disables game mode.
    pub(crate) fn set_game_mode(&self, mode: GameMode) -> Result<(), String> {
        self.send(CMD_SET_GAME_MODE, Some(&[mode as u8]))?;

        Ok(())
    }

    /// Returns the current LDAC mode.
    pub(crate) fn get_ldac_mode(&self) -> Result<LdacMode, String> {
        let response = self.send(CMD_GET_LDAC_MODE, None)?;
        let value = response.payload().unwrap()[0];
        let result = LdacMode::from_repr(value).expect("Invalid LDAC mode");

        Ok(result)
    }

    /// Sets the LDAC mode.
    pub(crate) fn set_ldac_mode(&self, mode: LdacMode) -> Result<(), String> {
        self.send(CMD_SET_LDAC_MODE, Some(&[mode as u8]))?;
        // todo: reopen bluetooth socket
        Ok(())
    }

    /// Returns the current noise cancellation mode.
    pub(crate) fn get_denoise_mode(&self) -> Result<DenoiseMode, String> {
        let response = self.send(CMD_GET_NOISE_MODE, None)?;
        let payload = response.payload().unwrap();
        let result = DenoiseMode::from_code(payload[0], Some(payload[1]))?;

        Ok(result)
    }

    /// Sets the noise cancellation mode and optional ambient volume.
    pub(crate) fn set_denoise_mode(&self, mode: DenoiseMode) -> Result<(), String> {
        let payload = match mode {
            Ambient(volume) => match volume {
                None => vec![mode.code()],
                Some(v) => vec![mode.code(), v],
            },
            _ => vec![mode.code()],
        };

        self.send(CMD_SET_NOISE_MODE, Some(payload.as_slice()))?;

        Ok(())
    }

    /// Returns the current equalizer preset.
    pub(crate) fn get_equalizer_preset(&self) -> Result<EqualizerPreset, String> {
        let response = self.send(CMD_GET_EQUALIZER_PRESET, None)?;
        let value = response.payload().unwrap()[0];
        let result = EqualizerPreset::from_repr(value).expect("Invalid equalizer preset");

        Ok(result)
    }

    /// Sets the equalizer preset.
    pub(crate) fn set_equalizer_preset(&self, preset: EqualizerPreset) -> Result<(), String> {
        self.send(CMD_SET_EQUALIZER_PRESET, Some(&[preset as u8]))?;

        Ok(())
    }

    /// Returns the configured button control set.
    pub(crate) fn get_button_control_set(&self) -> Result<ButtonControlSet, String> {
        let response = self.send(CMD_GET_BUTTON_CONTROL_SET, Some(&[0x0A]))?;
        let value = response.payload().unwrap()[1];
        let result = ButtonControlSet::from_repr(value).expect("Invalid button control set");

        Ok(result)
    }

    /// Sets the button control configuration.
    pub(crate) fn set_button_control_set(&self, set: ButtonControlSet) -> Result<(), String> {
        self.send(CMD_SET_BUTTON_CONTROL_SET, Some(&[0x0A, set as u8]))?;

        Ok(())
    }

    /// Returns the current prompt volume.
    pub(crate) fn get_prompt_volume(&self) -> Result<u8, String> {
        let response = self.send(CMD_GET_PROMPT_VOLUME, None)?;
        let result = response.payload().unwrap()[0];

        Ok(result)
    }

    /// Sets the prompt volume.
    pub(crate) fn set_prompt_volume(&self, volume: u8) -> Result<(), String> {
        if volume > MAX_PROMPT_VOLUME {
            err!("Prompt volume must be from 0 to {MAX_PROMPT_VOLUME}.")
        } else {
            self.send(CMD_SET_PROMPT_VOLUME, Some(&[volume]))?;

            Ok(())
        }
    }

    /// Puts the device into re-pairing mode.
    pub(crate) fn unpair(&self) -> Result<(), String> {
        self.send(CMD_RE_PAIR, None)?;

        Ok(())
    }

    /// Disconnects the current Bluetooth connection from the device side.
    pub(crate) fn disconnect_bluetooth(&self) -> Result<(), String> {
        self.send(CMD_DISCONNECT_BLUETOOTH, None)?;

        Ok(())
    }

    /// Powers off the device.
    pub(crate) fn power_off(&self) -> Result<(), String> {
        self.send(CMD_POWER_OFF, None)?;

        Ok(())
    }

    /// Resets the device to factory defaults.
    pub(crate) fn reset_factory_defaults(&self) -> Result<(), String> {
        self.send(CMD_RESET_FACTORY_DEFAULTS, None)?;

        Ok(())
    }

    fn send(&self, command_code: u8, payload: Option<&[u8]>) -> Result<EdifierMessage, String> {
        let request = EdifierMessage::new(command_code, payload);
        let response: EdifierMessage = bluetooth::send(self.socket, request.as_slice())?.into();

        /*if response.command_code() != request.command_code() {
            //todo: is [BB, 02, C3, 0D, 21, A6] an error?
            return format_err!(
                "Response {} does not match request command [{:#04X}]",
                response,
                request.command_code().unwrap()
            );
        }*/

        Ok(response)
    }
}

impl Drop for EdifierClient {
    fn drop(&mut self) {
        bluetooth::disconnect(self.socket);
    }
}

/*
    #[derive(Debug, Copy, Clone, FromRepr, EnumString, Display)]
    #[repr(u8)]
    #[strum(ascii_case_insensitive)]
    pub enum PlaybackStatus {
        Stopped = 0x03,
        Playing = 0x0D,
    }
*/

#[derive(Debug, Copy, Clone, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum GameMode {
    Off = 0x00,
    On = 0x01,
}

#[derive(Debug, Copy, Clone, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum LdacMode {
    Off = 0x00,
    #[strum(serialize = "48K")]
    K48 = 0x01,
    #[strum(serialize = "96K")]
    K96 = 0x02,
}

#[derive(Debug, Copy, Clone, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum EqualizerPreset {
    Default = 0x00, /* AKA "Classic" */
    Pop = 0x01,
    Classical = 0x02,
    Rock = 0x03,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum DenoiseMode {
    Off,
    On,
    Ambient(Option<u8>),
}

impl DenoiseMode {
    fn from_name(name: &str, volume: Option<u8>) -> Result<Self, String> {
        match name.trim().to_lowercase().as_str() {
            "off" => Ok(Off),
            "on" => Ok(On),
            "ambient" => Ok(Ambient(volume)),
            _ => err!("Illegal noise cancellation mode name"),
        }
    }

    fn from_code(code: u8, volume: Option<u8>) -> Result<Self, String> {
        match code {
            0x01 => Ok(Off),
            0x02 => Ok(On),
            0x03 => Ok(Ambient(volume)),
            _ => err!("Illegal noise cancellation mode code"),
        }
    }

    /// Returns the protocol command code for this noise cancellation mode.
    pub fn code(&self) -> u8 {
        match self {
            Off => 0x01,
            On => 0x02,
            Ambient(_) => 0x03,
        }
    }
}

impl Display for DenoiseMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Off => "Off".to_string(),
            On => "On".to_string(),
            Ambient(volume) => match volume {
                None => "Ambient".to_string(),
                Some(v) => format!("Ambient (volume: {v} of {MAX_AMBIENT_VOLUME})"),
            },
        };
        write!(f, "{s}")
    }
}

impl FromStr for DenoiseMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<_> = s.split('-').collect();

        let volume = if split.len() == 2 {
            let sv = split[1];
            let v = sv
                .parse()
                .map_err(|_| format!("Invalid ambient volume value: `{sv}`."))?;

            if v > MAX_AMBIENT_VOLUME {
                err!("Ambient volume must be from 0 to {MAX_AMBIENT_VOLUME}.")?
            } else {
                Some(v)
            }
        } else {
            None
        };

        Self::from_name(split[0], volume)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum ButtonControlSet {
    #[strum(serialize = "Default")]
    #[strum(serialize = "0")]
    Default = 0x00,
    #[strum(serialize = "Off-On")]
    #[strum(serialize = "1")]
    OffOn = 0x01,
    #[strum(serialize = "On-Off")]
    #[strum(serialize = "2")]
    OnOff = 0x03,
    #[strum(serialize = "Off-Ambient")]
    #[strum(serialize = "3")]
    OffAmbient = 0x04,
    #[strum(serialize = "On-Ambient")]
    #[strum(serialize = "4")]
    OnAmbient = 0x06,
    #[strum(serialize = "On-Off-Ambient")]
    #[strum(serialize = "6")]
    OnOffAmbient = 0x07,
    #[strum(serialize = "5")]
    #[strum(serialize = "Off-On-Ambient")]
    OffOnAmbient = 0x08,
}

#[cfg(test)]
mod test {
    use crate::device::{ButtonControlSet, DenoiseMode, EdifierClient, LdacMode};
    use std::sync::{LazyLock, Mutex};

    /// Prevents using the same socket in tests simultaneously
    static SOCKET_GUARD: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn get_client() -> EdifierClient {
        let _guard = SOCKET_GUARD.lock().unwrap();
        EdifierClient::new().unwrap()
    }

    #[test]
    fn test_get_device_name() {
        let result = get_client().get_device_name();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_device_name() {
        let result = get_client().set_device_name("BANANA DEVICE");

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_mac_address() {
        let result = get_client().get_mac_address();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_battery_level() {
        let result = get_client().get_battery_level();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_prompt_volume() {
        let result = get_client().get_prompt_volume();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_prompt_volume() {
        let result = get_client().set_prompt_volume(2);

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_denoise_mode() {
        let result = get_client().get_denoise_mode();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_denoise_mode() {
        let result = get_client().set_denoise_mode(DenoiseMode::Ambient(Some(7)));

        println!("{:?}", result);
        assert!(result.is_ok());

        let result = get_client().set_denoise_mode(DenoiseMode::On);

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_button_control_set() {
        let result = get_client().get_button_control_set();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_button_control_set() {
        let result = get_client().set_button_control_set(ButtonControlSet::Default);

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_ldac_mode() {
        let result = get_client().set_ldac_mode(LdacMode::K96);

        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
