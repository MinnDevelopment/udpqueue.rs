use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jobject};
use jni::JNIEnv;
use sender::Manager;
// use std::alloc::dealloc;
// use std::alloc::{alloc, dealloc, Layout};
use std::net::SocketAddr;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use crate::sender::Key;

mod sender;

// static LAYOUT: Layout = Layout::new::<Mutex<Manager>>();

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
        let ptr = instance as *mut Mutex<Manager>;
        // dealloc(ptr, LAYOUT);
        drop(Box::from_raw(ptr));
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
    if instance == 0 {
        return 0;
    }

    let data: Vec<u8> = match copy_data(&env, data_buffer, data_length) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to copy data: {e}");
            return 0;
        }
    };

    let address = match parse_address(&env, address_string, port) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Invalid socket address provided: {e}");
            return 0;
        }
    };

    let mut result = 0;
    get_locked!(instance, |mut m: Locked| {
        m.enqueue_packet(key.into(), address, data);
        result = 1;
    });

    result
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
    // TODO: Explicit socket handling
    Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_queuePacket(
        env,
        me,
        instance,
        key,
        address_string,
        port,
        data_buffer,
        data_length,
    )
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
    if instance != 0 {
        let handle = get_handle(instance);
        Manager::process(handle);
    }
}

#[no_mangle]
#[allow(non_snake_case, unused)]
pub extern "system" fn Java_com_sedmelluq_discord_lavaplayer_udpqueue_natives_UdpQueueManagerLibrary_processWithSocket(
    env: JNIEnv,
    me: jobject,
    instance: jlong,
    socketv4: jlong,
    socketv6: jlong,
) {
    if instance != 0 {
        let handle = get_handle(instance);
        Manager::process(handle);
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
