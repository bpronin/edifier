use crate::bluetooth;
use crate::message::EdifierMessage;
use windows::Win32::Networking::WinSock::SOCKET;
use windows_core::GUID;

static SPP_UUID: GUID = GUID::from_u128(0xEDF00000_EDFE_DFED_FEDF_EDFEDFEDFEDF);

#[derive(Default)]
pub struct EdifierClient {
    socket: SOCKET,
}

impl EdifierClient {
    pub(crate) fn power_off_device(&self) -> Result<(), String> {
        self.send(0xCE, None)?;

        Ok(())
    }

    pub(crate) fn get_mac_address(&self) -> Result<String, String> {
        let response = self.send(0xC8, None)?;
        let payload = response.payload().unwrap();
        let result = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            payload[0], payload[1], payload[2], payload[3], payload[4], payload[5]
        );

        Ok(result)
    }

    pub(crate) fn get_device_name(&self) -> Result<String, String> {
        let response = self.send(0xC9, None)?;
        let pl = response.payload().unwrap();
        let result = String::from_utf8_lossy(pl.as_ref());

        Ok(result.to_string())
    }

    pub(crate) fn set_device_name(&self, name: &str) -> Result<(), String> {
        todo!()
    }

    pub(crate) fn connect(&mut self) -> Result<(), String> {
        self.socket = bluetooth::connect(SPP_UUID)?;
        Ok(())
    }

    fn send(&self, command_code: u8, payload: Option<&[u8]>) -> Result<EdifierMessage, String> {
        let request = EdifierMessage::new(command_code, payload);
        let response_bytes = bluetooth::send(self.socket, request.bytes.as_ref())?;
        let response = EdifierMessage { bytes: response_bytes.to_vec(), };

        Ok(response)
    }
}

impl Drop for EdifierClient {
    fn drop(&mut self) {
        bluetooth::disconnect(self.socket);
    }
}
