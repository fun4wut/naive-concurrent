# Rust并发练手--以Web服务器为例
服务器本身只是返回了HTTP报文，没有什么特别的地方。
采用多线程和异步两种方式构建。

具体代码可在我的 [Github]() 上查看

## 性能比较

### 测试环境

> CPU：**i7-7700HQ（2.8GHZ 四核8线程）**
>
> 模拟延时：10ms
>
> 操作系统：Windows 10
>
> 工具链：beta-x86_64-pc-windows-msvc
>
> 运行程序为debug版（release版本windows无法运行）

### 测试结果

在同一局域网下通过 `wrk` 进行压力测试，8线程，持续10秒，结果如下

| 并发数 | 自制线程池 | 调库线程池 | 单线程异步 | 阻塞调用 |
| ------ | ---------- | ---------- | ---------- | -------- |
| 10     | 89         | 227        | 226        | 89       |
| 30     | 89         | 235        | 242        | 90       |
| 100    | 90         | 269        | 273        | 89       |
| 300    | 89         | 260        | 270        | 88       |
| 700    | 87         | 262        | 292        | 86       |
| 1000   | 85         | 240        | 290        | 80       |

### 测试结果

- 线程池与异步在并发量不高时差不多性能，并发量上去之后，异步模式明显更占优势
- 自制线程池（官方教学文档版）效果较差，经过调试发现实际上只有一个线程被调用了，导致性能和单进程阻塞调用差不多

### 原因分析

1. **线程池在高并发下表现不佳**

   线程池的大小是固定的（2*CPU线程数也就是16线程），在面对上千级并发时，线程数依然会不够用，唯一的办法是加钱上服务器CPU（逃

2. **异步模式的突出表现**

   服务器是一个比较典型的考虑到异步是 `非阻塞IO` 调用，在执行每句语句后会立刻让出控制权给其他调用，避免盲等，也减少了IO损耗。

3. **自制线程池性能堪忧**

   通过 [查找资料](<https://www.jianshu.com/p/f4d853c0ef1e>) 和翻看了 [其他库](<https://docs.rs/crate/threadpool/1.7.1/source/src/lib.rs>) 的源码，发现了一些自身潜在的问题：
   
   - 使用了全局 lock ，在并发系统里面，lock 如果使用不当，会造成非常严重的性能开销

## 线程池改进

### 现有的不足

自制线程池使用了 `Mutex + Channel` 的模式，通过标准库提供的 channel 进行通讯，但 channel 其实是一个 multi-producer，single-consumer 的结构，也就是我们俗称的 MPSC。但对于线程池来说，我们需要的是一个 MPMC 的 channel，也就是说，我们需要有一个队列，这个队列可以支持多个线程同时添加，同时获取任务。

虽然单独的 channel 没法支持，但如果我们给 channel 的 Receiver 套上一个 Mutex，在加上 Arc，其实就可以了。通过 Mutex 我们能保证多个线程同时只能有一个线程抢到 lock，然后从队列里面拿到数据。而加上 Arc 主要是能在多个线程共享了，下图是简化版的代码。

```rust
// 生产者消费者模型
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
// 接收者，使用了引用计数和互斥锁来保证多所有者共享和互斥访问
type Receiver = Arc<Mutex<mpsc::Receiver<Message>>>;
// 要传递的闭包，Send来线程间传递，'static生命周期意味着贯穿整个程序，因为不知道该线程执行多久
type Job = Box<dyn FnOnce() + Send + 'static>;
/// 传递的信息，有可能是新的任务，或是终止信息
enum Message {
    NewJob(Job),
    Terminate,
}
/// 线程池
pub struct ThreadPool {
    /// 工作线程
    workers: Vec<Worker>,
    /// 信息的发送者
    sender: mpsc::Sender<Message>,
}
impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        // 创建通道
        let (sender, receiver) = mpsc::channel();
        // 包装一下接收者
        let receiver = Arc::new(Mutex::new(receiver));
        Self {
            workers: (0..size)
                .map(|i| Worker::new(i, Arc::clone(&receiver)))
                .collect::<_>(),
            sender, // 发送者
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let message = Message::NewJob(Box::new(f));
        self.sender.send(message).unwrap();
    }
}
struct Worker {
    id: usize,
    // 里面的线程是可为空的
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, receiver: Receiver) -> Self {
        Self {
            id,
            thread: Some(thread::spawn(move || {
                // 不断尝试获得锁并读取message
                loop {
                    match receiver.lock().unwrap().recv().unwrap() {
					//...
                }
            })),
        }
    }
}
```

之前已经提到，使用全局的Lock非常影响性能，这也解释了为什么自制线程池实际变成了单线程。

### 其他实现方法

**To Be Continued...**

#### Condition Variable



#### Lock-free



#### 多channel



   

