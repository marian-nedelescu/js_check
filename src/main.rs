use clap::Parser;

mod utils;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    let file_name = args.path.as_os_str().to_str().unwrap();
    let check_comments = utils::check_no_comments_function(file_name);
    if !check_comments.result.is_empty() {
        println!("File {file_name} has functions without comments:");
        println!("\t{:?}", check_comments.result);
    }
}
