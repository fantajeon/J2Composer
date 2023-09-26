use std::panic;

fn print_error_chain(file: &str, line: u32, err: &dyn std::error::Error) {
    eprintln!("{}:{} {}", file, line, err);
    let mut source = err.source();
    while let Some(e) = source {
        eprintln!("Caused by: {}", e);
        source = e.source();
    }
}

pub fn panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("------------------ PANIC -------------------");
        eprintln!("{}", panic_info);
        eprintln!("---------------------------------------");
        if let Some(location) = panic_info.location() {
            if let Some(err) = panic_info
                .payload()
                .downcast_ref::<&dyn std::error::Error>()
            {
                print_error_chain(location.file(), location.line(), err);
            } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                eprintln!("{}:{} {}", location.file(), location.line(), s);
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                eprintln!("{}:{} {}", location.file(), location.line(), s);
            } else {
                eprintln!(
                    "{}:{} Panic occurred, but the cause is unknown.",
                    location.file(),
                    location.line()
                );
            }
        } else {
            eprintln!("Panic occurred, but the location is unknown.");
        }
    }));
}
