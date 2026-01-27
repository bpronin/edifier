use crate::device::ButtonControlSet::{
    Default, OffAmbient, OffOn, OffOnAmbient, OnAmbient, OnOff, OnOffAmbient,
};
use crate::device::LdacMode::{K48, K96, Off};
use crate::message::EdifierMessage;
use crate::utils::join_str;
use crate::{bluetooth, utils};
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::{Display, EnumString, FromRepr};
use utils::join_hex;
use windows::Win32::Networking::WinSock::SOCKET;
use windows_core::GUID;

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

static SPP_UUID: GUID = GUID::from_u128(0xEDF00000_EDFE_DFED_FEDF_EDFEDFEDFEDF);

#[derive(Debug)]
pub struct EdifierClient {
    socket: SOCKET,
}

impl EdifierClient {
    pub(crate) fn new() -> Result<EdifierClient, String> {
        Ok(Self {
            socket: bluetooth::connect(SPP_UUID)?,
        })
    }

    pub(crate) fn get_device_name(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_NAME, None)?;
        let payload = response.payload().unwrap();
        let result = String::from_utf8_lossy(payload.as_ref()).to_string();

        Ok(result)
    }

    pub(crate) fn set_device_name(&self, name: &str) -> Result<(), String> {
        self.send(CMD_SET_NAME, name.as_bytes().into())?;

        Ok(())
    }

    pub(crate) fn get_mac_address(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_MAC_ADDRESS, None)?;
        let result = join_hex(response.payload().unwrap(), ":");

        Ok(result)
    }

    pub(crate) fn get_battery_level(&self) -> Result<u8, String> {
        let response = self.send(CMD_GET_BATTERY_LEVEL, None)?;
        let result = response.payload().unwrap()[0];

        Ok(result)
    }

    pub(crate) fn get_firmware_version(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_FIRMWARE_VERSION, None)?;
        let result = join_str(response.payload().unwrap(), ".");

        Ok(result)
    }

    pub(crate) fn get_fingerprint(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_FINGERPRINT, None)?;
        let result = join_hex(response.payload().unwrap(), " ");

        Ok(result)
    }

    pub(crate) fn get_game_mode(&self) -> Result<GameMode, String> {
        let response = self.send(CMD_GET_GAME_MODE, None)?;
        let value = response.payload().unwrap()[0];
        let result = GameMode::from_repr(value).unwrap();

        Ok(result)
    }

    pub(crate) fn set_game_mode(&self, mode: GameMode) -> Result<(), String> {
        self.send(CMD_SET_GAME_MODE, Some(&[mode as u8]))?;

        Ok(())
    }

    pub(crate) fn get_ldac_mode(&self) -> Result<LdacMode, String> {
        let response = self.send(CMD_GET_LDAC_MODE, None)?;
        let value = response.payload().unwrap()[0];
        let result = LdacMode::from_repr(value).unwrap();

        Ok(result)
    }

    pub(crate) fn set_ldac_mode(&self, mode: LdacMode) -> Result<(), String> {
        self.send(CMD_SET_LDAC_MODE, Some(&[mode as u8]))?;

        Ok(())
    }

    pub(crate) fn get_noise_mode(&self) -> Result<NoiseCancellationMode, String> {
        let response = self.send(CMD_GET_NOISE_MODE, None)?;
        let value = response.payload().unwrap()[0];
        let result = NoiseCancellationMode::from_repr(value).unwrap();

        Ok(result)
    }

    pub(crate) fn set_noise_mode(
        &self,
        mode: NoiseCancellationMode,
        ambient_volume: Option<u8>,
    ) -> Result<(), String> {
        if let Some(volume) = ambient_volume {
            self.send(CMD_SET_NOISE_MODE, Some(&[mode as u8, volume]))?;
        } else {
            self.send(CMD_SET_NOISE_MODE, Some(&[mode as u8]))?;
        }

        Ok(())
    }

    pub(crate) fn get_ambient_volume(&self) -> Result<u8, String> {
        let response = self.send(CMD_GET_NOISE_MODE, None)?;
        let value = response.payload().unwrap()[1];

        Ok(value)
    }

    pub(crate) fn get_equalizer_preset(&self) -> Result<EqualizerPreset, String> {
        let response = self.send(CMD_GET_EQUALIZER_PRESET, None)?;
        let value = response.payload().unwrap()[0];
        let result = EqualizerPreset::from_repr(value).unwrap();

        Ok(result)
    }

    pub(crate) fn set_equalizer_preset(&self, preset: EqualizerPreset) -> Result<(), String> {
        self.send(CMD_SET_EQUALIZER_PRESET, Some(&[preset as u8]))?;

        Ok(())
    }

    pub(crate) fn get_button_control_set(&self) -> Result<ButtonControlSet, String> {
        let response = self.send(CMD_GET_BUTTON_CONTROL_SET, Some(&[0x0A]))?;
        let value = response.payload().unwrap()[1];
        let result = ButtonControlSet::from_repr(value).unwrap();

        Ok(result.into())
    }

    pub(crate) fn set_button_control_set(&self, value: ButtonControlSet) -> Result<(), String> {
        self.send(CMD_SET_BUTTON_CONTROL_SET, Some(&[0x0A, value as u8]))?;

        Ok(())
    }

    pub(crate) fn get_prompt_volume(&self) -> Result<u8, String> {
        let response = self.send(CMD_GET_PROMPT_VOLUME, None)?;

        Ok(response.payload().unwrap()[0])
    }

    pub(crate) fn set_prompt_volume(&self, volume: u8) -> Result<(), String> {
        self.send(CMD_SET_PROMPT_VOLUME, Some(&[volume]))?;

        Ok(())
    }

    pub(crate) fn re_pair(&self) -> Result<(), String> {
        self.send(CMD_RE_PAIR, None)?;

        Ok(())
    }

    pub(crate) fn disconnect_bluetooth(&self) -> Result<(), String> {
        self.send(CMD_DISCONNECT_BLUETOOTH, None)?;

        Ok(())
    }

    pub(crate) fn power_off(&self) -> Result<(), String> {
        self.send(CMD_POWER_OFF, None)?;

        Ok(())
    }

    pub(crate) fn reset_factory_defaults(&self) -> Result<(), String> {
        self.send(CMD_RESET_FACTORY_DEFAULTS, None)?;

        Ok(())
    }

    fn send(&self, command_code: u8, payload: Option<&[u8]>) -> Result<EdifierMessage, String> {
        let request = EdifierMessage::new(command_code, payload);
        let response: EdifierMessage = bluetooth::send(self.socket, request.as_slice())?.into();

        /*if response.command_code() != request.command_code() {
            //todo: is [BB, 02, C3, 0D, 21, A6] an error?
            return Err(format!(
                "Response {} does not match request command [{:#04X}]",
                response,
                request.command_code().unwrap()
            ));
        }*/

        Ok(response)
    }
}

