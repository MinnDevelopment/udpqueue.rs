use jni::objects::{JClass, JString, JValue};
use jni::sys::{jboolean, jint, jlong, jobject};
use jni::JNIEnv;
use sender::Manager;
use std::mem::ManuallyDrop;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::{Mutex, MutexGuard};
use std::thread::sleep;
use std::time::Duration;

use crate::sender::Key;

mod sender;

type Handle = &'static Mutex<Manager>;
type Locked<'a> = MutexGuard<'a, Manager>;

macro_rules! get_locked {
    ($instance:expr, $code:expr) => {
        let handle = get_handle($instance);
        if let Ok(m) = handle.lock() {
            $code(m)
        }
    };
}

#[inline]
fn get_handle(instance: jlong) -> Handle {
    unsafe { &*(instance as *mut Mutex<Manager>) }
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
fn copy_data(env: &JNIEnv, buffer: jobject, length: jint) -> Result<Vec<u8>, jni::errors::Error> {
    let length = length as usize;
    let mut buf = vec![0; length];
    let slice = env.get_direct_buffer_address(buffer.into())?;
    buf.copy_from_slice(&slice[..length]);
    Ok(buf)
}

/// Wrapper for System.getProperty(String): String?
#[inline]
fn get_property(env: &JNIEnv, name: &str) -> Option<String> {
    let class = match env.find_class("java/lang/System") {
        Ok(c) => c,
        Err(_) => return None,
    };

    let args = match env.new_string(name) {
        Ok(string) => JValue::Object(string.into()),
        Err(_) => return None,
    };

    match env.call_static_method(
        class,
        "getProperty",
        "(Ljava/lang/String;)Ljava/lang/String;",
        &[args],
    ) {
        Ok(JValue::Object(obj)) => env
            .get_string(JString::from(obj))
            .ok()
            .and_then(|s| s.to_str().map(|s| s.to_string()).ok()),
        _ => None,
    }
}

/// Whether to log send errors, default true
/// Configured using -Dudpqueue.log_errors=<bool>
#[inline]
fn is_log_errors(env: &JNIEnv) -> bool {
    get_property(env, "udpqueue.log_errors").unwrap_or("true".to_string()) == "true"
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
        let b = Box::new(Mutex::new(Manager::new(queue_buffer_capacity, interval)));
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

    get_locked!(instance, |mut m: Locked| {
        m.shutdown();
    });

    unsafe {
        let boxed = Box::from_raw(instance as *mut Mutex<Manager>);

        // Wait for the manager to finish
        loop {
            sleep(Duration::from_millis(1));
            match boxed.lock() {
                Ok(m) if !m.is_destroyed() => continue,
                _ => break,
            };
        }

        drop(boxed);
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
    let mut remaining = 0;
    get_locked!(instance, |m: Locked| {
        let key: Key = key.into();
        remaining = m.remaining(key);
    });

    remaining as jint
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
    socket: Option<ManuallyDrop<UdpSocket>>,
) -> bool {
    if instance == 0 {
        return false;
    }

    let data: Vec<u8> = match copy_data(&env, data_buffer, data_length) {
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

    let mut result = false;
    get_locked!(instance, |mut m: Locked| {
        m.enqueue_packet(key.into(), address, data, socket);
        result = true;
    });

    result
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

    let mut result = 0;
    get_locked!(instance, |mut m: Locked| {
        m.delete_queue(key.into());
        result = 1;
    });

    result
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_process(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
) {
    let socket_v4 =
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("Could not bind IPv4 UdpSocket");
    let socket_v6 =
        UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).expect("Could not bind IPv6 UdpSocket");

    let log_errors = is_log_errors(&env);
    if instance != 0 {
        let handle = get_handle(instance);
        Manager::process(handle, &socket_v4, &socket_v6, log_errors);
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
    let socket_v6 = unsafe { to_socket(socketv6) };
    let socket_v4 = unsafe { to_socket(socketv4) };

    let log_errors = is_log_errors(&env);
    if instance != 0 {
        let handle = get_handle(instance);
        Manager::process(handle, &socket_v4, &socket_v6, log_errors);
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
    let socket = unsafe { to_socket(socket_handle) };
    if let Err(e) = socket.try_clone() {
        eprintln!(
            "Cannot use UdpSocket because cloning is not supported. Error: {}",
            e
        );
        return 0;
    }

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
use fallback::to_socket;
#[cfg(unix)]
use unix_specific::to_socket;
#[cfg(windows)]
use windows_specific::to_socket;

#[cfg(unix)]
mod unix_specific {
    use super::*;
    use std::os::unix::io::{FromRawFd, RawFd};

    #[inline(always)]
    pub unsafe fn to_socket(handle: jlong) -> ManuallyDrop<UdpSocket> {
        ManuallyDrop::new(UdpSocket::from_raw_fd(handle as RawFd))
    }
}

#[cfg(windows)]
mod windows_specific {
    use super::*;
    use std::os::windows::io::{FromRawSocket, RawSocket};

    #[inline(always)]
    pub unsafe fn to_socket(handle: jlong) -> ManuallyDrop<UdpSocket> {
        ManuallyDrop::new(UdpSocket::from_raw_socket(handle as RawSocket))
    }
}

#[cfg(not(any(unix, windows)))]
mod fallback {
    use super::*;

    #[inline(always)]
    pub unsafe fn to_socket(handle: jlong) -> ManuallyDrop<UdpSocket> {
        panic!("Cannot convert UdpSocket handle for this platform");
    }
}
