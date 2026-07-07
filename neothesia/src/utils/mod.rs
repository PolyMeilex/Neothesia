use std::{future::Future, pin::Pin, task::Waker};

pub use neothesia_core::utils::*;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub mod task;
pub mod window;

pub fn noop_waker_ref() -> &'static Waker {
    use std::{
        ptr::null,
        task::{RawWaker, RawWakerVTable},
    };

    unsafe fn noop_clone(_data: *const ()) -> RawWaker {
        noop_raw_waker()
    }

    unsafe fn noop(_data: *const ()) {}

    const NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

    const fn noop_raw_waker() -> RawWaker {
        RawWaker::new(null(), &NOOP_WAKER_VTABLE)
    }

    struct SyncRawWaker(RawWaker);
    unsafe impl Sync for SyncRawWaker {}

    static NOOP_WAKER_INSTANCE: SyncRawWaker = SyncRawWaker(noop_raw_waker());

    // SAFETY: `Waker` is #[repr(transparent)] over its `RawWaker`.
    unsafe { &*(&NOOP_WAKER_INSTANCE.0 as *const RawWaker as *const Waker) }
}
