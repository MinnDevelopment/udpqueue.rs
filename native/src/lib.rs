use jni::objects::{JClass, JString, JValue};
use jni::sys::{jboolean, jint, jlong, jobject};
use jni::JNIEnv;
use sender::{Manager, Sockets};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

mod sender;

#[inline]
fn get_handle(instance: jlong) -> &'static Manager {
    unsafe { &*(instance as *mut Manager) }
}

#[inline(always)]
fn parse_address(
    env: &JNIEnv,
    string: JString,
    port: jint,
) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let s = format!("{}:{}", env.get_string(string)?.to_str()?, port);
    Ok(s.parse()?)
}

#[inline(always)]
fn copy_data(env: &JNIEnv, buffer: jobject, length: jint) -> Result<Box<[u8]>, jni::errors::Error> {
    let length = length as usize;
    let slice = env.get_direct_buffer_address(buffer.into())?;
    Ok(Box::from(&slice[..length]))
}

/// Wrapper for System.getProperty(String): String?
#[inline]
fn get_property(env: &JNIEnv, name: &str) -> Option<String> {
    let class = env.find_class("java/lang/System").ok()?;
    let args = JValue::Object(env.new_string(name).ok()?.into());

    match env.call_static_method(
        class,
        "getProperty",
        "(Ljava/lang/String;)Ljava/lang/String;",
        &[args],
    ) {
        Ok(JValue::Object(obj)) => Some(
            env.get_string(JString::from(obj))
                .ok()?
                .to_str()
                .ok()?
                .to_string(),
        ),
        _ => None,
    }
}

/// Whether to log send errors, default true
/// Configured using -Dudpqueue.log_errors=<bool>
#[inline]
fn is_log_errors(env: &JNIEnv) -> bool {
    get_property(env, "udpqueue.log_errors")
        .map(|s| s == "true")
        .unwrap_or(true)
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_create(
    env: JNIEnv,
    me: jobject,
    queue_buffer_capacity: jint,
    packet_interval: jlong,
) -> jlong {
    let queue_buffer_capacity = queue_buffer_capacity as usize;
    let interval = Duration::from_nanos(packet_interval as u64);
    unsafe {
        let b = Box::new(Manager::new(queue_buffer_capacity, interval));
        Box::into_raw(b) as jlong
    }
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_destroy(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
) {
    if instance == 0 {
        return;
    }

    unsafe {
        let manager = Box::from_raw(instance as *mut Manager);
        manager.shutdown();

        // Wait for the manager to finish
        manager.wait_shutdown();

        drop(manager);
    }
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_getRemainingCapacity(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
    key: jlong,
) -> jint {
    if instance == 0 {
        return 0;
    }

    get_handle(instance).remaining(key) as jint
}

#[allow(unused, clippy::too_many_arguments)]
fn queue_packet(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
    key: jlong,
    address_string: JString,
    port: jint,
    data_buffer: jobject,
    data_length: jint,
    socket: Option<UdpSocket>,
) -> bool {
    if instance == 0 {
        return false;
    }

    let data = match copy_data(&env, data_buffer, data_length) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to copy data: {e}");
            return false;
        }
    };

    let address = match parse_address(&env, address_string, port) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Invalid socket address provided: {e}");
            return false;
        }
    };

    get_handle(instance).enqueue_packet(key, address, data, socket)
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_queuePacket(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
    key: jlong,
    address_string: JString,
    port: jint,
    data_buffer: jobject,
    data_length: jint,
) -> jboolean {
    queue_packet(
        env,
        me,
        instance,
        key,
        address_string,
        port,
        data_buffer,
        data_length,
        None,
    ) as jboolean
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_deleteQueue(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
    key: jlong,
) -> jboolean {
    if instance == 0 {
        return 0;
    }

    get_handle(instance).delete_queue(key) as jboolean
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_process(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
) {
    let log_errors = is_log_errors(&env);
    if instance != 0 {
        get_handle(instance).process(log_errors);
    }
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_pauseDemo(
    env: JNIEnv,
    me: JClass,
    length: jint,
) {
    // todo!();
}

// Explicit socket handling requires platform-specific conversions between file descriptors / handles to UdpSocket instances.
// Note: I haven't tested any of this since I have no usable java interface at the moment.

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_processWithSocket(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
    socketv4: jlong,
    socketv6: jlong,
) {
    if instance == 0 {
        return;
    }

    assert!(
        socketv4 > 0,
        "Invalid socket handle for IPv4 provided: {socketv4}"
    );

    let v4 = unsafe { to_socket(socketv4) };

    let v6 = if socketv6 > 0 {
        Some(unsafe { to_socket(socketv6) })
    } else {
        None
    };

    let sockets = Sockets { v4, v6 };

    let log_errors = is_log_errors(&env);
    get_handle(instance).process_with_sockets(log_errors, &sockets);

    // This gives up ownership of the file descriptors back to the caller, allowing them to stay open
    unsafe {
        to_fd(sockets.v4);
        if let Some(v6) = sockets.v6 {
            to_fd(v6);
        }
    }
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_queuePacketWithSocket(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
    key: jlong,
    address_string: JString,
    port: jint,
    data_buffer: jobject,
    data_length: jint,
    socket_handle: jlong,
) -> jboolean {
    if socket_handle < 1 {
        eprintln!("[udpqueue] Invalid socket handle provided for packet: {socket_handle}");
        return false as jboolean;
    }

    let socket = unsafe { to_socket(socket_handle) };

    queue_packet(
        env,
        me,
        instance,
        key,
        address_string,
        port,
        data_buffer,
        data_length,
        Some(socket),
    ) as jboolean
}

// Pick implementation for current platform, or fallback to panic

#[cfg(not(any(unix, windows)))]
use fallback::*;
#[cfg(unix)]
use unix_specific::*;
#[cfg(windows)]
use windows_specific::*;

#[cfg(not(any(unix, windows)))]
mod fallback {
    #[inline(always)]
    pub unsafe fn to_socket(handle: jlong) -> UdpSocket {
        panic!("Cannot convert UdpSocket handle for this platform");
    }

    #[inline(always)]
    pub unsafe fn to_fd(socket: UdpSocket) -> RawFd {
        panic!("Cannot convert UdpSocket handle for this platform");
    }
}

#[cfg(unix)]
mod unix_specific {
    use super::*;
    use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};

    #[inline(always)]
    pub unsafe fn to_socket(handle: jlong) -> UdpSocket {
        UdpSocket::from_raw_fd(handle as RawFd)
    }

    #[inline(always)]
    pub unsafe fn to_fd(socket: UdpSocket) -> RawFd {
        socket.into_raw_fd()
    }
}

#[cfg(windows)]
mod windows_specific {
    use super::*;
    use std::os::windows::io::{FromRawSocket, IntoRawSocket, RawSocket};

    #[inline(always)]
    pub unsafe fn to_socket(handle: jlong) -> UdpSocket {
        UdpSocket::from_raw_socket(handle as RawSocket)
    }

    #[inline(always)]
    pub unsafe fn to_fd(socket: UdpSocket) -> RawSocket {
        socket.into_raw_socket()
    }
}
