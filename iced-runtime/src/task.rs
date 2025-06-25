use std::{future::Future, pin::Pin};

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// A set of concurrent actions to be performed by the iced runtime.
///
/// A [`Task`] _may_ produce a bunch of values of type `T`.
#[allow(missing_debug_implementations)]
#[must_use = "`Task` must be returned to the runtime to take effect; normally in your `update` or `new` functions."]
pub struct Task<T> {
    stream: Option<BoxFuture<T>>,
}

impl<T> Task<T> {
    /// Creates a [`Task`] that does nothing.
    pub fn none() -> Self {
        Self { stream: None }
    }

    pub fn into_future(self) -> Option<BoxFuture<T>> {
        self.stream
    }

    /// Maps the output of a [`Task`] with the given closure.
    pub fn map<O>(self, mut f: impl FnMut(T) -> O + Send + 'static) -> Task<O>
    where
        T: Send + 'static,
        O: Send + 'static,
    {
        let Some(stream) = self.stream else {
            return Task::none();
        };

        Task::future(async move {
            let res = stream.await;
            f(res)
        })
    }

    /// Creates a new [`Task`] that runs the given [`Future`] and produces
    /// its output.
    pub fn future(future: impl Future<Output = T> + Send + 'static) -> Self
    where
        T: 'static,
    {
        Self {
            stream: Some(Box::pin(future)),
        }
    }
}

pub mod thread {
    use async_channel::Receiver;
    use std::any::Any;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::thread as sync;

    #[derive(Debug)]
    pub struct JoinHandle<T> {
        imp: sync::JoinHandle<()>,
        chan: Receiver<sync::Result<T>>,
    }

    impl<T> JoinHandle<T> {
        pub async fn join(self) -> sync::Result<T> {
            let ret = self
                .chan
                .recv()
                .await
                .map_err(|x| -> Box<dyn Any + Send + 'static> { Box::new(x) })
                .and_then(|x| x);
            let _ = self.imp.join(); // synchronize threads
            ret
        }

        pub fn thread(&self) -> &sync::Thread {
            self.imp.thread()
        }
    }

    pub fn spawn<F, T>(name: String, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let builder = std::thread::Builder::new().name(name);

        let (send, recv) = async_channel::bounded(1);
        let handle = builder
            .spawn(move || {
                let _ = send.send_blocking(catch_unwind(AssertUnwindSafe(f)));
            })
            .unwrap();

        JoinHandle {
            chan: recv,
            imp: handle,
        }
    }
}
