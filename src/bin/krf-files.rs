use structopt::StructOpt;

use krf::util::editor_quote;

#[derive(StructOpt)]
struct Opts {
    session: String,
}

fn main() -> krf::Result<()> {
    let opts = Opts::from_args();
    let path = krf::ask_for_path_to_open(&opts.session)?;

    if let Some(path) = path {
        println!("edit {}", editor_quote(path.to_str().unwrap()));
    }
    Ok(())
}
