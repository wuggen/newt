use structopt::StructOpt;
use newt::cli::Options;

fn main() {
    let options = Options::from_args();
    println!("{:#?}", options);
}
