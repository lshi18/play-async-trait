// use std::thread;

// use tokio::{task, time::Duration};

// #[cfg(feature = "atask")]
// use tokio::time::sleep;

// #[cfg(feature = "stask")]
// async fn run<F>(handler: F)
// where
//     F: Fn() + Copy + Send + 'static,
// {
//     for i in 0..20 {
//         task::spawn(async move {
//             println!("{i} start");
//             handler();
//             println!("{i} end");
//         });
//     }
// }

// #[cfg(feature = "stask")]
// fn sync_sleep() {
//     std::thread::sleep(Duration::from_secs(1));
// }
// // #[tokio::main(worker_threads = 1)]
// #[tokio::main]
// async fn main() {
//     #[cfg(feature = "stask")]
//     run(sync_sleep).await;

//     #[cfg(feature = "atask")]
//     {
//         let h = HahaHandler {};
//         h.handle().await;
//     }

//     thread::sleep(Duration::from_secs(3));
// }

// #[cfg(feature = "atask")]
// async fn async_sleep() {
//     sleep(Duration::from_secs(1)).await;
// }

// #[cfg(feature = "atask")]
// trait Handler {
//     async fn handle(self) -> ();
// }

// #[cfg(feature = "atask")]
// struct HahaHandler;

// #[cfg(feature = "atask")]
// impl Handler for HahaHandler {
//     async fn handle(self) -> () {
//         for i in 0..20 {
//             task::spawn(async move {
//                 println!("{i} start");
//                 async_sleep().await;
//                 println!("{i} end");
//             });
//         }
//     }
// }

use anyhow::{anyhow, Result};
use std::time::Duration;

trait Service<Request> {
    type Response;

    async fn call(&mut self, req: Request) -> Result<Self::Response>;
}

#[derive(Debug, Clone)]
struct Timeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Timeout<S> {
    fn new(inner: S, timeout: Duration) -> Self {
        Self { inner, timeout }
    }
}

impl<S, Request> Service<Request> for Timeout<S>
where
    S: Service<Request>,
{
    type Response = S::Response;

    async fn call(&mut self, req: Request) -> Result<Self::Response> {
        tokio::select! {
            _ = tokio::time::sleep(self.timeout) => {
                Err(anyhow!("TimeoutError"))
            }
            res = self.inner.call(req) => {
                res.map_err(Into::into)
            }
        }
    }
}

struct Leaf;

impl Service<Duration> for Leaf {
    type Response = Duration;

    async fn call(&mut self, req: Duration) -> Result<Self::Response> {
        tokio::time::sleep(req).await;
        Ok(req)
    }
}

#[tokio::main]
async fn main() {
    let timeout = Duration::from_secs(3);
    let mut timeout_service = Timeout::new(Leaf, timeout);
    let res = timeout_service
        .call(timeout.saturating_sub(Duration::from_secs(1)))
        .await;
    println!("Compute: {res:?}");

    let mut timeout_service = Timeout::new(Leaf, timeout);
    let res = timeout_service
        .call(timeout.saturating_add(Duration::from_secs(1)))
        .await;
    println!("Sleepy leaf: {res:?}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_compute_ok() {
        let timeout = Duration::from_secs(3);
        let mut timeout_service = Timeout::new(Leaf, timeout);
        let res = timeout_service
            .call(timeout.saturating_sub(Duration::from_secs(1)))
            .await;
        assert_eq!(res.unwrap(), Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_compute_timeout() {
        let timeout = Duration::from_secs(3);
        let mut timeout_service = Timeout::new(Leaf, timeout);
        let res = timeout_service
            .call(timeout.saturating_add(Duration::from_secs(1)))
            .await;
        assert_eq!(res.unwrap_err().to_string(), "TimeoutError");
    }
}
