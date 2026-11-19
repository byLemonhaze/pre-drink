#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--mcp") {
        if let Err(error) = pre_drink::mcp::run() {
            eprintln!("{error}");
            std::process::exit(1);
        }
    } else {
        pre_drink::run();
    }
}
