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

type BluetoothDevice = (HANDLE, BLUETOOTH_DEVICE_INFO);

const WINSOCK_VERSION_2_2: u16 = 0x0202;
const A2DP_SINK_UUID: GUID = GUID::from_u128(0x0000110B_0000_1000_8000_00805F9B34FB);
const HFP_AG_UUID: GUID = GUID::from_u128(0x0000111E_0000_1000_8000_00805F9B34FB);
const AUDIO_SERVICE_UUIDS: [GUID; 2] = [A2DP_SINK_UUID, HFP_AG_UUID];

/// Resets Bluetooth audio-related services for the device that provides the specified SPP service.
pub(crate) fn pair(spp_guid: &GUID) -> Result<(), String> {
    let (radio_handle, device_info) = find_device(spp_guid)?;

    for service_guid in AUDIO_SERVICE_UUIDS {
        reset_bluetooth_service(radio_handle, &device_info, &service_guid)?;
    }

    Ok(())
}

/// Opens an RFCOMM Bluetooth socket connection to the device that provides the specified SPP service.
pub(crate) fn connect(spp_guid: &GUID) -> Result<SOCKET, String> {
    unsafe {
        startup_winsock()?;

        let socket = WinSock::socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32)
            .map_err(|e| e.to_string())?;
        if socket == INVALID_SOCKET {
            return err!("Invalid socket.");
        }

        let (_radio, device_info) = find_device(spp_guid)?;
        let address = bluetooth_socket_address(&device_info, spp_guid);

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

fn startup_winsock() -> Result<(), String> {
    unsafe {
        let mut data: WSADATA = zeroed();
        let result = WSAStartup(WINSOCK_VERSION_2_2, &mut data);
        if result != 0 {
            return err!("WSA startup failed: ERROR ({result}).");
        }
    }
    Ok(())
}

fn reset_bluetooth_service(
    radio_handle: HANDLE,
    device_info: &BLUETOOTH_DEVICE_INFO,
    service_guid: &GUID,
) -> Result<(), String> {
    unsafe {
        BluetoothSetServiceState(
            Some(radio_handle),
            device_info,
            service_guid,
            BLUETOOTH_SERVICE_DISABLE,
        );

        let result = BluetoothSetServiceState(
            Some(radio_handle),
            device_info,
            service_guid,
            BLUETOOTH_SERVICE_ENABLE,
        );

        if result != BTH_ERROR_SUCCESS {
            return err!("Bluetooth reset state failed: {result}.");
        }
    }

    Ok(())
}

fn bluetooth_socket_address(
    device_info: &BLUETOOTH_DEVICE_INFO,
    service_guid: &GUID,
) -> SOCKADDR_BTH {
    let mut address: SOCKADDR_BTH = unsafe { zeroed() };
    address.addressFamily = AF_BTH;
    address.btAddr = unsafe { device_info.Address.Anonymous.ullLong };
    address.serviceClassId = *service_guid;
    address
}

/// Checks if the device has the specified service enabled.
fn device_has_service(
    radio_handle: HANDLE,
    device_info: &BLUETOOTH_DEVICE_INFO,
    service_guid: &GUID,
) -> bool {
    let mut service_guids = [GUID::default(); 10];
    let mut service_guid_count = service_guids.len() as u32;

    let result = unsafe {
        BluetoothEnumerateInstalledServices(
            radio_handle.into(),
            device_info,
            &mut service_guid_count,
            service_guids.as_mut_ptr().into(),
        )
    };

    result == BTH_ERROR_SUCCESS
        && service_guids[..service_guid_count as usize]
            .iter()
            .any(|installed_service_guid| installed_service_guid == service_guid)
}

/// Searches for the first Bluetooth device that provides the service matching the specified UUID
fn find_device(service_guid: &GUID) -> Result<BluetoothDevice, String> {
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
                    if device_has_service(radio_handle, &device_info, service_guid) {
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
