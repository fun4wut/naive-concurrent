#[macro_use]
mod utils;
mod condvar_pool;
mod mpmc_pool;
mod naive_pool;
pub use mpmc_pool::MPMCThreadPool;
pub use condvar_pool::CVarThreadPool;
pub use naive_pool::NaiveThreadPool;
pub use scheduled_thread_pool::ScheduledThreadPool;

// 要传递的闭包，Send来线程间传递，'static生命周期意味着贯穿整个程序，因为不知道该线程执行多久
type Job = Box<dyn FnOnce() + Send + 'static>;
/// 传递的信息，有可能是新的任务，或是终止信息
enum Message {
    NewJob(Job),
    Terminate,
}

pub trait ThreadPool: Drop {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;
}
/// 使第三方crate获得ThreadPool接口
impl ThreadPool for ScheduledThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.execute(f);
    }
}
