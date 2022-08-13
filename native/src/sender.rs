use std::{
    collections::{HashMap, VecDeque},
    mem::ManuallyDrop,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Condvar, Mutex, MutexGuard},
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
    pub fn new(address: SocketAddr, key: Key, capacity: usize) -> Self {
        Self {
            packets: VecDeque::with_capacity(capacity),
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
    state: Mutex<QueueState>,
    condvar: Arc<Condvar>,
    interval: Duration,
}

struct QueueState {
    queues: VecDeque<Key>,
    index: HashMap<Key, Queue>,
    status: Status,
    capacity: usize,
    condvar: Arc<Condvar>,
}

impl QueueState {
    fn new(capacity: usize, condvar: Arc<Condvar>) -> Self {
        Self {
            queues: VecDeque::with_capacity(100),
            index: HashMap::with_capacity(100),
            status: Status::Running,
            condvar,
            capacity,
        }
    }

    #[inline(always)]
    fn shutdown(&mut self) {
        self.status = Status::Shutdown;
        self.condvar.notify_all();
    }

    #[inline(always)]
    fn next(&mut self) -> Option<&mut Queue> {
        self.queues.pop_front().and_then(|k| self.index.get_mut(&k))
    }

    #[inline(always)]
    fn append(&mut self, key: Key) {
        self.queues.push_back(key);
    }

    #[inline(always)]
    fn delete_queue(&mut self, key: Key) -> bool {
        self.index.remove(&key).is_some()
    }

    #[inline(always)]
    fn remaining(&self, key: Key) -> usize {
        if let Some(queue) = self.index.get(&key) {
            self.capacity.saturating_sub(queue.packets.len())
        } else {
            self.capacity
        }
    }

    #[inline(always)]
    fn enqueue_packet(
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
            let mut q = Queue::new(address, key, self.capacity);
            q.socket = socket;
            self.index.insert(key, q);
            self.queues.push_front(key); // queue should be immediately used on next iteration!
            self.index
                .get_mut(&key)
                .expect("Queue must be in index after insert call")
        };

        queue.packets.push_back(data);
        self.condvar.notify_all();
    }
}

impl Manager {
    pub fn new(capacity: usize, interval: Duration) -> Self {
        let condvar = Arc::new(Condvar::new());
        let state = Mutex::new(QueueState::new(capacity, condvar.clone()));
        Self {
            interval,
            state,
            condvar,
        }
    }

    pub fn wait_shutdown(&self) {
        let mut guard = self.state();
        while guard.status != Status::Destroyed {
            guard = self.condvar.wait(guard).unwrap();
        }
    }

    #[inline(always)]
    fn state(&self) -> MutexGuard<QueueState> {
        self.state.lock().unwrap()
    }

    #[inline(always)]
    pub fn remaining(&self, key: Key) -> usize {
        self.state().remaining(key)
    }

    #[inline(always)]
    pub fn shutdown(&self) {
        self.state().shutdown();
    }

    #[inline(always)]
    pub fn delete_queue(&self, key: Key) -> bool {
        self.state().delete_queue(key)
    }

    pub fn enqueue_packet(
        &self,
        key: Key,
        address: SocketAddr,
        data: Vec<u8>,
        socket: Option<ManuallyDrop<UdpSocket>>,
    ) -> bool {
        match self.state.lock() {
            Ok(mut state) if state.status == Status::Running => {
                state.enqueue_packet(key, address, data, socket);
                true
            }
            _ => false,
        }
    }

    pub fn process(&self, socket_v4: &UdpSocket, socket_v6: &UdpSocket, log_errors: bool) {
        loop {
            let packet;
            let due_time;
            let key;
            let address;
            let explicit_socket;

            {
                let mut state = self.state();
                if let Some(q) = state.next() {
                    key = q.key;
                    due_time = q.due_time;
                    address = q.address;
                    explicit_socket = q.socket.as_ref().and_then(|s| s.try_clone().ok());

                    if let Some(p) = q.pop() {
                        packet = p;
                    } else {
                        state.delete_queue(key);
                        continue;
                    }
                } else {
                    // Wait until either shutdown or a new packet is enqueued
                    state = self.condvar.wait(state).unwrap();
                    if state.status != Status::Running {
                        break;
                    } else {
                        continue;
                    }
                }
            }

            // Sleep without mutex lock
            sleep_until(due_time);

            let result = if let Some(socket) = explicit_socket {
                socket.send_to(&packet, address)
            } else if address.is_ipv4() {
                socket_v4.send_to(&packet, address)
            } else {
                socket_v6.send_to(&packet, address)
            };

            // Disable this using -Dudpqueue.log_errors=false in your java command line
            match result {
                Err(e) if log_errors => eprintln!("[udpqueue] Error sending packet: {}", e),
                _ => {}
            }

            let now = SystemTime::now();

            let mut state = self.state();
            if let Some(queue) = state.index.get_mut(&key) {
                // Let the queue expire if it is currently empty
                if now.duration_since(due_time).unwrap_or(Duration::ZERO) >= 2 * self.interval {
                    // If the sending took more than twice the interval, we reschedule the next packet to avoid overlap
                    // Normally, the next packet would now send immediately after this, which is undesirable.
                    queue.due_time = now + self.interval;
                } else {
                    // Otherwise just send the next packet when the interval is over
                    queue.due_time += self.interval;
                }

                state.append(key);
            }

            if state.status != Status::Running {
                break;
            }
        }

        self.state().status = Status::Destroyed;
        self.condvar.notify_all();
    }
}

#[inline(always)]
fn sleep_until(time: SystemTime) {
    if let Ok(dur) = time.duration_since(SystemTime::now()) {
        sleep(dur);
    }
}
