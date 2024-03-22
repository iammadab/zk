use std::cell::RefCell;
use std::time::Instant;

type TimedUnit = (&'static str, Instant);

thread_local! {
    pub static BLOCKS: RefCell<Vec<TimedUnit>> = RefCell::new(vec![]);
}

/// Starts a timer and stores the timer description
#[macro_export]
macro_rules! start_timer {
    ($str:literal) => {
        $crate::BLOCKS.with(|blocks| {
            println!("");
            println!("{} (begin)", $str);
            blocks.borrow_mut().push(($str, std::time::Instant::now()))
        });
    }
}

/// End the timer and print the elapsed time
#[macro_export]
macro_rules! end_timer {
    () => {
        $crate::BLOCKS.with(|blocks| {
            let (description, start_time) = blocks.borrow_mut().pop().unwrap();
            println!("{} (end): {:?}", description, start_time.elapsed());
        });
    }
}