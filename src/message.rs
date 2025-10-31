#[derive(Debug, PartialEq, Eq)]
pub struct EdifierMessage {
    bytes: Vec<u8>,
}

impl EdifierMessage {
    pub(crate) fn new(command_code: u8, payload: Option<&[u8]>) -> Self {
        let length = payload.map_or(0, |p| p.len());
        let mut bytes = vec![0u8; length + 5];
        let last_index = bytes.len() - 1;

        /* Header */
        bytes[0] = 0xAA;
        bytes[1] = (length + 1) as u8;
        bytes[2] = command_code;

        /* Payload */
        if let Some(payload) = payload {
            bytes[3..3 + length].copy_from_slice(payload);
        }

        /* CRC */
        let crc = split_into_bytes(compute_crc(&bytes));
        bytes[last_index - 1] = crc[0];
        bytes[last_index] = crc[1];

        Self { bytes }
    }

    pub(crate) fn payload(&self) -> Option<Vec<u8>> {
        let pl_bytes = self.bytes[3..self.bytes.len() - 2].to_vec();
        if pl_bytes.is_empty() {
            None
        } else {
            Some(pl_bytes)
        }
    }

/*
    pub(crate) fn signature(&self) -> u8 {
        self.bytes[0]
    }
    
    pub(crate) fn data_size(&self) -> u8 {
        self.bytes[1]
    }
    
    pub(crate) fn command_code(&self) -> u8 {
        self.bytes[2]
    }
    
    pub(crate) fn crc(&self) -> u16 {
        u16::from_be_bytes([
            self.bytes[self.bytes.len() - 2],
            self.bytes[self.bytes.len() - 1],
        ])
    }
*/
    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl From<Vec<u8>> for EdifierMessage {
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

fn compute_crc(data: &[u8]) -> u16 {
    0x2019 + data.iter().map(|&b| b as u16).sum::<u16>()
}

fn split_into_bytes(value: u16) -> [u8; 2] {
    [(value >> 8) as u8, (value & 0xFF) as u8]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(
            EdifierMessage::new(0xC9, None).bytes,
            vec![0xAA, 0x01, 0xC9, 0x21, 0x8D]
        );
        assert_eq!(
            EdifierMessage::new(0xCE, None).bytes,
            vec![0xAA, 0x01, 0xCE, 0x21, 0x92]
        );
        assert_eq!(
            EdifierMessage::new(0xC8, None).bytes,
            vec![0xAA, 0x01, 0xC8, 0x21, 0x8C]
        );
    }

    #[test]
    fn test_from() {
        assert_eq!(
            EdifierMessage::from(vec![0xAA, 0x01, 0xC9, 0x21, 0x8D]),
            EdifierMessage {
                bytes: vec![0xAA, 0x01, 0xC9, 0x21, 0x8D],
            }
        );
    }
}
