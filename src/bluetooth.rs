use crate::err;
#[cfg(feature = "debug")]
use crate::utils::join_hex;
use std::mem::zeroed;
use windows::Win32::Devices::Bluetooth::{
    BluetoothEnumerateInstalledServices, BluetoothFindDeviceClose, BluetoothFindFirstDevice, BluetoothFindFirstRadio,
    BluetoothFindNextDevice, BluetoothFindNextRadio, BluetoothFindRadioClose, BluetoothSetServiceState,
    AF_BTH, BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS,
    BLUETOOTH_FIND_RADIO_PARAMS, BLUETOOTH_SERVICE_DISABLE, BLUETOOTH_SERVICE_ENABLE,
    BTHPROTO_RFCOMM, BTH_ERROR_SUCCESS, SOCKADDR_BTH,
};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Networking::WinSock;
use windows::Win32::Networking::WinSock::{
    WSACleanup, WSAGetLastError, WSAStartup, INVALID_SOCKET, SEND_RECV_FLAGS, SOCKADDR, SOCKET,
    SOCKET_ERROR, SOCK_STREAM, WSADATA, WSAETIMEDOUT,
};
use windows_core::GUID;

/// Resets Bluetooth audio-related services for the device that provides the specified SPP service.
pub(crate) fn pair(spp_guid: &GUID) -> Result<(), String> {
    let (radio_handle, device_info) = find_device(spp_guid)?;

    const A2DP_SINK_UUID: GUID = GUID::from_u128(0x0000110B_0000_1000_8000_00805F9B34FB);
    const HFP_AG_UUID: GUID = GUID::from_u128(0x0000111E_0000_1000_8000_00805F9B34FB);

    for service in &[A2DP_SINK_UUID, HFP_AG_UUID] {
        unsafe {
            BluetoothSetServiceState(
                Some(radio_handle),
                &device_info,
                service,
                BLUETOOTH_SERVICE_DISABLE,
            );
            let result = BluetoothSetServiceState(
                Some(radio_handle),
                &device_info,
                service,
                BLUETOOTH_SERVICE_ENABLE,
            );
            if result != BTH_ERROR_SUCCESS {
                return err!("Bluetooth reset state failed: {result}.");
            }
        }
    }

    Ok(())
}

/// Opens an RFCOMM Bluetooth socket connection to the device that provides the specified SPP service.
pub(crate) fn connect(spp_guid: &GUID) -> Result<SOCKET, String> {
    unsafe {
        let mut wsa_data: WSADATA = zeroed();

        let startup_result = WSAStartup(0x202, &mut wsa_data); /* 0x202 = MAKEWORD(2,2) */
        if startup_result != 0 {
            return err!("WSA startup failed: ERROR ({startup_result}).");
        }

        let socket = WinSock::socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32)
            .map_err(|e| e.to_string())?;
        if socket == INVALID_SOCKET {
            return err!("Invalid socket.");
        }

        let (_radio, device_info) = find_device(spp_guid)?;
        let mut address: SOCKADDR_BTH = zeroed();
        address.addressFamily = AF_BTH;
        address.btAddr = device_info.Address.Anonymous.ullLong;
        address.serviceClassId = *spp_guid;

        let connect_result = WinSock::connect(
            socket,
            &address as *const SOCKADDR_BTH as *const SOCKADDR,
            size_of::<SOCKADDR_BTH>() as i32,
        );

        if connect_result == SOCKET_ERROR {
            let error = WSAGetLastError();
            return if error == WSAETIMEDOUT {
                err!("Unable to connect to device.")
            } else {
                err!("Failed to connect to device: {error:?}.")
            };
        }

        Ok(socket)
    }
}

/// Closes the Bluetooth socket and cleans up the WinSock session.
pub(crate) fn disconnect(socket: SOCKET) {
    unsafe {
        WinSock::closesocket(socket);
        WSACleanup();
    }
}

