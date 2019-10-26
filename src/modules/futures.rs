use crate::modules::Module;
use futures::prelude::*;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tokio::runtime::Runtime;

lazy_static! {
  static ref TOKIO_RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);
}

pub struct FuturesModule {}

impl FuturesModule {
  pub fn new() -> Self {
    Self {}
  }

  pub fn spawn_future<F: Future<Output = ()> + Send + 'static>(f: F) {
    let tokio_runtime = TOKIO_RUNTIME.lock();

    let rt = tokio_runtime.as_ref().expect("spawn_future: no runtime?");
    rt.spawn(f);
  }

  pub fn block_future<T, F>(f: F) -> T
  where
    F: Future<Output = T>,
  {
    let tokio_runtime = TOKIO_RUNTIME.lock();

    let rt = tokio_runtime.as_ref().expect("block_future: no runtime?");
    rt.block_on(f)
  }
}

impl Module for FuturesModule {
  fn load(&mut self) {
    let rt = Runtime::new().expect("tokio Runtime::new()");

    let mut tokio_runtime = TOKIO_RUNTIME.lock();
    *tokio_runtime = Some(rt);
  }

  fn unload(&mut self) {
    let mut tokio_runtime = TOKIO_RUNTIME.lock();

    if let Some(rt) = tokio_runtime.take() {
      rt.shutdown_now();
    }
  }
}
