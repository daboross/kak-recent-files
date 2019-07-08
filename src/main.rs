use std::path::Path;

use structopt::StructOpt;

use kak_recent_files::{util::editor_quote, CommonOps};

#[derive(StructOpt)]
struct Ops {
    #[structopt(flatten)]
    common: CommonOps,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "open-initial-file")]
    OpenInitialFile,
    #[structopt(name = "opened-file")]
    OpenedFile {
        file: String,
    },
    #[structopt(name = "open-menu")]
    OpenMenu {
        #[structopt(long = "from")]
        current_buffile: String,
        #[structopt(long = "cmd")]
        cmd: String,
    },
    #[structopt(name = "remove-file")]
    RemoveFile {
        file: String,
    },
    #[structopt(name = "reset")]
    Reset,
}

fn main() -> kak_recent_files::Result<()> {
    let ops = Ops::from_args();

    match ops.cmd {
        Command::OpenInitialFile => {
            let path = kak_recent_files::most_recent_file_if_exists(&ops.common)?;

            if let Some(path) = path {
                println!("edit {}", editor_quote(&path));
            }
        }
        Command::OpenMenu {
            current_buffile,
            cmd,
        } => {
            let path = kak_recent_files::ask_for_path_to_open(&ops.common, &cmd, &current_buffile)?;

            if let Some(path) = path {
                if Path::new(&path).exists() {
                    println!("edit {}", editor_quote(&path));
                } else {
                    println!("buffer {}", editor_quote(&path));
                }
            }
        }
        Command::OpenedFile { file } => {
            kak_recent_files::opened_file(&ops.common, &file)?;
        }
        Command::RemoveFile { file } => {
            kak_recent_files::remove_file(&ops.common, &file)?;
        }
        Command::Reset => {
            kak_recent_files::reset_storage(&ops.common)?;
        }
    }

    Ok(())
}