/// Sends raw data over the Bluetooth socket and returns the response bytes.
pub(crate) fn send(socket: SOCKET, data: &[u8]) -> Result<Vec<u8>, String> {
    #[cfg(feature = "debug")]
    println!("BTQ: [{}]", join_hex(&data, ", "));

    let result = unsafe {
        let bytes_sent = WinSock::send(socket, data, SEND_RECV_FLAGS(0));
        if bytes_sent == SOCKET_ERROR {
            let error = WSAGetLastError();
            return err!("Write error: {error:?}.",);
        }

        let mut buffer = [0u8; 256];
        let bytes_read = WinSock::recv(socket, &mut buffer, SEND_RECV_FLAGS(0));
        if bytes_read == SOCKET_ERROR {
            let error = WSAGetLastError();
            return err!("Read error: {error:?}.");
        }

        buffer[..bytes_read as usize].to_vec()
    };

    #[cfg(feature = "debug")]
    println!("BTR: [{}]", join_hex(&result, ", "));

    Ok(result)
}

/// Checks if the device has the specified service enabled.
fn device_has_service(
    radio_handle: HANDLE,
    device_info: &BLUETOOTH_DEVICE_INFO,
    spp_guid: &GUID,
) -> bool {
    let mut guids = [GUID::default(); 10];
    let mut guids_count = guids.len() as u32;

    let result = unsafe {
        BluetoothEnumerateInstalledServices(
            radio_handle.into(),
            device_info,
            &mut guids_count,
            guids.as_mut_ptr().into(),
        )
    };

    if result == BTH_ERROR_SUCCESS {
        guids[..guids_count as usize].iter().any(|g| g == spp_guid)
    } else {
        false
    }
}

/// Looks for the first device providing service with specified SPP UUID.
fn find_device(spp_guid: &GUID) -> Result<(HANDLE, BLUETOOTH_DEVICE_INFO), String> {
    let find_radio_params = BLUETOOTH_FIND_RADIO_PARAMS {
        dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
    };
    let mut radio_handle = HANDLE::default();

    unsafe {
        let find_radio_handle = BluetoothFindFirstRadio(&find_radio_params, &mut radio_handle)
            .map_err(|e| e.to_string())?;
        if find_radio_handle.is_invalid() {
            return err!("No Bluetooth radio.");
        }

        let device_search_params = BLUETOOTH_DEVICE_SEARCH_PARAMS {
            dwSize: size_of::<BLUETOOTH_DEVICE_SEARCH_PARAMS>() as u32,
            fReturnAuthenticated: true.into(),
            fReturnRemembered: true.into(),
            fReturnUnknown: true.into(),
            fReturnConnected: true.into(),
            fIssueInquiry: false.into(),
            cTimeoutMultiplier: 2,
            hRadio: radio_handle,
        };

        'radios: loop {
            let mut device_info = BLUETOOTH_DEVICE_INFO {
                dwSize: size_of::<BLUETOOTH_DEVICE_INFO>() as u32,
                ..Default::default()
            };

            let find_device_handle =
                BluetoothFindFirstDevice(&device_search_params, &mut device_info)
                    .map_err(|e| e.to_string())?;

            if !find_device_handle.is_invalid() {
                'devices: loop {
                    if device_has_service(radio_handle, &device_info, spp_guid) {
                        return Ok((radio_handle, device_info));
                    }
                    if BluetoothFindNextDevice(find_device_handle, &mut device_info).is_err() {
                        break 'devices;
                    }
                }
                BluetoothFindDeviceClose(find_device_handle).map_err(|e| e.to_string())?;
            }

            if BluetoothFindNextRadio(find_radio_handle, &mut radio_handle).is_err() {
                break 'radios;
            }
        }

        BluetoothFindRadioClose(find_radio_handle).map_err(|e| e.to_string())?;
    }

    err!("No devices found.")
}

#[cfg(test)]
mod test {
    use super::*;
    use windows_core::PCWSTR;

    const SPP_UUID: GUID = GUID::from_u128(0xEDF00000_EDFE_DFED_FEDF_EDFEDFEDFEDF);

    #[test]
    fn test_find_device() {
        let result = find_device(&SPP_UUID);
        assert!(result.is_ok());

        let (radio, device) = result.unwrap();
        assert_ne!(radio, HANDLE::default());

        let name = unsafe { PCWSTR(device.szName.as_ptr()).to_string() }.unwrap();
        println!("Name: {:?}", name);

        let address = unsafe { device.Address.Anonymous.ullLong };
        println!("Address: {:?}", address);
    }

    #[test]
    fn test_connect_device() {
        let result = connect(&SPP_UUID);
        assert!(result.is_ok());

        let socket = result.unwrap();
        assert_ne!(socket, INVALID_SOCKET);
    }

    #[test]
    fn test_reconnect_device() {
        let result = pair(&SPP_UUID);
        assert!(result.is_ok());
    }
}
