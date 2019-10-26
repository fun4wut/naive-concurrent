/// 让派生类实现pool trait
/// 不通过OO实现是因为trait object的限制
/// 由于宏作用域的关系，必须将其提到最顶部
#[macro_export]
macro_rules! impl_pool_traits {
    ($($t:ty),*) => {
        $(impl Drop for $t {
            fn drop(&mut self) {
                println!("Sending terminate message to all workers.");
                // 先发送停机message
                for _ in &mut self.workers {
                    self.sender.send(Message::Terminate).unwrap();
                }
                println!("Shutting down all workers.");
                // 等待所有worker关闭
                for worker in &mut self.workers {
                    println!("Shutting down worker {}", worker.id);
                    // 利用take将线程从worker中取出
                    if let Some(thread) = worker.thread.take() {
                        thread.join().unwrap();
                    }
                }
            }
        }
        impl ThreadPool for $t {
            fn execute<F>(&self, f: F)
                where
                    F: FnOnce() + Send + 'static,
            {
                let message = Message::NewJob(box f);
                self.sender.send(message).unwrap();
            }
        })*
    };
}

#[macro_export]
macro_rules! gen_struct {
    (worker) => {
        struct Worker {
            id: usize,
            // 里面的线程是可为空的
            thread: Option<thread::JoinHandle<()>>,
        }
    };
}
