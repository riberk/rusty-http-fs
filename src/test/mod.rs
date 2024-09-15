pub mod client;
pub mod pool;
pub mod ports;
pub mod server;
pub mod test_context;
pub mod test_environment;
pub mod test_subscriber;
pub mod test_time;
pub mod value_generator;
pub mod web_server;

pub use inner::{get_free_port, start_test, test};
pub use web_server::{RunServer, TestResponse};

mod inner {
    use std::{future::Future, sync::LazyLock};

    use super::pool::Pool;
    use super::ports::{Ports, UsingPort};
    use super::test_context::TestContext;
    use super::test_environment::TestEnvironment;
    use super::test_subscriber::{LogCollector, WriteBehaviour};
    use tokio::task;
    use tracing::instrument::WithSubscriber;

    static POOL: LazyLock<Pool<TestEnvironment>> = LazyLock::new(|| {
        let available_parallelism = std::thread::available_parallelism().unwrap().get();
        let max_capacity = available_parallelism.clamp(8, 64);
        Pool::new(max_capacity, 0)
    });

    static PORTS: LazyLock<Ports> = LazyLock::new(Default::default);

    pub async fn start_test(logs: LogCollector) -> TestContext {
        POOL.acquire(TestEnvironment::make)
            .await
            .start_test(logs)
            .await
    }

    pub fn test<Fun, Fut>(f: Fun)
    where
        Fun: FnOnce(TestContext) -> Fut,
        Fut: Future<Output = ()> + 'static + Sized,
    {
        use colored::Colorize;

        let logs = LogCollector::default();
        let subscriber = logs.make_subscriber();

        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on({
                let logs = logs.clone();
                async move {
                    let ctx = start_test(logs.clone()).await;
                    let number = ctx.env().number();
                    let logs = ctx.logs().clone();

                    let fut = f(ctx);

                    let local = task::LocalSet::new();

                    let res = local
                        .run_until(async move { task::spawn_local(fut).await })
                        .await;

                    let mut has_error = false;
                    if let Err(e) = res {
                        eprintln!("Error has been occurred");
                        eprintln!("{:#?}", e);
                        has_error = true;
                    }

                    if let Some(logs) = logs.take() {
                        let write_logs = match logs.write_behaviour() {
                            WriteBehaviour::OnError => has_error,
                            WriteBehaviour::Always => true,
                            WriteBehaviour::Never => false,
                        };

                        if write_logs {
                            eprintln!("Test logs: ");
                            let number = format!("{:03}", number).bold().dimmed();
                            for log in logs.logs() {
                                eprintln!("{} {}", number, log)
                            }
                        }
                    }

                    if has_error {
                        std::panic::set_hook(Box::new(|_info| {
                            // previous panic had to be printed
                        }));

                        panic!()
                    }
                }
                .with_subscriber(subscriber)
            });
    }

    pub fn get_free_port() -> UsingPort {
        PORTS.acquire()
    }
}
