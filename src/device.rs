use windows::Win32::Networking::WinSock::SOCKET;
use windows_core::GUID;
use crate::bluetooth;

static SPP_UUID: GUID = GUID::from_u128(0xEDF00000_EDFE_DFED_FEDF_EDFEDFEDFEDF);

#[derive(Default)]
pub struct EdifierClient {
    socket: SOCKET,
}

impl EdifierClient {
    pub(crate) fn power_off_device(&self) -> Result<(), String> {
        let request = &[0xAA, 0x01, 0xCE, 0x21, 0x92];
        self.send(request)?;

        Ok(())
    }

    pub(crate) fn get_mac_address(&self) -> Result<String, String> {
        let request = &[0xAA, 0x01, 0xC8, 0x21, 0x8C];
        let response = self.send(request)?;
        let payload = &response[3..response.len() - 2];
        let result = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            payload[0], payload[1], payload[2], payload[3], payload[4], payload[5]
        );

        Ok(result)
    }

    pub(crate) fn get_device_name(&self) -> Result<String, String> {
        let request = &[0xAA, 0x01, 0xC9, 0x21, 0x8D];
        let response = self.send(request)?;
        let payload = &response[3..response.len() - 2];
        let result = String::from_utf8_lossy(payload);

        Ok(result.to_string())
    }

    pub(crate) fn set_device_name(&self, name: String) -> Result<(), String> {
        todo!()
    }

    pub(crate) fn connect(&mut self) -> Result<(), String> {
        self.socket = bluetooth::connect(SPP_UUID)?;
        Ok(())
    }
    
    pub(crate) fn send(&self, request:&[u8]) -> Result<Vec<u8>, String> {
        bluetooth::send(self.socket, request)
    }
}

impl Drop for EdifierClient {
    fn drop(&mut self) {
        bluetooth::disconnect(self.socket);
    }
}
