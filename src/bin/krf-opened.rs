use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Opts {
    session: String,
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> krf::Result<()> {
    let opts = Opts::from_args();
    krf::opened_file(&opts.session, &opts.file)?;

    Ok(())
}
