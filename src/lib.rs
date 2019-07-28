//! Library for stuff
use std::{
    borrow::Cow,
    cmp,
    collections::HashMap,
    env,
    fs::{self, File},
    io::{prelude::*, BufReader, BufWriter},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use structopt::StructOpt;

pub mod util;

const ME: &str = "kak-recent-files";
const MAX_INITIAL_LOAD: u32 = 10000;

#[derive(StructOpt, Debug)]
pub struct CommonOps {
    #[structopt(long = "session")]
    session: String,
    #[structopt(long = "use-temp", parse(try_from_str = "bool_value_names"))]
    use_temp_storage: bool,
    #[structopt(long = "temp-storage")]
    temp_storage: Option<String>,
}

fn bool_value_names(v: &str) -> Result<bool> {
    match v {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("unknown boolean value {}", other).into()),
    }
}

#[derive(Debug)]
enum StorageState {
    Temp,
    Permanent(PathBuf),
}

impl StorageState {
    fn new(ops: &CommonOps) -> Result<Self> {
        let res = if ops.use_temp_storage {
            StorageState::Temp
        } else {
            StorageState::Permanent(session_file(&ops.session)?)
        };
        Ok(res)
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn session_file(session: &str) -> Result<PathBuf> {
    let mut dir = dirs::data_dir().ok_or("user has no data directory")?;

    dir.push(ME);
    fs::create_dir_all(&dir)?;
    let mut file = dir;
    file.push(session);
    Ok(file)
}

fn initial_session_population() -> Result<String> {
    let mut files = Vec::new();

    let mut num_loaded = 0;
    for item in ignore::Walk::new(env::current_dir()?) {
        if num_loaded >= MAX_INITIAL_LOAD {
            break;
        }
        num_loaded += 1;
        let item = match item {
            Ok(v) => v,
            Err(e) => {
                eprintln!("warning: error walking source dir: {}", e);
                continue;
            }
        };
        let metadata = item.metadata().unwrap();
        if !metadata.is_file() {
            continue;
        }
        files.push((item.into_path(), metadata.modified()?));
    }

    files.sort_by_key(|&(_, time)| cmp::Reverse(time));

    let mut res = String::new();
    for (file, _) in files {
        res.push_str(file.to_str().unwrap());
        res.push('\n');
    }

    Ok(res)
}

fn export_temp_session_storage(value: &str) {
    println!(
        "set-option global krf_temp_storage {}",
        util::editor_quote(value)
    );
}

fn load_session_storage<'a>(
    state: &StorageState,
    ops: &'a CommonOps,
    will_save: bool,
) -> Result<Cow<'a, str>> {
    let res = match state {
        StorageState::Temp => match &ops.temp_storage {
            Some(storage) if !storage.trim().is_empty() => Cow::from(&**storage),
            _ => {
                // no initial population for temp storage.
                // let buf = initial_session_population()?;
                // if !will_save {
                //     export_temp_session_storage(&buf);
                // }
                // Cow::from(buf)
                Cow::from("")
            }
        },
        StorageState::Permanent(session_file_path) => {
            if session_file_path.exists() {
                let buf = fs::read_to_string(&session_file_path)?;
                if !buf.trim().is_empty() {
                    return Ok(Cow::from(buf));
                }
            }
            let buf = initial_session_population()?;
            if !will_save {
                save_session_storage(state, &buf)?;
            }
            Cow::from(buf)
        }
    };
    Ok(res)
}

fn save_session_storage(state: &StorageState, buf: &str) -> Result<()> {
    match state {
        StorageState::Temp => export_temp_session_storage(buf),
        StorageState::Permanent(session_file_path) => {
            fs::write(&session_file_path, &buf)?;
        }
    }
    Ok(())
}

fn add_file_to_buffer(file_path: &str, buf: &str) -> String {
    let mut finished = String::with_capacity(buf.len() + file_path.len() + 1);
    finished.push_str(&file_path);
    finished.push('\n');
    for line in buf.lines() {
        if line != file_path {
            finished.push_str(line);
            finished.push('\n');
        }
    }
    finished
}

