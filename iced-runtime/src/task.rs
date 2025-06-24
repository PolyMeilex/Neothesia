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
