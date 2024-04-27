use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "dirbust-rs")]
#[command(version = "1.0")]
#[command(version, about = "Directory bruteforce and fuzzer. Made by @dubs3c.", long_about = None)]
pub struct Args {
    /// Domain to use
    #[arg(short, long)]
    pub domain: String,

    /// Wordlist to use
    #[arg(short, long)]
    pub wordlist: String,
}
