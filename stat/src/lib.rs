thread_local! {
    static DESCRIPTIONS: Vec<&'static str> = vec![];
}

/// Starts a timer and stores the timer description
#[macro_export]
macro_rules! start_timer {
    ($str:literal) => {
        DESCRIPTIONS.push(&str);
    }
}

/// End the timer and print the elapsed time
#[macro_export]
macro_rules! end_timer {
    () => {
        println!("{}", DESCRIPTIONS[0]);
    }
}