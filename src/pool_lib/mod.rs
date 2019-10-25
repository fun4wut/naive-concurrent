mod mpmc_pool;
mod naive_pool;
pub use mpmc_pool::MPMCThreadPool;
pub use naive_pool::NaiveThreadPool;
pub use scheduled_thread_pool::ScheduledThreadPool;
use std::thread::JoinHandle;
// 要传递的闭包，Send来线程间传递，'static生命周期意味着贯穿整个程序，因为不知道该线程执行多久
type Job = Box<dyn FnOnce() + Send + 'static>;
/// 传递的信息，有可能是新的任务，或是终止信息
enum Message {
    NewJob(Job),
    Terminate,
}
// JoinHandle<()>编译器无法知道大小，把他放到堆上以使用trait object
trait Worker {
    fn thread(&self) -> Box<Option<JoinHandle<()>>>;
    fn id(&self) -> usize;
}
pub trait ThreadPool: Drop {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;
    fn workers(&self) -> Vec<Box<dyn Worker>>;
}
/// 使第三方crate获得ThreadPool接口
impl ThreadPool for ScheduledThreadPool {
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.execute(f);
    }

    // 无意义的函数，确保不会调用
    fn workers(&self) -> Vec<Box<dyn Worker>> {
        vec![]
    }
}
