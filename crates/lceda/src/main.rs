mod cli;
mod gui;
mod i18n;

fn main() {
    match cli::run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("Error: {err:#}");
            std::process::exit(2);
        }
    }
}
