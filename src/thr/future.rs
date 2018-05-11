use cpu::wait_for_int;
use futures::prelude::*;
use thr::wake::WakeNop;

/// Platform future extensions.
pub trait FuturePlat: Future {
  /// Blocks the current thread until the future is resolved.
  fn trunk_wait(self) -> Result<Self::Item, Self::Error>;
}

impl<T: Future> FuturePlat for T {
  fn trunk_wait(mut self) -> Result<Self::Item, Self::Error> {
    loop {
      match poll_future(&mut self) {
        Ok(Async::Pending) => wait_for_int(),
        Ok(Async::Ready(value)) => break Ok(value),
        Err(err) => break Err(err),
      }
    }
  }
}

fn poll_future<F: Future>(fut: &mut F) -> Poll<F::Item, F::Error> {
  let waker = WakeNop::new().waker();
  let mut map = task::LocalMap::new();
  let mut cx = task::Context::without_spawn(&mut map, &waker);
  fut.poll(&mut cx)
}