impl Drop for EdifierClient {
    fn drop(&mut self) {
        bluetooth::disconnect(self.socket);
    }
}
#[derive(Debug, Copy, Clone, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum PlaybackStatus {
    Stopped = 0x03,
    Playing = 0x0D,
}

#[derive(Debug, Copy, Clone, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum GameMode {
    Off = 0x00,
    On = 0x01,
}

#[derive(Debug, Copy, Clone, FromRepr)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum LdacMode {
    Off = 0x00,
    K48 = 0x01,
    K96 = 0x02,
}

impl FromStr for LdacMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "OFF" => Ok(Off),
            "48K" => Ok(K48),
            "96K" => Ok(K96),
            _ => Err("Illegal LDAC mode")?,
        }
    }
}

impl Display for LdacMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Off => f.write_str("Off"),
            K48 => f.write_str("48K"),
            K96 => f.write_str("96K"),
        }
    }
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

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum NoiseCancellationMode {
    Off = 0x01,
    On = 0x02,
    Ambient = 0x03,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum ButtonControlSet {
    Default = 0,
    OffOn = 1,
    OnOff = 3,
    OffAmbient = 4,
    OnAmbient = 6,
    OnOffAmbient = 7,
    OffOnAmbient = 8,
}

impl ButtonControlSet {
    pub(crate) fn from_arg(string: &str) -> Result<ButtonControlSet, String> {
        match string.trim().to_lowercase() {
            s if s == "default" => Ok(Default),
            s if s == "off-on" => Ok(OffOn),
            s if s == "on-off" => Ok(OnOff),
            s if s == "off-ambient" => Ok(OffAmbient),
            s if s == "on-ambient" => Ok(OnAmbient),
            s if s == "on-off-ambient" => Ok(OnOffAmbient),
            s if s == "off-on-ambient" => Ok(OffOnAmbient),
            s => Err(format!("`{s}` is an illegal button control set"))?,
        }
    }

    pub(crate) fn to_arg(&self) -> String {
        match self {
            Default => "default",
            OffOn => "off-on",
            OnOff => "on-off",
            OffAmbient => "off-ambient",
            OnAmbient => "on-ambient",
            OnOffAmbient => "on-off-ambient",
            OffOnAmbient => "off-on-ambient",
        }
        .to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    /// Prevents using the same socket in test simultaneously
    static SOCKET_GUARD: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn get_client() -> EdifierClient {
        let _guard = SOCKET_GUARD.lock().unwrap();
        EdifierClient::new().unwrap()
    }

    #[test]
    fn test_get_device_name() {
        let client = get_client();

        let name = client.get_device_name();

        println!("{:?}", name);
        assert!(name.is_ok());
    }

    #[test]
    fn test_set_device_name() {
        let result = get_client().set_device_name("SOME DEVICE");

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
    fn test_get_ambient_volume() {
        let result = get_client().get_ambient_volume();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_noise_mode() {
        let result = get_client().get_noise_mode();

        assert!(result.is_ok());
        println!("{}", result.unwrap());
    }

    #[test]
    fn test_set_noise_mode() {
        let result = get_client().set_noise_mode(NoiseCancellationMode::Ambient, Some(12));

        println!("{:?}", result);
        assert!(result.is_ok());

        let result = get_client().set_noise_mode(NoiseCancellationMode::On, None);

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_button_control_set() {
        let result = get_client().get_button_control_set();

        assert!(result.is_ok());
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_set_button_control_set() {
        let result = get_client().set_button_control_set(Default);

        assert!(result.is_ok());
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_button_control_set_to_arg() {
        assert_eq!("on-off", OnOff.to_arg().as_str());
        assert_eq!("off-ambient", OffAmbient.to_arg().as_str());
        assert_eq!("on-ambient", OnAmbient.to_arg().as_str());
        assert_eq!("on-off-ambient", OnOffAmbient.to_arg().as_str());
        assert_eq!("off-on-ambient", OffOnAmbient.to_arg().as_str());
        assert_eq!("default", Default.to_arg().as_str());
    }

    #[test]
    fn test_button_control_set_from_arg() {
        use ButtonControlSet::*;

        assert_eq!(Ok(OnOff), ButtonControlSet::from_arg("on-off"));
        assert_eq!(Ok(OffOn), ButtonControlSet::from_arg("off-on"));
        assert_eq!(Ok(OffAmbient), ButtonControlSet::from_arg("off-ambient"));
        assert_eq!(Ok(OnAmbient), ButtonControlSet::from_arg("on-ambient"));
        assert_eq!(Ok(OnOffAmbient), ButtonControlSet::from_arg("on-off-ambient"));
        assert_eq!(Ok(OffOnAmbient), ButtonControlSet::from_arg("off-on-ambient"));

        assert_eq!(Ok(OnOff), ButtonControlSet::from_arg("ON-OFF"));
        assert_eq!(
            Err("`banana` is an illegal button control set".to_string()),
            ButtonControlSet::from_arg("banana")
        );
    }
}
