use async_runtime::{
    runtime::Runtime, 
    sleep::Sleep
};

fn main() {
    let runtime = Runtime::new();

    runtime.spawn(async {
        println!("Task 1 starting...");
        Sleep::new(2).await;
        println!("Task 1 finishing.");
        1
    });

    runtime.spawn(async {
        Sleep::new(1).await;
        println!("Task 2 starting...");
        Sleep::new(2).await;
        println!("Task 2 finishing.");
        2
    });

    let result = runtime.select().unwrap();
    println!("The task that finished first was: {}", result);
}

