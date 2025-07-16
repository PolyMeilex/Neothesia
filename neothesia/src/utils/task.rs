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

        #[allow(unused)]
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
