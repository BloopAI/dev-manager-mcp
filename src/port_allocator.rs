use std::collections::{HashSet, VecDeque};
use std::net::TcpListener;

pub struct PortAllocator {
    next_port: u16,
    free_list: VecDeque<u16>,
    in_use: HashSet<u16>,
}

impl PortAllocator {
    pub fn new(start_port: u16) -> Self {
        Self {
            next_port: start_port,
            free_list: VecDeque::new(),
            in_use: HashSet::new(),
        }
    }

    pub fn allocate(&mut self) -> anyhow::Result<u16> {
        // Try reusing from free list first
        while let Some(port) = self.free_list.pop_front() {
            if self.is_available(port) {
                self.in_use.insert(port);
                return Ok(port);
            }
        }

        // Allocate new port sequentially
        loop {
            if self.next_port == u16::MAX {
                anyhow::bail!("Port allocation overflow - no more ports available");
            }

            let port = self.next_port;
            self.next_port += 1;

            if !self.in_use.contains(&port) && self.is_available(port) {
                self.in_use.insert(port);
                return Ok(port);
            }
        }
    }

    pub fn free(&mut self, port: u16) {
        if self.in_use.remove(&port) {
            self.free_list.push_back(port);
        }
    }

    fn is_available(&self, port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }
}
