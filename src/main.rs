use newt::cli;

fn main() {
    cli::run().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}
