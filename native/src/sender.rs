use std::{
    collections::{HashMap, VecDeque},
    io,
    mem::ManuallyDrop,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket},
    sync::{Arc, Condvar, Mutex, MutexGuard},
    thread::sleep,
    time::{Duration, SystemTime},
};

struct Queue {
    packets: VecDeque<Box<[u8]>>,                 // the packets in the queue
    due_time: SystemTime,                         // when the next packet needs to be sent
    key: i64, // key links to the send handler (so that each sender has its own queue of packets)
    address: SocketAddr, // the remote address to send our packets to
    socket: Option<Arc<ManuallyDrop<UdpSocket>>>, // the socket to send our packets on
}

impl Queue {
    fn new(address: SocketAddr, key: i64, capacity: usize) -> Self {
        Self {
            packets: VecDeque::with_capacity(capacity),
            due_time: SystemTime::now(),
            key,
            address,
            socket: None,
        }
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<Box<[u8]>> {
        self.packets.pop_front()
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq)]
pub(crate) enum Status {
    Running,
    Shutdown,
    Destroyed,
}

pub(crate) struct Sockets {
    pub(crate) v4: UdpSocket,
    pub(crate) v6: Option<UdpSocket>,
}

impl Sockets {
    fn bind(log_errors: bool) -> (UdpSocket, Option<UdpSocket>) {
        // IPv4 is required by discord since every voice server uses it
        let v4 = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
            .expect("[udpqueue] Could not bind IPv4 UdpSocket");

        // IPv6 is optional since discord doesn't use it really, its only here for future proofing
        let v6 = match UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)) {
            Ok(s) => Some(s),
            Err(e) if log_errors => {
                eprintln!("[udpqueue] Could not bind IPv6 UdpSocket: {e}");
                None
            }
            _ => None,
        };

        (v4, v6)
    }

    #[inline]
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        match (addr, &self.v6) {
            (SocketAddr::V4(address), _) => self.v4.send_to(buf, address),
            (SocketAddr::V6(address), Some(v6)) => v6.send_to(buf, address),
            _ => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "socket for IPv6 not bound",
            )),
        }
    }
}

pub(crate) struct Manager {
    state: Mutex<QueueState>,
    condvar: Condvar,
    interval: Duration,
}

struct QueueState {
    queues: VecDeque<i64>,
    index: HashMap<i64, Queue>,
    status: Status,
    capacity: usize,
}

struct QueueEntry {
    packet: Box<[u8]>,
    due_time: SystemTime,
    key: i64,
    address: SocketAddr,
    explicit_socket: Option<Arc<ManuallyDrop<UdpSocket>>>,
}

impl QueueState {
    fn new(capacity: usize) -> Self {
        Self {
            queues: VecDeque::with_capacity(100),
            index: HashMap::with_capacity(100),
            status: Status::Running,
            capacity,
        }
    }

    #[inline(always)]
    fn shutdown(&mut self) {
        self.status = Status::Shutdown;
    }

    #[inline(always)]
    fn next(&mut self) -> Option<&mut Queue> {
        self.queues.pop_front().and_then(|k| self.index.get_mut(&k))
    }

    #[inline(always)]
    fn append(&mut self, key: i64) {
        self.queues.push_back(key);
    }

    #[inline(always)]
    fn delete_queue(&mut self, key: i64) -> bool {
        self.index.remove(&key).is_some()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.index.values().all(|q| q.packets.is_empty())
    }

    #[inline(always)]
    fn remaining(&self, key: i64) -> usize {
        if let Some(queue) = self.index.get(&key) {
            self.capacity.saturating_sub(queue.packets.len())
        } else {
            self.capacity
        }
    }

    #[inline(always)]
    fn enqueue_packet(
        &mut self,
        key: i64,
        address: SocketAddr,
        data: Box<[u8]>,
        socket: Option<UdpSocket>,
    ) {
        let queue = self.index.entry(key).or_insert_with_key(|&k| {
            self.queues.push_front(k); // queue should be immediately used on next iteration!
            let mut q = Queue::new(address, k, self.capacity);
            q.socket = socket.map(ManuallyDrop::new).map(Arc::new);
            q
        });

        queue.packets.push_back(data);
    }
}

impl Manager {
    pub(crate) fn new(capacity: usize, interval: Duration) -> Self {
        Self {
            interval,
            state: Mutex::new(QueueState::new(capacity)),
            condvar: Condvar::new(),
        }
    }

    #[inline]
    pub(crate) fn wait_shutdown(&self) {
        let mut guard = self.state();
        while guard.status != Status::Destroyed {
            guard = self.condvar.wait(guard).unwrap();
        }
    }

    #[inline(always)]
    fn state<'a>(&'a self) -> MutexGuard<'a, QueueState> {
        self.state.lock().unwrap()
    }

    #[inline(always)]
    pub(crate) fn remaining(&self, key: i64) -> usize {
        self.state().remaining(key)
    }

    #[inline(always)]
    pub(crate) fn shutdown(&self) {
        self.state().shutdown();
        self.condvar.notify_all();
    }

    #[inline(always)]
    pub(crate) fn delete_queue(&self, key: i64) -> bool {
        self.state().delete_queue(key)
    }

    pub(crate) fn enqueue_packet(
        &self,
        key: i64,
        address: SocketAddr,
        data: Box<[u8]>,
        socket: Option<UdpSocket>,
    ) -> bool {
        match self.state.lock() {
            Ok(mut state) if state.status == Status::Running => {
                state.enqueue_packet(key, address, data, socket);
                self.condvar.notify_all();
                true
            }
            _ => false,
        }
    }

    fn get_next(&self) -> Option<QueueEntry> {
        let mut state = self.state();

        while state.status == Status::Running {
            let mut context = None;
            let entry = state.next().and_then(|queue| {
                context = Some((queue.key, queue.due_time));
                queue.pop().map(|packet| QueueEntry {
                    packet,
                    due_time: queue.due_time,
                    key: queue.key,
                    address: queue.address,
                    explicit_socket: queue.socket.clone(),
                })
            });

            if entry.is_some() {
                return entry;
            }

            match context {
                Some((key, time)) if time.elapsed().is_ok() => {
                    state.delete_queue(key);
                }
                Some((key, _)) => state.append(key),
                _ => {}
            }

            if state.is_empty() {
                state = self.condvar.wait(state).unwrap();
            }
        }

        None
    }

    #[inline(always)]
    pub(crate) fn process(&self, log_errors: bool) {
        let (v4, v6) = Sockets::bind(log_errors);
        self.process_with_sockets(log_errors, &Sockets { v4, v6 })
    }

    pub(crate) fn process_with_sockets(&self, log_errors: bool, sockets: &Sockets) {
        while let Some(entry) = self.get_next() {
            let QueueEntry {
                packet,
                due_time,
                key,
                address,
                explicit_socket,
            } = entry;

            // Sleep without mutex lock
            sleep_until(due_time);

            let result = if let Some(socket) = explicit_socket {
                socket.send_to(&packet, address)
            } else {
                sockets.send_to(&packet, address)
            };

            // Disable this using -Dudpqueue.log_errors=false in your java command line
            match result {
                Err(e) if log_errors => eprintln!("[udpqueue] Error sending packet: {e}"),
                _ => {}
            }

            let mut state = self.state();
            if let Some(queue) = state.index.get_mut(&key) {
                let now = SystemTime::now();
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
