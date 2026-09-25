mod memory_limit;
#[global_allocator]
static ALLOCATOR: memory_limit::Limited = memory_limit::Limited;

fn main() {
    makepad_widgets::makepad_platform::makepad_error_log::set_log_handler(
        |_, _, _, _, _, message, _| eprintln!("{message}")
    );
    let result = std::env::args_os().nth(1).ok_or_else(|| "usage: app-validator <bundle>".to_string())
        .and_then(|bundle| octosense_app_validator::validate_native(std::path::Path::new(&bundle)));
    match result {
        Ok(report) => println!("{}", serde_json::to_string(&report).unwrap()),
        Err(error) => {
            println!("{}", octosense_app_hub::runtime::failure_json(&error));
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
