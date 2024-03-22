use std::cell::RefCell;

thread_local! {
    pub static DESCRIPTIONS: RefCell<Vec<&'static str>> = RefCell::new(vec![]);
}

/// Starts a timer and stores the timer description
#[macro_export]
macro_rules! start_timer {
    ($str:literal) => {
        $crate::DESCRIPTIONS.with(|descriptions| {
            descriptions.borrow_mut().push($str);
        });
    }
}

/// End the timer and print the elapsed time
#[macro_export]
macro_rules! end_timer {
    () => {
        $crate::DESCRIPTIONS.with(|descriptions| {
            println!("{}", descriptions.borrow_mut().pop().unwrap());
        });
    }
}