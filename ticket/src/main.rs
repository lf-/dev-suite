use anyhow::Result;

#[derive(structopt::StructOpt)]
enum Args {
  /// Initialize the repo to use ticket
  Init,
}

#[paw::main]
fn main(args: Args) {
  if let Err(e) = match args {
    Args::Init => init(),
  } {
    eprintln!("{}", e);
    std::process::exit(1);
  }
}

fn init() -> Result<()> {
  Ok(())
}
