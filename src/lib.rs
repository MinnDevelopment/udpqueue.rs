use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jobject};
use jni::JNIEnv;
use sender::Manager;
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
    let s = format!("{}:80", env.get_string(string)?.to_str()?);
    let mut addr: SocketAddr = s.parse()?;
    addr.set_port(port as u16);
    Ok(addr)
}

#[inline(always)]
fn copy_data(
    env: &JNIEnv,
    buffer: jobject,
    length: jint,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(env
        .get_direct_buffer_address(buffer.into())?
        .into_iter()
        .take(length as usize)
        .map(|b| *b)
        .collect())
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
            if let Ok(m) = boxed.lock() {
                if !m.is_destroyed() {
                    continue;
                }
            }
            break;
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

#[allow(unused)]
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
    match queue_packet(
        env,
        me,
        instance,
        key,
        address_string,
        port,
        data_buffer,
        data_length,
        None,
    ) {
        true => 1,
        false => 0,
    }
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
    let socket_v4 = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("Could not bind IPv4 UdpSocket");
    let socket_v6 = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).expect("Could not bind IPv6 UdpSocket");

    if instance != 0 {
        let handle = get_handle(instance);
        Manager::process(handle, &socket_v4, &socket_v6);
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

#[cfg(unix)]
mod unix_specific {
    use std::os::unix::io::{FromRawFd, RawFd};

    use super::*;

    #[no_mangle]
    #[allow(non_snake_case, unused)]
    pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_processWithSocket(
        env: JNIEnv,
        me: jobject,
        instance: jlong,
        socketv4: jlong,
        socketv6: jlong,
    ) {
        let socket_v6 = unsafe { UdpSocket::from_raw_fd(socketv6 as RawFd) };
        let socket_v4 = unsafe { UdpSocket::from_raw_fd(socketv4 as RawFd) };

        if instance != 0 {
            let handle = get_handle(instance);
            Manager::process(handle, &socket_v4, &socket_v6);
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
        let socket = unsafe { UdpSocket::from_raw_fd(socket_handle as RawFd) };
        if let Err(e) = socket.try_clone() {
            eprintln!(
                "Cannot use UdpSocket because cloning is not supported. Error: {}",
                e
            );
            return 0;
        }

        match queue_packet(
            env,
            me,
            instance,
            key,
            address_string,
            port,
            data_buffer,
            data_length,
            Some(socket),
        ) {
            true => 1,
            false => 0,
        }
    }
}