pub fn opened_file(ops: &CommonOps, opened: &str) -> Result<()> {
    let storage = StorageState::new(ops)?;
    let buf = load_session_storage(&storage, ops, true)?;

    let finished = add_file_to_buffer(opened, &buf);

    save_session_storage(&storage, &finished)?;

    Ok(())
}

pub fn remove_file(ops: &CommonOps, file: &str) -> Result<()> {
    let storage = StorageState::new(ops)?;
    let buf = load_session_storage(&storage, ops, true)?;

    let file_path = env::current_dir()?
        .join(file)
        .into_os_string()
        .into_string()
        .unwrap();
    let mut finished = String::with_capacity(buf.len());
    for line in buf.lines() {
        if line != file_path && line != file {
            finished.push_str(line);
            finished.push('\n');
        }
    }

    save_session_storage(&storage, &finished)?;

    Ok(())
}

pub fn reset_storage(ops: &CommonOps) -> Result<()> {
    let storage = StorageState::new(ops)?;
    save_session_storage(&storage, "")?;
    Ok(())
}

struct RecentFiles<'a> {
    ordered_names: Vec<&'a str>,
    files: HashMap<&'a str, &'a str>,
}

impl<'a> RecentFiles<'a> {
    fn from_file(buf: &'a str, excluding: &str) -> Result<Self> {
        let mut ordered_names = Vec::new();
        let mut files = HashMap::new();
        let cd = env::current_dir()?;
        for line in buf.lines() {
            if line == excluding {
                continue;
            }
            let name = match Path::new(line).strip_prefix(&cd) {
                Ok(v) => v.to_str().unwrap(),
                Err(_) => line,
            };
            ordered_names.push(name);
            files.insert(name, line);
        }
        Ok(RecentFiles {
            ordered_names,
            files,
        })
    }

    fn path_from_name(&self, name: &str) -> &'a str {
        match self.files.get(name) {
            Some(v) => v,
            None => panic!("found name '{}' which was not expected", name),
        }
    }
}

fn execute_rofi<'a>(files: &RecentFiles<'a>, menu_command: &str) -> Result<Option<&'a str>> {
    let parts = shellwords::split(menu_command)
        .map_err(|_| format!("invalid configured menu command: {}", menu_command))?;
    let mut child = Command::new(&parts[0])
        .args(&parts[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut child_stdin = BufWriter::new(child.stdin.take().unwrap());
    for name in &files.ordered_names {
        write!(child_stdin, "{}\n", name)?;
    }
    child_stdin.flush()?;
    drop(child_stdin);

    let mut child_stdout = child.stdout.take().unwrap();
    let name = {
        let mut buf = String::new();
        child_stdout.read_to_string(&mut buf)?;
        buf
    };
    let name = name.trim_end_matches(&['\r', '\n'][..]);
    if name.is_empty() {
        Ok(None)
    } else {
        Ok(Some(files.path_from_name(name)))
    }
}

pub fn ask_for_path_to_open(
    ops: &CommonOps,
    menu_command: &str,
    exclude_file: &str,
) -> Result<Option<String>> {
    let storage = StorageState::new(ops)?;
    let buf = load_session_storage(&storage, ops, false)?;

    let files = RecentFiles::from_file(&buf, &exclude_file)?;
    let path = execute_rofi(&files, &menu_command)?;

    Ok(path.map(ToOwned::to_owned))
}

pub fn most_recent_file_if_exists(ops: &CommonOps) -> Result<Option<String>> {
    if ops.use_temp_storage {
        return Ok(None);
    }
    let session_file_path = session_file(&ops.session)?;
    if !session_file_path.exists() {
        return Ok(None);
    }

    let mut file = BufReader::new(File::open(&session_file_path)?);
    let mut path_to_open = String::new();
    let num_read = file.read_line(&mut path_to_open)?;

    let path_to_open = path_to_open.trim_end_matches(&['\r', '\n'][..]);

    if num_read == 0 || path_to_open.is_empty() {
        Ok(None)
    } else {
        Ok(Some(path_to_open.to_owned()))
    }
}
