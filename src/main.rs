use async_runtime::{executor::Executor, sleep::Sleep};

fn main() {
    let mut runtime = Executor::new();

    runtime.add_task(task_one(), 1).unwrap();
    runtime.add_task(task_two(), 2).unwrap();

    let result = runtime.select().unwrap();
    println!("The faster task was: {}", result);

    let mut runtime = Executor::new();

    runtime.add_task(task_one(), 1).unwrap();
    runtime.add_task(task_two(), 2).unwrap();

    let results = runtime.join();
    println!("The first task returned: {}", results.get(&1).unwrap());
    println!("The second task returned: {}", results.get(&2).unwrap());
}

async fn task_one() -> i32 {
    Sleep::new(5).await;
    println!("Hello from task one!");
    return 1;
}

async fn task_two() -> i32 {
    Sleep::new(2).await;
    println!("Hello from task two!");
    Sleep::new(4).await;
    return 2;
}


