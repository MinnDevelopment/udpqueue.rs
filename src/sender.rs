use std::{
    collections::{HashMap, VecDeque},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    sync::Mutex,
    thread::sleep,
    time::{Duration, SystemTime},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key(i64);

impl From<i64> for Key {
    fn from(other: i64) -> Self {
        Key(other)
    }
}

struct Queue {
    pub packets: VecDeque<Vec<u8>>, // the packets in the queue
    pub due_time: SystemTime,       // when the next packet needs to be sent
    pub key: Key, // key links to the send handler (so that each sender has its own queue of packets)
    pub address: SocketAddr, // the remote address to send our packets to
}

impl Queue {
    pub fn new(address: SocketAddr, key: Key) -> Self {
        Self {
            packets: VecDeque::with_capacity(20), // 400ms buffer
            due_time: SystemTime::now(),
            key,
            address,
        }
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<Vec<u8>> {
        self.packets.pop_front()
    }
}

pub struct Manager {
    queues: VecDeque<Key>,      // Orders the queues based on due_time
    index: HashMap<Key, Queue>, // Easy access to the map for a specific send handler
    shutdown: bool,
    capacity: usize,
    interval: Duration,
}

impl Manager {
    pub fn new(capacity: usize, interval: Duration) -> Self {
        Self {
            capacity,
            interval,
            queues: VecDeque::with_capacity(100), // 100 connections
            index: HashMap::with_capacity(100),
            shutdown: false,
        }
    }

    fn next(&mut self) -> Option<&mut Queue> {
        self.queues.pop_front().and_then(|k| self.index.get_mut(&k))
    }

    fn append(&mut self, key: Key) {
        self.queues.push_back(key);
    }

    pub fn enqueue_packet(&mut self, key: Key, address: SocketAddr, data: Vec<u8>) {
        if self.shutdown {
            return;
        }

        let queue = if let Some(queue) = self.index.get_mut(&key) {
            queue
        } else {
            let q = Queue::new(address, key);
            self.index.insert(key, q);
            self.queues.push_front(key); // queue should be immediately used on next iteration!
            self.index
                .get_mut(&key)
                .expect("Queue is in index after insert call")
        };

        queue.packets.push_back(data);
    }

    pub fn shutdown(&mut self) {
        self.shutdown = true;
    }

    pub fn delete_queue(&mut self, key: Key) {
        self.index.remove(&key);
    }

    pub fn remaining(&self, key: Key) -> usize {
        if let Some(queue) = self.index.get(&key) {
            self.capacity.saturating_sub(queue.packets.len())
        } else {
            self.capacity
        }
    }

    pub fn process(manager: &Mutex<Manager>) {
        let socket_v4 = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let socket_v6 = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).unwrap();

        let mut idle = false;
        loop {
            if idle {
                sleep(Duration::from_millis(10));
            }

            let packet;
            let due_time;
            let key;
            let address;
            let interval;

            {
                let mut manager = match manager.lock() {
                    Ok(manager) => manager,
                    Err(_) => continue,
                };

                interval = manager.interval;

                if manager.shutdown {
                    break;
                }

                if let Some(q) = manager.next() {
                    key = q.key;
                    due_time = q.due_time;
                    address = q.address;
                    packet = q.pop();
                    idle = packet.is_none();
                } else {
                    idle = true;
                    continue;
                }
            }

            let now = SystemTime::now();
            if let Some(packet) = packet {
                sleep_until(due_time);
                // TODO: Explicit socket handling
                let result = if address.is_ipv4() {
                    socket_v4.send_to(&packet, address)
                } else {
                    socket_v6.send_to(&packet, address)
                };

                if let Err(e) = result {
                    eprintln!("Error sending packet: {}", e);
                }
            } else if due_time.elapsed().is_ok() {
                if let Ok(mut manager) = manager.lock() {
                    manager.index.remove(&key);
                    continue;
                }
            }

            if let Ok(mut manager) = manager.lock() {
                if let Some(queue) = manager.index.get_mut(&key) {
                    queue.due_time = now + interval;
                    manager.append(key);
                }
            }
        }
    }
}

#[inline(always)]
fn sleep_until(time: SystemTime) {
    if let Ok(dur) = time.duration_since(SystemTime::now()) {
        sleep(dur);
    }
}
