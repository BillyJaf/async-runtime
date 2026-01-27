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

In general, asynchronous programming is useful for tasks that are I/O bound or long-running - if a program is run synchrnously, yet requires waiting for some external factor, the CPU will just spin and waste cycles. We may create a new thread to run this I/O bound / long-running task, however this is not always practical. Consider a system that must load 100+ videos at a time from a database, the overhead of 100 OS-backed threads will be too steep and tank performance. If we instead mark a block as `async` and `.await` a response, our thread yields to the runtime to run other `async` tasks. The runtime executor can then awake tasks when they are able to progress, all on the same thread. So, why doesn't Rust procide an `async` runtime by default?   

The goal of this project is to become familiar with `async` in Rust, understand the tradeoffs that were made during design, and implement an asynchronous runtime.
The goal of this README is for me to flesh out my understanding of `async` in Rust and demonstrate the runtime I create; this is not a holistic write-up, several parts of the book and documentation will go unsaid.

## Rust
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

Since futures are lazy and do nothing unless polled by a runtime executor, then this process would exit without running any of its internal code! While this is technically fine, it would almost certainly be a bug - who wants a process that does nothing? Hence, Rust does not give you the option to do so.






