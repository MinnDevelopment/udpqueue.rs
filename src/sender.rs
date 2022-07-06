use std::{
    collections::{HashMap, VecDeque},
    mem::ManuallyDrop,
    net::{SocketAddr, UdpSocket},
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

// TODO: Consider using constant ring buffer
struct Queue {
    pub packets: VecDeque<Vec<u8>>, // the packets in the queue
    pub due_time: SystemTime,       // when the next packet needs to be sent
    pub key: Key, // key links to the send handler (so that each sender has its own queue of packets)
    pub address: SocketAddr, // the remote address to send our packets to
    pub socket: Option<ManuallyDrop<UdpSocket>>, // the socket to send our packets on
}

impl Queue {
    pub fn new(address: SocketAddr, key: Key) -> Self {
        Self {
            packets: VecDeque::with_capacity(20), // 400ms buffer
            due_time: SystemTime::now(),
            key,
            address,
            socket: None,
        }
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<Vec<u8>> {
        self.packets.pop_front()
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq)]
pub enum Status {
    Running,
    Shutdown,
    Destroyed,
}

pub struct Manager {
    queues: VecDeque<Key>,      // Orders the queues based on due_time
    index: HashMap<Key, Queue>, // Easy access to the map for a specific send handler
    capacity: usize,
    interval: Duration,
    status: Status,
}

impl Manager {
    pub fn new(capacity: usize, interval: Duration) -> Self {
        Self {
            capacity,
            interval,
            queues: VecDeque::with_capacity(100), // 100 connections
            index: HashMap::with_capacity(100),
            status: Status::Running,
        }
    }

    fn next(&mut self) -> Option<&mut Queue> {
        self.queues.pop_front().and_then(|k| self.index.get_mut(&k))
    }

    fn append(&mut self, key: Key) {
        self.queues.push_back(key);
    }

    pub fn enqueue_packet(
        &mut self,
        key: Key,
        address: SocketAddr,
        data: Vec<u8>,
        socket: Option<ManuallyDrop<UdpSocket>>,
    ) {
        if self.status != Status::Running {
            return;
        }

        let queue = if let Some(queue) = self.index.get_mut(&key) {
            queue
        } else {
            let mut q = Queue::new(address, key);
            q.socket = socket;
            self.index.insert(key, q);
            self.queues.push_front(key); // queue should be immediately used on next iteration!
            self.index
                .get_mut(&key)
                .expect("Queue is in index after insert call")
        };

        queue.packets.push_back(data);
    }

    pub fn shutdown(&mut self) {
        self.status = Status::Shutdown;
    }

    pub fn is_destroyed(&self) -> bool {
        self.status == Status::Destroyed
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

    pub fn process(manager: &Mutex<Manager>, socket_v4: &UdpSocket, socket_v6: &UdpSocket) {
        let mut idle = false;
        loop {
            if idle {
                // 1us time window to allow for new packages to be enqueued
                sleep(Duration::from_nanos(1000));
            }

            let packet;
            let due_time;
            let key;
            let address;
            let interval;
            let explicit_socket;

            match manager.lock() {
                Err(_) => continue,
                Ok(mut manager) => {
                    interval = manager.interval;

                    if manager.status != Status::Running {
                        manager.status = Status::Destroyed;
                        return;
                    }

                    if let Some(q) = manager.next() {
                        key = q.key;
                        due_time = q.due_time;
                        address = q.address;
                        explicit_socket = q.socket.as_ref().and_then(|s| s.try_clone().ok());
                        packet = q.pop();
                        idle = packet.is_none();
                    } else {
                        idle = true;
                        continue;
                    }
                }
            };

            if let Some(ref packet) = packet {
                sleep_until(due_time);
                let result = if let Some(socket) = explicit_socket {
                    socket.send_to(packet, &address)
                } else if address.is_ipv4() {
                    socket_v4.send_to(packet, address)
                } else {
                    socket_v6.send_to(packet, address)
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

            let now = SystemTime::now();
            if let Ok(mut manager) = manager.lock() {
                if let Some(queue) = manager.index.get_mut(&key) {
                    // Let the queue expire if it is currently empty
                    if packet.is_some() {
                        queue.due_time = now + interval;
                    }
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
