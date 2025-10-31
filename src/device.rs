use crate::bluetooth;
use crate::message::EdifierMessage;
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
const CMD_GET_PLAYBACK_STATUS: u8 = 0xC3;
const CMD_SET_EQ_MODE: u8 = 0xC4;
const CMD_GET_FIRMWARE_VERSION: u8 = 0xC6;
const CMD_GET_MAC_ADDRESS: u8 = 0xC8;
const CMD_GET_NAME: u8 = 0xC9;
const CMD_SET_NAME: u8 = 0xCA;
const CMD_GET_NOISE_MODE: u8 = 0xCC;
const CMD_DISCONNECT_BLUETOOTH: u8 = 0xCD;
const CMD_POWER_OFF: u8 = 0xCE;
const CMD_RE_PAIR: u8 = 0xCF;
const CMD_GET_BATTERY_LEVEL: u8 = 0xD0;
const CMD_SET_AUTO_POWER_OFF_TIME: u8 = 0xD1;
const CMD_DISABLE_AUTO_POWER_OFF: u8 = 0xD2;
const CMD_GET_AUTO_POWER_OFF_TIME: u8 = 0xD3;
const CMD_GET_EQ_MODE: u8 = 0xD5;
const CMD_GET_FINGERPRINT: u8 = 0xD8;
const CMD_GET_BUTTON_CONTROL: u8 = 0xF0;
const CMD_SET_BUTTON_CONTROL: u8 = 0xF1;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Stopped = 0x03,
    Playing = 0x0D,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Off = 0x00,
    On = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LdacMode {
    Off = 0x00,
    K48 = 0x01,
    K96 = 0x02,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseCancellationMode {
    Off = 0x01,
    On = 0x02,
    Ambient = 0x03,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqMode {
    Classic = 0x00,
    Pop = 0x01,
    Classical = 0x02,
    Rock = 0x03,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientVolume {
    Plus3 = 0x09,
    Plus2 = 0x08,
    Plus1 = 0x07,
    Default = 0x06,
    Minus1 = 0x05,
    Minus2 = 0x04,
    Minus3 = 0x03,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyControlSet {
    All = 0x07,
    AmbientAndCancellation = 0x06,
    OffAndAmbient = 0x05,
    OffAndCancellation = 0x03,
    AmbientAndOff = 0x04,
    CancellationAndAmbient = 0x02,
    CancellationAndOff = 0x01,
}

// Дополнительные реализации для удобства
// impl TryFrom<u8> for PlaybackStatus {
//     type Error = ();
//
//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         match value {
//             0x03 => Ok(Self::Stopped),
//             0x0D => Ok(Self::Playing),
//             _ => Err(()),
//         }
//     }
// }
//
// impl TryFrom<u8> for GameMode {
//     type Error = ();
//
//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         match value {
//             0x00 => Ok(Self::Off),
//             0x01 => Ok(Self::On),
//             _ => Err(()),
//         }
//     }
// }

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

    pub(crate) fn get_mac_address(&self) -> Result<String, String> {
        let response = self.send(CMD_GET_MAC_ADDRESS, None)?;
        let payload = response.payload().unwrap();
        let result = payload
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join(":");

        Ok(result)
    }

    pub(crate) fn set_device_name(&self, name: &str) -> Result<(), String> {
        self.send(CMD_SET_NAME, name.as_bytes().into())?;

        Ok(())
    }

    pub(crate) fn power_off_device(&self) -> Result<(), String> {
        self.send(CMD_POWER_OFF, None)?;

        Ok(())
    }

    fn send(&self, command_code: u8, payload: Option<&[u8]>) -> Result<EdifierMessage, String> {
        let request = EdifierMessage::new(command_code, payload);
        let response = bluetooth::send(self.socket, request.as_slice())?;

        Ok(response.into())
    }
}

impl Drop for EdifierClient {
    fn drop(&mut self) {
        bluetooth::disconnect(self.socket);
    }
}

#[cfg(test)]
mod test {
    use crate::device::EdifierClient;
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
}
