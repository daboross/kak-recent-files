//! Library for stuff
use std::{
    collections::HashMap,
    env, fs,
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub mod util;

const ME: &str = "krf";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn session_file(session: &str) -> Result<PathBuf> {
    let mut dir = dirs::data_dir().ok_or("user has no data directory")?;

    dir.push(ME);
    fs::create_dir_all(&dir)?;
    let mut file = dir;
    file.push(session);
    Ok(file)
}

pub fn initial_session_population() -> Result<String> {
    let mut files = Vec::new();

    for item in ignore::Walk::new(env::current_dir()?) {
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

    eprintln!("found files: {:?}", files);

    files.sort_by_key(|&(_, time)| time);

    let mut res = String::new();
    for (file, _) in files {
        res.push_str(file.to_str().unwrap());
        res.push('\n');
    }

    Ok(res)
}

pub fn opened_file(session: &str, opened: &Path) -> Result<()> {
    let session_file_path = session_file(session)?;
    let buf = if session_file_path.exists() {
        fs::read_to_string(&session_file_path)?
    } else {
        initial_session_population()?
    };
    let file_path = env::current_dir()?
        .join(opened)
        .into_os_string()
        .into_string()
        .unwrap();
    let mut finished = String::with_capacity(buf.len() + file_path.len() + 1);
    finished.push_str(&file_path);
    finished.push('\n');
    for line in buf.lines() {
        if line != file_path {
            finished.push_str(line);
            finished.push('\n');
        }
    }
    fs::write(&session_file_path, &finished)?;

    Ok(())
}

struct RecentFiles<'a> {
    ordered_names: Vec<&'a str>,
    files: HashMap<&'a str, &'a Path>,
}

impl<'a> RecentFiles<'a> {
    fn from_file(buf: &'a str) -> Result<Self> {
        let mut ordered_names = Vec::new();
        let mut files = HashMap::new();
        let cd = env::current_dir()?;
        for file in buf.lines() {
            let path = Path::new(file);
            let name = match path.strip_prefix(&cd) {
                Ok(v) => v.to_str().unwrap(),
                Err(_) => path.to_str().unwrap(),
            };
            ordered_names.push(name);
            files.insert(name, path);
        }
        Ok(RecentFiles {
            ordered_names,
            files,
        })
    }

    fn path_from_name(&self, name: &str) -> &'a Path {
        match self.files.get(name) {
            Some(v) => v,
            None => panic!("found name '{}' which was not expected", name),
        }
    }
}

fn execute_rofi<'a>(files: &RecentFiles<'a>) -> Result<Option<&'a Path>> {
    let mut child = Command::new("rofi")
        .args(&["-dmenu", "-i", "-matching", "fuzzy"])
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

pub fn ask_for_path_to_open(session: &str) -> Result<Option<PathBuf>> {
    let session_file_path = session_file(session)?;
    let buf = if session_file_path.exists() {
        fs::read_to_string(&session_file_path)?
    } else {
        let buf = initial_session_population()?;
        fs::write(&session_file_path, &buf)?;
        buf
    };
    let files = RecentFiles::from_file(&buf)?;
    let path = execute_rofi(&files)?;
    Ok(path.map(ToOwned::to_owned))
}
