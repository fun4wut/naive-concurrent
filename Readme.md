# naive-web服务器
只是返回了HTTP报文，没有什么特别的地方
采用多线程和异步两种方式构建。

通过wrk进行压力测试，8线程，10000并发，持续10秒，结果如下

| CPU型号                       | 线程池（CPU线程数*2） | 单线程异步 | 阻塞调用 |
| ----------------------------- | --------------------- | ---------- | -------- |
| i7-7700HQ（2.8GHZ 四核8线程） | 3090                  | 3271       | 40       |
| 锐龙                          |                       |            |          |

