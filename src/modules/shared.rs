#![allow(dead_code)]

use futures::lock::Mutex as FutureMutex;
use std::{
  cell::RefCell,
  marker::Unsize,
  ops::{CoerceUnsized, DerefMut},
  rc::Rc,
  sync::{Arc, Mutex},
};

pub struct SyncShared<T: ?Sized> {
  inner: Rc<RefCell<T>>,
}

// fix for SyncShared<dyn Module>
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<SyncShared<U>> for SyncShared<T> {}

impl<T> SyncShared<T> {
  pub fn new(value: T) -> Self {
    Self {
      inner: Rc::new(RefCell::new(value)),
    }
  }
}

impl<T: ?Sized> SyncShared<T> {
  #[inline]
  pub fn lock(&mut self) -> impl DerefMut<Target = T> + '_ {
    self.inner.borrow_mut()
  }

  #[inline]
  pub fn with<F, R>(&mut self, f: F) -> R
  where
    F: FnOnce(&mut T) -> R,
  {
    let mut guard = self.lock();
    f(&mut guard)
  }
}

impl<T: ?Sized> Clone for SyncShared<T> {
  #[inline]
  fn clone(&self) -> SyncShared<T> {
    Self {
      inner: self.inner.clone(),
    }
  }
}

pub struct ThreadShared<T: ?Sized> {
  inner: Arc<Mutex<T>>,
}

impl<T> ThreadShared<T> {
  pub fn new(value: T) -> Self {
    Self {
      inner: Arc::new(Mutex::new(value)),
    }
  }

  #[inline]
  pub fn lock(&mut self) -> impl DerefMut<Target = T> + '_ {
    self.inner.lock().unwrap()
  }

  #[inline]
  pub fn with<F, R>(&mut self, f: F) -> R
  where
    F: FnOnce(&mut T) -> R,
  {
    let mut guard = self.lock();
    f(&mut guard)
  }
}

impl<T: ?Sized> Clone for ThreadShared<T> {
  #[inline]
  fn clone(&self) -> ThreadShared<T> {
    Self {
      inner: self.inner.clone(),
    }
  }
}

pub struct FutureShared<T: ?Sized> {
  inner: Arc<FutureMutex<T>>,
}

impl<T> FutureShared<T> {
  pub fn new(value: T) -> Self {
    Self {
      inner: Arc::new(FutureMutex::new(value)),
    }
  }

  #[inline]
  pub async fn lock(&mut self) -> impl DerefMut<Target = T> + '_ {
    self.inner.lock().await
  }

  #[inline]
  pub async fn with<F, R>(&mut self, f: F) -> R
  where
    F: FnOnce(&mut T) -> R,
  {
    let mut guard = self.lock().await;
    f(&mut guard)
  }
}

impl<T: ?Sized> Clone for FutureShared<T> {
  #[inline]
  fn clone(&self) -> FutureShared<T> {
    Self {
      inner: self.inner.clone(),
    }
  }
}

#[test]
fn test_shared() {
  println!("Hello, world!");

  {
    let mut shared = SyncShared::new(1);
    shared.with(|v| {
      println!("{}", v);
    });
    let v = shared.lock();
    println!("{}", {
      let a: &u8 = &v;
      a
    });
  }

  {
    #[derive(Debug)]
    struct NotClone {}

    let mut shared = ThreadShared::new(NotClone {});

    {
      let mut shared = shared.clone();
      std::thread::spawn(move || {
        shared.with(|v| {
          println!("{:?}", v);
        });
        let v = shared.lock();
        println!("{:?}", {
          let a: &NotClone = &v;
          a
        });
      });
    }

    shared.with(|v| {
      println!("{:?}", v);
    });
    let v = shared.lock();
    println!("{:?}", {
      let a: &NotClone = &v;
      a
    });
  }

  futures::executor::block_on(async {
    let mut shared = FutureShared::new(3);
    shared
      .with(|v| {
        println!("{}", v);
      })
      .await;
    let v = shared.lock().await;
    println!("{}", {
      let a: &u8 = &v;
      a
    });
  });

  trait Module {
    fn load(&mut self);
    fn unload(&mut self);
  }

  struct ModuleThing {}
  impl Module for ModuleThing {
    fn load(&mut self) {}
    fn unload(&mut self) {}
  }

  let mut list_of_sync_shareds: Vec<Rc<dyn Module>> = Vec::new();
  let mod_thing: Rc<dyn Module> = Rc::new(ModuleThing {});
  list_of_sync_shareds.push(mod_thing);

  let mut list_of_sync_shareds: Vec<SyncShared<dyn Module>> = Vec::new();
  let mod_thing: SyncShared<dyn Module> = SyncShared::new(ModuleThing {});
  list_of_sync_shareds.push(mod_thing);

  for module in list_of_sync_shareds.iter_mut() {
    module.lock().load();
  }
}
