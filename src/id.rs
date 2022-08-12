use crate::{ClientError, ClientResult};
use std::collections::LinkedList;

pub trait AllocRequestId {
    fn alloc(&mut self) -> ClientResult<u16>;

    fn release(&mut self, id: u16);
}

pub struct FixRequestIdAllocator;

impl AllocRequestId for FixRequestIdAllocator {
    fn alloc(&mut self) -> ClientResult<u16> {
        Ok(0)
    }

    fn release(&mut self, _id: u16) {}
}

pub struct PooledRequestIdAllocator {
    ids: LinkedList<u16>,
}

impl Default for PooledRequestIdAllocator {
    fn default() -> Self {
        let mut ids = LinkedList::new();
        for id in 0..u16::max_value() {
            ids.push_front(id);
        }
        Self { ids }
    }
}

impl AllocRequestId for PooledRequestIdAllocator {
    fn alloc(&mut self) -> ClientResult<u16> {
        self.ids.pop_back().ok_or(ClientError::RequestIdExhausted)
    }

    fn release(&mut self, id: u16) {
        self.ids.push_back(id);
    }
}
