use crate::bluetooth;
use windows::Win32::Networking::WinSock::SOCKET;
use windows_core::GUID;

static SPP_UUID: GUID = GUID::from_u128(0xEDF00000_EDFE_DFED_FEDF_EDFEDFEDFEDF);

#[derive(Default)]
pub struct EdifierDevice {
    socket: SOCKET,
    is_connected: bool,
}

impl EdifierDevice {
    pub(crate) fn connect(&mut self) -> Result<(), String> {
        if !self.is_connected {
            self.socket = bluetooth::open_socket(SPP_UUID)?;
            self.is_connected = true;
        }
        
        Ok(())
    }

    pub(crate) fn read_raw(&self) -> Result<Vec<u8>, String> {
        bluetooth::read_socket(self.socket)
    }

    pub(crate) fn write_raw(&self, data: &[u8]) -> Result<(), String> {
        bluetooth::write_socket(self.socket, data)
    }
}

impl Drop for EdifierDevice {
    fn drop(&mut self) {
        if !self.is_connected {
            bluetooth::close_socket(self.socket);
            self.is_connected = false;
        }
    }
}
