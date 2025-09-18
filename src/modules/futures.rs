use std::time::Duration;

use futures::prelude::*;
use parking_lot::Mutex;
use tokio::runtime::{Builder, Runtime};

use crate::modules::Module;

static TOKIO_RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);

pub struct FuturesModule {}

impl FuturesModule {
    pub fn new() -> Self {
        Self {}
    }

    pub fn spawn_future<F>(f: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // TODO store remote_handle in a list, clear() on unload()

        let tokio_runtime = TOKIO_RUNTIME.lock();

        let rt = tokio_runtime.as_ref().expect("spawn_future: no runtime?");
        rt.spawn(f);
    }

    pub fn block_future<T, F>(f: F) -> T
    where
        F: Future<Output = T>,
    {
        let mut tokio_runtime = TOKIO_RUNTIME.lock();

        let rt = tokio_runtime.as_mut().expect("block_future: no runtime?");
        rt.block_on(f)
    }
}

impl Module for FuturesModule {
    fn load(&mut self) {
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();

        let mut tokio_runtime = TOKIO_RUNTIME.lock();
        *tokio_runtime = Some(rt);
    }

    fn unload(&mut self) {
        let mut tokio_runtime = TOKIO_RUNTIME.lock();

        if let Some(rt) = tokio_runtime.take() {
            rt.shutdown_timeout(Duration::from_millis(256));
        }
    }
}
