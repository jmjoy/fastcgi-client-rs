use crate::{ClientError, ClientResult};
use std::{collections::HashSet, time::Duration};
use tokio::{
    sync::Mutex,
    time::{sleep, timeout},
};

const MAX_REQUEST_ID: u16 = u16::max_value() - 1;

pub(crate) struct RequestIdGenerator {
    id: Mutex<u16>,
    ids: Mutex<HashSet<u16>>,
    timeout: Duration,
}

impl RequestIdGenerator {
    pub(crate) fn new(timeout: Duration) -> Self {
        Self {
            id: Mutex::new(0),
            ids: Default::default(),
            timeout,
        }
    }

    pub(crate) async fn alloc(&self) -> ClientResult<u16> {
        timeout(self.timeout, self.inner_alloc())
            .await
            .map_err(|_| ClientError::RequestIdGenerateTimeout)
    }

    async fn inner_alloc(&self) -> u16 {
        let mut id = self.id.lock().await;

        loop {
            if *id >= MAX_REQUEST_ID {
                *id = 0;
            }
            *id += 1;

            let ids = self.ids.lock().await;
            if ids.contains(&id) {
                drop(ids);
                sleep(Duration::from_millis(10)).await;
            } else {
                break;
            }
        }

        *id
    }

    pub(crate) async fn release(&self, id: u16) {
        self.ids.lock().await.remove(&id);
    }
}
