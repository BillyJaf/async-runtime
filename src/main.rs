use async_runtime::{
    runtime::Runtime, 
    sleep::Sleep
};

fn main() {
    // let mut runtime = Runtime::new();

    // runtime.spawn(async {
    //     println!("Task 1 starting...");
    //     Sleep::new(2).await;
    //     println!("Task 1 finishing.");
    //     1
    // });

    // runtime.spawn(async {
    //     Sleep::new(1).await;
    //     println!("Task 2 starting...");
    //     Sleep::new(2).await;
    //     println!("Task 2 finishing.");
    //     2
    // });

    // let result = runtime.select().unwrap();
    // println!("The task that finished first was: {}", result);

    let mut runtime = Runtime::new();

    let task_one_id = runtime.spawn(async {
        println!("Task 1 starting...");
        Sleep::new(2).await;
        println!("Task 1 finishing.");
        50
    });

    let task_two_id = runtime.spawn_with_id(2, async {
        Sleep::new(1).await;
        println!("Task 2 starting...");
        Sleep::new(2).await;
        println!("Task 2 finishing.");
        100
    }).unwrap();

    let result = runtime.join();

    println!("Task {} returned: {}", task_one_id, result.get(&task_one_id).unwrap());
    println!("Task {} returned: {}", task_two_id, result.get(&task_two_id).unwrap());
}

