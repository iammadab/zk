/// Starts a timer and stores the timer description
#[macro_export]
macro_rules! start_timer {
    ($str:literal) => {
        let timer_description = $str;
        let start_time = std::time::Instant::now();
    }
}

/// End the timer and print the elapsed time
#[macro_export]
macro_rules! end_timer {
    () => {
        let elapsed_time = start_time.elapsed();
        println!("{}: {:?}", timer_description, elapsed_time);
    }
}