use crate::device::EdifierDevice;

mod device;
mod bluetooth;

fn main() -> Result<(), String> {
    let mut device = EdifierDevice::default();
    device.connect()?;

    device.write_raw(&[0xAA, 0x01, 0xC9, 0x21, 0x8D])?;

    let data = device.read_raw()?;
    let payload = &data[3..data.len() - 2];
    println!("{}", String::from_utf8_lossy(payload));

    Ok(())
}
