use async_runtime::{executor::Executor, sleep::Sleep};

fn main() {
    let mut runtime = Executor::new();

    runtime.add_task(task_one());
    runtime.add_task(task_two());

    let result = runtime.select().unwrap();
    println!("The faster task was: {}", result);
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


