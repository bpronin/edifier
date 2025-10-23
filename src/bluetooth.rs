use std::mem::zeroed;
use windows::Win32::Devices::Bluetooth::{
    BluetoothEnumerateInstalledServices, BluetoothFindDeviceClose, BluetoothFindFirstDevice, BluetoothFindFirstRadio,
    BluetoothFindNextDevice, BluetoothFindNextRadio, BluetoothFindRadioClose,
    AF_BTH, BLUETOOTH_DEVICE_INFO, BLUETOOTH_DEVICE_SEARCH_PARAMS,
    BLUETOOTH_FIND_RADIO_PARAMS, BTHPROTO_RFCOMM, SOCKADDR_BTH,
};
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE};
use windows::Win32::Networking::WinSock::{
    closesocket, connect, recv, send, socket, WSACleanup, WSAGetLastError,
    WSAStartup, INVALID_SOCKET, SEND_RECV_FLAGS, SOCKADDR, SOCKET, SOCKET_ERROR, SOCK_STREAM, WSADATA,
};
use windows_core::GUID;

pub(crate) fn open_socket(spp_uuid: GUID) -> Result<SOCKET, String> {
    unsafe {
        let mut wsa_data: WSADATA = zeroed();
        let startup_result = WSAStartup(0x202, &mut wsa_data); /* 0x202 = MAKEWORD(2,2) */
        if startup_result != 0 {
            return Err(format!("WSAStartup failed: {}", startup_result));
        }

        let socket = socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32)
            .map_err(|e| e.to_string())?;
        if socket == INVALID_SOCKET {
            return Err("Invalid socket".into());
        }

        let mut address: SOCKADDR_BTH = zeroed();
        address.addressFamily = AF_BTH;
        address.btAddr = find_device_address(spp_uuid)?;
        address.serviceClassId = spp_uuid;

        let connect_result = connect(
            socket,
            &address as *const SOCKADDR_BTH as *const SOCKADDR,
            size_of::<SOCKADDR_BTH>() as i32,
        );
        if connect_result == SOCKET_ERROR {
            return Err("Unable to connect to BT device".into());
        }

        Ok(socket)
    }
}

pub(crate) fn close_socket(sock: SOCKET) {
    unsafe {
        closesocket(sock);
        WSACleanup();
    }
}

pub(crate) fn read_socket(socket: SOCKET) -> Result<Vec<u8>, String> {
    unsafe {
        let mut buffer = [0u8; 256];
        let bytes_read = recv(socket, &mut buffer, SEND_RECV_FLAGS(0));
        if bytes_read == SOCKET_ERROR {
            return Err(format!("Read error: {:?}", WSAGetLastError()));
        }

        let result = buffer[..bytes_read as usize].to_vec();
        Ok(result)
    }
}

pub(crate) fn write_socket(socket: SOCKET, data: &[u8]) -> Result<(), String> {
    unsafe {
        let bytes_sent = send(socket, data, SEND_RECV_FLAGS(0));
        if bytes_sent == SOCKET_ERROR {
            return Err(format!("Write error: {:?}", WSAGetLastError()));
        }
    }

    Ok(())
}

fn find_device_address(spp_guid: GUID) -> Result<u64, String> {
    unsafe {
        let find_radio_params = BLUETOOTH_FIND_RADIO_PARAMS {
            dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
        };
        let mut radio_handle = HANDLE::default();
        let find_radio_handle = BluetoothFindFirstRadio(&find_radio_params, &mut radio_handle)
            .map_err(|e| e.to_string())?;
        if find_radio_handle.is_invalid() {
            return Err("No bluetooth-radio".into());
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
                    if has_spp_service(radio_handle, &mut device_info, spp_guid) {
                        return Ok(device_info.Address.Anonymous.ullLong);
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

    Err("Not found".into())
}

fn has_spp_service(
    radio_handle: HANDLE,
    device_info: &BLUETOOTH_DEVICE_INFO,
    spp_guid: GUID,
) -> bool {
    let mut guids = [GUID::default(); 10];
    let mut guids_count = guids.len() as u32;
    unsafe {
        let result = BluetoothEnumerateInstalledServices(
            radio_handle.into(),
            device_info,
            &mut guids_count,
            guids.as_mut_ptr().into(),
        );

        if result == ERROR_SUCCESS.0 {
            guids[..guids_count as usize].iter().any(|g| *g == spp_guid)
        } else {
            false
        }
    }
}
