# Rust and Async

## Authors
- Billy Jaffray

## Overview and Goal
Rust does not, by default, provide and aysnchronous runtime.
As outlined in the [Rust Programming Language Book](https://doc.rust-lang.org/book/title-page.html), trying to mark the `main` function as `async` results in a compiler error:
```rust
error[E0752]: `main` function is not allowed to be `async`
 --> src/main.rs:6:1
  |
6 | async fn main() {
  | ^^^^^^^^^^^^^^^ `main` function is not allowed to be `async`
```

In general, asynchronous programming is useful for tasks that are I/O bound or long-running - if a program is run synchronously, yet requires waiting for some external factor, the CPU will just spin and waste cycles. We may create a new thread to run this I/O bound / long-running task, however this is not always practical. Consider a system that must load 100+ videos at a time from a database, the overhead of 100 OS-backed threads will be too steep and tank performance. If we instead mark a block as `async` and `.await` a response, our thread yields to the runtime to run other `async` tasks. The runtime executor can then awake tasks when they are able to progress, all on the same thread. So, why doesn't Rust provide an `async` runtime by default?   

The goal of this project is to become familiar with `async` in Rust, understand the tradeoffs that were made during design, and implement an asynchronous runtime.
The goal of this README is for me to flesh out my understanding of `async` in Rust and demonstrate the runtime I create; this is not a holistic write-up, several parts of the book and documentation will go unsaid.

## Does Async Exist?
In Rust, asynchronous programming is achieved through the use of the `Future` trait (the documentation can be found [here](https://doc.rust-lang.org/std/future/trait.Future.html)):
```rust
pub trait Future {
    type Output;

    // Required method
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

The `Future` trait implements the method `poll` that is intended to be run by the executor. At a high level, the executor will call `poll` on its tasks, each of which will return either `Pending` or `Ready` with the result value:
```rust
pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

If a task returns `Pending`, then it yields to the executor to check on other tasks etc. If a task returns `Ready(Self::Output)`, then this task does not yield, it instead continues to run from where it left off with the output that it was waiting for. This is key to understand `async` blocks and `async` functions in `Rust`; futures are lazy and do not do anything unless polled.   

Consider the following async function:
```rust
async fn print_hello() {
  println!("hello");
}
```

Marking a funciton with `async` implies that the function will return a future. In this case, the following non-`async` function is essentially equivalent to the compiler:
```rust
fn print_hello() -> impl Future<Output = ()> {
  async move { println!("hello") }
}
```

Recalling that futures are lazy, this explains why we may not mark the `main` function as `async`. Consider the following:
```rust
async fn main() {
  println!("Hello, world!")
}
```

If this was allowed, then this is equivalent to:
```rust
fn main() -> impl Future<Output = ()> {
  async move { println!("Hello, world!") }
}
```

Since futures are lazy and do nothing unless polled by a runtime executor, then this process would exit without running any of its internal code! While this is technically fine, it would almost certainly be a bug - who wants a process that does nothing? Hence, Rust does not give you the option to do so. Furthermore, Rust does not include a runtime in the standard library as every situation where an runtime executor is required is different and providing a blanket solution would result in suboptimal performance in the majority of cases. For example, as explained in the book [here](https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html), *a high-throughput web server with many CPU cores and a large amount of RAM has very different needs than a microcontroller with a single core, a small amount of RAM, and no heap allocation ability*.

Of course, if you want to write runtime agnostic `async` functions, blocks and libraries then you may do so. However, without either using a third-party runtime or creating your own, you cannot call `.await` on a future from the `main` function due to `async` being contagious. Hence, `async` doesn't really exist in vanilla Rust.

## What if `main` Could Be Async?
Let's assume that we could mark the `main` function as `async`. i.e. we may write something like:
```rust
use std::time::Duration;

async fn main() {
  wait_then_print_hello().await;
}

async fn wait_then_print_hello() {
  sleep(Duration::from_secs(2)).await;
  println!("hello");
}
```
Upon running the above code, instead of getting a compile error, the process waits for two seconds and then prints `hello`. At this point, we have to ask ourselves "*what is the purpose of asynchronous programming?*". Recall, that asynchronous programming is useful for tasks that are I/O bound or long-running. In the above program, we don't really need to use `async` - there is nothing technically wrong with the CPU spinning and wasting cylces since it only has one job to do. As a better example, consider someone who lives in the middle of Australia and wants to load an image. Unfortunately, not only does our subject live in the middle of the dessert, but there are no servers near him to respond to his request with the image. Our subject wants to send two identical requests to the two closest servers, one on the East coast and one on the West coast. Both responses should contain the same information, hence he may just look at whichever respponse arrives first and discard the second - this is a perfect use case for `async`! We want to do something like the following:
```rust
async fn main() {
  let request_one = get_image_from("https://url_one.com.au").await;
  let request_two = get_image_from("https://url_two.com.au").await;

  let image = whatever_returns_first(request_one, request_two);
  render(image);
}

async fn get_image_from(url: &str) {
  let image = get("https://url_one.com.au").await;
  image
}
```
Ignoring the fact that this code will not compile for several reasons, the point of this example is that we called the function: `whatever_returns_first(request_one, request_two)`. Unsurprisingly, this function doesn't exist. Furthermore, without an asynchronous executor, this function cannot exist - but why?

