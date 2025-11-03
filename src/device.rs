use crate::message::EdifierMessage;
use crate::utils::join_str;
use crate::{bluetooth, utils};
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

#[derive(Debug, Copy, Clone, FromRepr, EnumString, Display)]
#[repr(u8)]
#[strum(ascii_case_insensitive)]
pub enum LdacMode {
    Off = 0x00,
    K48 = 0x01,
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

// #[derive(Copy, Clone, FromRepr, EnumString, Display)]
// #[repr(u8)]
// #[strum(ascii_case_insensitive)]
// pub enum AmbientVolume {
//     Plus3 = 0x09,
//     Plus2 = 0x08,
//     Plus1 = 0x07,
//     Default = 0x06,
//     Minus1 = 0x05,
//     Minus2 = 0x04,
//     Minus3 = 0x03,
// }

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
    OnOff = 0x03,
    OnAmbient = 0x06,
    OffAmbient = 0x05,
    All = 0x07,
}

impl From<&Vec<NoiseCancellationMode>> for ButtonControlSet {
    fn from(set: &Vec<NoiseCancellationMode>) -> Self {
        use ButtonControlSet::*;
        use NoiseCancellationMode::*;

        match set {
            v if v.contains(&On) && v.contains(&Off) && v.contains(&Ambient) => All,
            v if v.contains(&On) && v.contains(&Off) => OnOff,
            v if v.contains(&On) && v.contains(&Ambient) => OnAmbient,
            v if v.contains(&Off) && v.contains(&Ambient) => OffAmbient,
            _ => panic!("invalid combination"),
        }
    }
}

impl Into<Vec<NoiseCancellationMode>> for ButtonControlSet {
    fn into(self) -> Vec<NoiseCancellationMode> {
        use ButtonControlSet::*;
        use NoiseCancellationMode::*;
        match self {
            OnOff => vec![On, Off],
            OnAmbient => vec![On, Ambient],
            OffAmbient => vec![Ambient, Off],
            All => vec![On, Off, Ambient],
        }
    }
}

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

    pub(crate) fn set_noise_mode(&self, mode: NoiseCancellationMode) -> Result<(), String> {
        self.send(CMD_SET_NOISE_MODE, Some(&[mode as u8]))?;

        Ok(())
    }

    // pub(crate) fn get_ambient_volume(&self) -> Result<u8, String> {
    //     let response = self.send(CMD_GET_NOISE_MODE, None)?;
    //     Ok(response.payload().unwrap()[1] - 2)
    // }

    // pub(crate) fn set_ambient_volume(&self, mode: NoiseMode) -> Result<(), String> {
    //     self.send(CMD_SET_NOISE_MODE, Some(&[mode as u8]))?;
    //
    //     Ok(())
    // }

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

    pub(crate) fn get_button_control_set(&self) -> Result<Vec<NoiseCancellationMode>, String> {
        let response = self.send(CMD_GET_BUTTON_CONTROL_SET, Some(&[0x0A]))?;
        let value = response.payload().unwrap()[1];
        let result = ButtonControlSet::from_repr(value).unwrap();

        Ok(result.into())
    }

    pub(crate) fn set_button_control_set(
        &self,
        set: &Vec<NoiseCancellationMode>,
    ) -> Result<(), String> {
        let value = ButtonControlSet::from(set);
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

        if response.command_code() != request.command_code() {
            return Err(format!(
                "Response [{}] does not match request command [{:#04X}]",
                response,
                request.command_code().unwrap()
            ));
        }

        Ok(response)
    }
}

impl Drop for EdifierClient {
    fn drop(&mut self) {
        bluetooth::disconnect(self.socket);
    }
}

#[cfg(test)]
mod test {
    use crate::device::NoiseCancellationMode::{Ambient, Off, On};
    use crate::device::{ButtonControlSet, EdifierClient, NoiseCancellationMode};
    use std::sync::{LazyLock, Mutex};

    /// Prevents of using same socket in test simultaneously
    static SOCKET_GUARD: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn test_get_device_name() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let name = client.get_device_name();
        println!("{:?}", name);
        assert!(name.is_ok());
    }

    #[test]
    fn test_set_device_name() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let result = client.set_device_name("SOME DEVICE");

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_mac_address() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let result = client.get_mac_address();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_battery_level() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let result = client.get_battery_level();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_prompt_volume() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let result = client.get_prompt_volume();

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_prompt_volume() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let result = client.set_prompt_volume(2);

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_noise_mode() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let result = client.get_noise_mode();

        assert!(result.is_ok());
        println!("{}", result.unwrap());
    }

    #[test]
    fn test_get_button_control_set() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let result = client.get_button_control_set();

        assert!(result.is_ok());
        println!("{:?}", result.unwrap());
    }

    #[test]
    fn test_set_button_control_set() {
        let _guard = SOCKET_GUARD.lock().unwrap();
        let client = EdifierClient::new().unwrap();

        let result = client.set_button_control_set(&vec![On, Off, Ambient]);
        assert!(result.is_ok());

        println!("{:?}", client.get_button_control_set().unwrap());
    }

    #[test]
    fn test_button_control_set_from_noise_mode() {
        use ButtonControlSet::*;
        use NoiseCancellationMode::*;

        assert_eq!(OnOff, ButtonControlSet::from(&vec![On, Off]));
        assert_eq!(OnOff, ButtonControlSet::from(&vec![Off, On]));
        assert_eq!(OnAmbient, ButtonControlSet::from(&vec![On, Ambient]));
        assert_eq!(OffAmbient, ButtonControlSet::from(&vec![Ambient, Off]));
        assert_eq!(All, ButtonControlSet::from(&vec![On, Off, Ambient]));
    }

    #[test]
    fn test_button_control_set_into_noise_mode() {
        use ButtonControlSet::*;
        use NoiseCancellationMode::*;

        let mut v: Vec<NoiseCancellationMode> = OnOff.into();
        assert_eq!(vec![On, Off], v);

        v = OnAmbient.into();
        assert_eq!(vec![On, Ambient], v);

        v = OffAmbient.into();
        assert_eq!(vec![Ambient, Off], v);

        v = All.into();
        assert_eq!(vec![On, Off, Ambient], v);
    }
}
