use std::cell::RefCell;
use std::time::Instant;

type TimedUnit = (&'static str, Instant);

thread_local! {
    pub static BLOCKS: RefCell<Vec<TimedUnit>> = RefCell::new(vec![]);
    pub static TAB_COUNT: RefCell<usize> = RefCell::new(0);
}

/// Starts a timer and stores the timer description
#[macro_export]
macro_rules! start_timer {
    ($str:literal) => {
        // guard should only run when PERF_LOG is set to true
        if std::env::var("PERF_LOG") == Ok(String::from("true")) {
            // create timed unit
            $crate::BLOCKS
                .with(|blocks| blocks.borrow_mut().push(($str, std::time::Instant::now())));
            $crate::TAB_COUNT.with(|tab_count| {
                // print with current tab count
                let spaces = " ".repeat(*tab_count.borrow());
                println!("");
                println!("{}{}", spaces, format!("{} (begin)", $str));
                // update tab count
                *tab_count.borrow_mut() += 1;
            })
        }
    };
}

/// End the timer and print the elapsed time
#[macro_export]
macro_rules! end_timer {
    () => {
        // guard should only run when PERF_LOG is set to true
        if std::env::var("PERF_LOG") == Ok(String::from("true")) {
            let (description, start_time) = $crate::BLOCKS.with(|blocks| {
                blocks.borrow_mut().pop().unwrap()
                // println!("{} (end): {:?}", description, start_time.elapsed());
            });
            $crate::TAB_COUNT.with(|tab_count| {
                // update the tab count
                *tab_count.borrow_mut() -= 1;
                // print with current tab count
                let spaces = " ".repeat(*tab_count.borrow());
                println!(
                    "{}{}",
                    spaces,
                    format!("{} (end): {:?}", description, start_time.elapsed())
                );
                println!("");
            })
        }
    };
}
