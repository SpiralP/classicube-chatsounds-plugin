pub use std::thread::*;

pub fn spawn<S, F, T>(name: S, f: F) -> JoinHandle<T>
where
  S: Into<String>,
  F: FnOnce() -> T + Send + 'static,
  T: Send + 'static,
{
  Builder::new()
    .name(name.into())
    .spawn(f)
    .expect("failed to spawn thread")
}
