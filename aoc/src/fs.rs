use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::Command,
    str,
};

use lazy_errors::{prelude::*, Result};

use crate::ident::{Day, Id, Year};

#[cfg(test)]
use crate::ident::Part;

#[cfg(test)]
use tempfile::TempDir;

const APP_SUBDIR_NAME: &str = "advent_of_code";
const LEADERBOARD_SUBDIR_NAME: &str = "personal_leaderboard_statistics";

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct Config {
    repo_dir:   RepoDir,
    data_dir:   DataDir,
    config_dir: ConfigDir,
    cache_dir:  CacheDir,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct RepoDir {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct DataDir {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ConfigDir {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct CacheDir {
    path: PathBuf,
    personal_puzzle_inputs_dir: PathBuf,
}

impl TryFrom<&Path> for RepoDir {
    type Error = Error;

    fn try_from(dir: &Path) -> Result<Self> {
        let path = from_path_if_dir(dir)?;
        Ok(Self { path })
    }
}

impl TryFrom<&Path> for DataDir {
    type Error = Error;

    fn try_from(dir: &Path) -> Result<Self> {
        let dir = from_path_if_dir(dir)?;
        Ok(Self { path: dir })
    }
}

impl AsRef<Path> for RepoDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Config {
    #[cfg(test)]
    pub fn from(
        repo_dir: RepoDir,
        data_dir: DataDir,
        config_dir: ConfigDir,
        cache_dir: CacheDir,
    ) -> Self {
        Self {
            repo_dir,
            data_dir,
            config_dir,
            cache_dir,
        }
    }

    /// Reads all required environment variables and uses defaults if missing.
    pub fn from_env_or_defaults() -> Result<Self> {
        Ok(Self {
            repo_dir:   RepoDir::from_env_or_cargo()?,
            data_dir:   DataDir::from_env()?,
            config_dir: ConfigDir::from_env()?,
            cache_dir:  CacheDir::from_env()?,
        })
    }

    pub fn save_session_cookie(&mut self, cookie: &str) -> Result<()> {
        self.config_dir
            .save_session_cookie(cookie)
    }

    pub fn read_session_cookie(&self) -> Result<Option<String>> {
        self.config_dir.read_session_cookie()
    }

    pub fn delete_session_cookie(&mut self) -> Result<()> {
        self.config_dir.delete_session_cookie()
    }

    pub fn save_personal_puzzle_input(
        &mut self,
        y: Year,
        d: Day,
        input: &str,
    ) -> Result<()> {
        self.cache_dir
            .save_personal_puzzle_input(y, d, input)
    }

    pub fn read_personal_puzzle_input(
        &self,
        y: Year,
        d: Day,
    ) -> Result<Option<String>> {
        self.cache_dir
            .read_personal_puzzle_input(y, d)
    }

    pub fn personal_puzzle_inputs_dir(&self) -> PathBuf {
        self.cache_dir
            .personal_puzzle_inputs_dir()
    }

    pub fn personal_leaderboard_file(&self, y: Year) -> PathBuf {
        self.data_dir
            .personal_leaderboard_file(y)
    }

    pub fn personal_leaderboard_dir(&self) -> PathBuf {
        self.data_dir.personal_leaderboard_dir()
    }

    #[cfg(test)]
    pub fn read_example_puzzle_input(
        &self,
        y: Year,
        d: Day,
        label: &str,
    ) -> Result<String> {
        self.repo_dir
            .read_personal_puzzle_input(y, d, label)
    }

    #[cfg(test)]
    pub fn personal_puzzle_answer(
        &self,
        y: Year,
        d: Day,
        p: Part,
    ) -> Result<String> {
        self.data_dir
            .personal_puzzle_answer(y, d, p)
    }
}

impl RepoDir {
    const ENV_VAR: &'static str = "CARGO_WORKSPACE_DIR";

    pub fn from_env_or_cargo() -> Result<Self> {
        let path = match env_var_dir_check(Self::ENV_VAR)? {
            Some(path) => path,
            None => Self::read_workspace_dir_from_cargo()?,
        };

        Self::try_from(path.as_path())
    }

    #[cfg(test)]
    pub fn read_personal_puzzle_input(
        &self,
        y: Year,
        d: Day,
        label: &str,
    ) -> Result<String> {
        self.example_puzzle_input_file(y, d, label)
            .and_then(read_to_string)
            .or_wrap_with(|| {
                let id = Id((y, d));
                format!("Failed to read {id} example puzzle input '{label}'")
            })
    }

    #[cfg(test)]
    pub fn example_puzzle_input_file(
        &self,
        y: Year,
        d: Day,
        label: &str,
    ) -> Result<PathBuf> {
        let id = Id((y, d));

        let mut path = self.path.clone();
        path.push(format!(
            "aoc/examples/{id}/{id}_example_puzzle_input_{label}.txt"
        ));

        Ok(path)
    }

    /// [As of 2023-05-27, the `CARGO_WORKSPACE_DIR` environment variable
    /// is still a WIP.][1]
    /// Thus, for the time being, this function determines the correct value.
    ///
    /// [1]: https://github.com/rust-lang/cargo/issues/3946
    fn read_workspace_dir_from_cargo() -> Result<PathBuf> {
        let proc = Command::new("cargo")
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .or_wrap()?;

        let stdout = parse_utf8(&proc.stdout)?;
        let stderr = parse_utf8(&proc.stderr)?;

        let result: Result<(), Error> = match proc.status.code() {
            Some(0) => Ok(()),
            Some(c) => Err(err!("Process returned status code {c}: {stderr}")),
            None => Err(err!(
                "Process exitted without any status code (terminated by \
                 signal?): {stderr}"
            )),
        };

        result.or_wrap_with(|| "Failed to run `cargo locate-project`")?;

        let cargo_toml = Path::new(stdout);
        let dir = cargo_toml.parent().ok_or_else(|| {
            err!("Invalid output of `cargo locate-project`: '{stdout}'")
        })?;

        Ok(dir.into())
    }
}

impl DataDir {
    pub fn from_env() -> Result<Self> {
        match dirs::data_dir() {
            Some(mut path) => {
                path.push(APP_SUBDIR_NAME);
                Ok(Self { path })
            }
            None => Err(err!("Failed to determine user data directory")),
        }
    }

    pub fn personal_leaderboard_file(&self, y: Year) -> PathBuf {
        let y = Id(y);

        let mut path = self.personal_leaderboard_dir();
        path.push(format!("{y}_personal_leaderboard_statistics.txt"));

        path
    }

    pub fn personal_leaderboard_dir(&self) -> PathBuf {
        let mut path = self.path.clone();
        path.push(LEADERBOARD_SUBDIR_NAME);
        path
    }

    #[cfg(test)]
    pub fn personal_puzzle_answer(
        &self,
        y: Year,
        d: Day,
        p: Part,
    ) -> Result<String> {
        let id = Id((y, d, p));

        let mut path = self.path.clone();
        path.push(format!(
            "personal_puzzle_answers/{id}_personal_puzzle_answer.txt"
        ));

        read_to_string(&path).map(|data| data.trim_end().to_string())
    }
}

impl ConfigDir {
    /// Creates the directory if it does not exist.
    pub fn from_env() -> Result<Self> {
        match dirs::config_dir() {
            Some(mut path) => {
                path.push(APP_SUBDIR_NAME);
                Self::new(&path)
            }
            None => Err(err!("Failed to determine user config directory")),
        }
    }

    /// Creates the directory if it does not exist.
    pub fn new(dir: &Path) -> Result<Self> {
        create_dir_all(dir)
            .or_wrap_with(|| "Failed to create user config directory")?;

        Ok(Self {
            path: dir.to_owned(),
        })
    }

    pub fn save_session_cookie(&mut self, cookie: &str) -> Result<()> {
        write(self.session_cookie_file(), cookie)
            .or_wrap_with(|| "Failed to save session cookie")
    }

    pub fn read_session_cookie(&self) -> Result<Option<String>> {
        let path = self.session_cookie_file();
        if !path.exists() {
            return Ok(None);
        }

        read_to_string(path)
            .map(Some)
            .or_wrap_with(|| "Failed to read session cookie")
    }

    pub fn delete_session_cookie(&mut self) -> Result<()> {
        delete(self.session_cookie_file())
            .or_wrap_with(|| "Failed to delete session cookie")
    }

    fn session_cookie_file(&self) -> PathBuf {
        let mut path = self.path.clone();
        path.push("session.cookie");
        path
    }
}

impl CacheDir {
    /// Creates the directory if it does not exist.
    pub fn from_env() -> Result<Self> {
        match dirs::cache_dir() {
            Some(mut path) => {
                path.push(APP_SUBDIR_NAME);
                Self::new(&path)
            }
            None => Err(err!("Failed to determine cache directory")),
        }
    }

    /// Creates the directory if it does not exist.
    pub fn new(dir: &Path) -> Result<Self> {
        let path = dir.to_path_buf();
        create_dir_all(&path)
            .or_wrap_with(|| "Failed to create cache directory")?;

        let mut personal_puzzle_inputs_dir = path.clone();
        personal_puzzle_inputs_dir.push("personal_puzzle_inputs");

        create_dir_all(&personal_puzzle_inputs_dir).or_wrap_with(|| {
            "Failed to create personal puzzle inputs directory"
        })?;

        Ok(Self {
            path,
            personal_puzzle_inputs_dir,
        })
    }

    pub fn save_personal_puzzle_input(
        &mut self,
        y: Year,
        d: Day,
        input: &str,
    ) -> Result<()> {
        let path = self.personal_puzzle_input_file(y, d);
        write(path, input)
            .or_wrap_with(|| "Failed to save personal puzzle input")
    }

    pub fn read_personal_puzzle_input(
        &self,
        y: Year,
        d: Day,
    ) -> Result<Option<String>> {
        let path = self.personal_puzzle_input_file(y, d);

        if !path.exists() {
            return Ok(None);
        }

        read_to_string(path)
            .map(Some)
            .or_wrap_with(|| "Failed to read personal puzzle input")
    }

    pub fn personal_puzzle_inputs_dir(&self) -> PathBuf {
        self.personal_puzzle_inputs_dir.clone()
    }

    fn personal_puzzle_input_file(&self, y: Year, d: Day) -> PathBuf {
        let mut path = self.personal_puzzle_inputs_dir();
        path.push(format!("{}_personal_puzzle_input.txt", Id((y, d))));
        path
    }
}

pub fn create_dir_all<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    std::fs::create_dir_all(path).or_wrap_with(|| {
        format!("Failed to create directory '{}'", path.display())
    })
}

pub fn open<P>(path: P) -> Result<BufReader<File>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let file = File::open(path)
        .or_wrap_with(|| format!("Failed to open file '{}'", path.display()))?;

    let reader = BufReader::new(file);
    Ok(reader)
}

pub fn read_to_string<P>(path: P) -> Result<String>
where
    P: AsRef<Path> + Debug,
{
    let mut buffer = String::new();

    let path = path.as_ref();
    open(path)?
        .read_to_string(&mut buffer)
        .or_wrap_with(|| {
            format!("Failed to read contents of file '{}'", path.display())
        })?;

    Ok(buffer)
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
    path: P,
    contents: C,
) -> Result<()> {
    let path = path.as_ref();
    std::fs::write(path, contents)
        .or_wrap_with(|| format!("Failed to write file '{}'", path.display()))
}

pub fn delete<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        return Ok(());
    }

    std::fs::remove_file(path)
        .or_wrap_with(|| format!("Failed to delete '{}'", path.display()))
}

pub fn lines(reader: BufReader<File>) -> impl Iterator<Item = Result<String>> {
    reader
        .lines()
        .map(|r| -> Result<String> {
            r.or_wrap_with(|| "Failed to read line from file")
        })
}

fn env_var_dir_check(var: &str) -> Result<Option<PathBuf>> {
    match std::env::var(var) {
        Ok(v) => Ok(Some(PathBuf::from(v))),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(e)
            .or_wrap_with(|| format!("Environment variable {var} is invalid")),
    }
}

fn from_path_if_dir(p: &Path) -> Result<PathBuf> {
    if !p.is_dir() {
        return Err(err!("Not a directory: '{}'", p.display()));
    }

    Ok(p.to_path_buf())
}

fn parse_utf8(bytes: &[u8]) -> Result<&str> {
    Ok(str::from_utf8(bytes)
        .or_wrap_with(|| "Cannot create string from bytes")?
        .trim())
}

#[cfg(test)]
pub fn tempdir() -> Result<TempDir> {
    tempfile::tempdir().or_wrap_with(|| "Failed to create tempdir")
}

/// Creates a [`Config`] whose paths all point to `path`.
///
/// You can pass _a reference to_ a [`tempfile::TempDir`] value,
/// thereby ensuring it does not go out of scope
/// since that would delete the corresponding directory on the filesystem.
#[cfg(test)]
pub fn create_config_for<P>(path: P) -> Result<Config>
where
    P: AsRef<Path>,
{
    let repo_dir = RepoDir::from_env_or_cargo()?;

    let path = path.as_ref();
    let data_dir = DataDir::try_from(path)?;
    let config_dir = ConfigDir::new(path)?;
    let cache_dir = CacheDir::new(path)?;

    Ok(Config::from(repo_dir, data_dir, config_dir, cache_dir))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    #[cfg_attr(all(windows, miri), ignore)] // Because of `tempdir`
    fn create_config_dir() -> Result<()> {
        let tempdir = tempdir()?;
        ConfigDir::new(tempdir.path())?;
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `set_permissions`
    #[cfg(not(windows))] // Windows allows creating a subdir in a readonly dir
    fn create_config_dir_when_parent_readonly() -> Result<()> {
        let (tempdir, tempdir_path) = tempdir_readonly_with_path()?;

        let mut path = tempdir.path().to_path_buf();
        path.push("bad_config_dir");

        let err = ConfigDir::new(&path).unwrap_err();
        let msg = err.to_string();

        dbg!(&msg, &tempdir_path);
        assert!(msg.contains("Failed to create user config directory"));
        assert!(msg.contains(&tempdir_path));
        assert!(msg.contains("bad_config_dir"));

        Ok(())
    }

    #[test]
    #[cfg_attr(all(windows, miri), ignore)] // Because of `tempdir`
    fn create_cache_dir() -> Result<()> {
        let tempdir = tempdir()?;
        CacheDir::new(tempdir.path())?;
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `set_permissions`
    #[cfg(not(windows))] // Windows allows creating a subdir in a readonly dir
    fn create_cache_dir_when_parent_readonly() -> Result<()> {
        let (tempdir, tempdir_path) = tempdir_readonly_with_path()?;

        let mut path = tempdir.path().to_path_buf();
        path.push("bad_cache_dir");

        let err = CacheDir::new(&path).unwrap_err();
        let msg = err.to_string();

        dbg!(&msg, &tempdir_path);
        assert!(msg.contains("Failed to create cache directory"));
        assert!(msg.contains(&tempdir_path));
        assert!(msg.contains("bad_cache_dir"));

        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `set_permissions`
    #[cfg(not(windows))] // Windows allows creating a subdir in a readonly dir
    fn create_cache_dir_when_itself_readonly() -> Result<()> {
        let (tempdir, tempdir_path) = tempdir_readonly_with_path()?;
        let err = CacheDir::new(tempdir.path()).unwrap_err();
        let msg = err.to_string();

        dbg!(&msg, &tempdir_path);
        assert!(
            msg.contains("Failed to create personal puzzle inputs directory")
        );
        assert!(msg.contains(&tempdir_path));

        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_config_for`
    fn session_cookie() -> Result<()> {
        let tempdir = tempdir()?;
        let mut config = create_config_for(&tempdir)?;

        // Make sure the cookie does not exist if the last test run was aborted.
        config.delete_session_cookie()?;

        let cookie = config.read_session_cookie()?;
        assert!(cookie.is_none());

        config.save_session_cookie("mock cookie")?;
        let cookie = config.read_session_cookie()?;
        assert_eq!(cookie.unwrap(), "mock cookie");

        config.delete_session_cookie()?;
        let cookie = config.read_session_cookie()?;
        assert!(cookie.is_none());

        // Must be idempotent.
        config.delete_session_cookie()?;
        let cookie = config.read_session_cookie()?;
        assert!(cookie.is_none());

        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_config_for`
    fn personal_puzzle_input() -> Result<()> {
        use crate::ident::{D01, D02, Y21};

        let tempdir = tempdir()?;
        let mut config = create_config_for(&tempdir)?;

        let input = config.read_personal_puzzle_input(Y21, D02)?;
        assert!(input.is_none());

        config.save_personal_puzzle_input(Y21, D01, "mock input 1")?;
        let input = config.read_personal_puzzle_input(Y21, D01)?;
        assert_eq!(input.unwrap(), "mock input 1");

        config.save_personal_puzzle_input(Y21, D01, "mock input 2")?;
        let input = config.read_personal_puzzle_input(Y21, D01)?;
        assert_eq!(input.unwrap(), "mock input 2");

        // Must be idempotent.
        config.save_personal_puzzle_input(Y21, D01, "mock input 1")?;
        config.save_personal_puzzle_input(Y21, D01, "mock input 1")?;
        let input = config.read_personal_puzzle_input(Y21, D01)?;
        assert_eq!(input.unwrap(), "mock input 1");

        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `tempfile`
    fn open_ok() -> Result<()> {
        open(good_file()?)?;
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `tempfile`
    fn open_err() -> Result<()> {
        let (_dir, file, name) = missing_file()?;
        let err = open(file).unwrap_err();
        let msg = err.to_string();
        dbg!(&msg);
        assert!(msg.starts_with("Failed to open"));
        assert!(msg.contains(&name));
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `tempfile`
    fn read_to_string_ok() -> Result<()> {
        assert_eq!(read_to_string(good_file()?)?, "Valid file contents\n");
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `tempfile`
    fn read_to_string_missing() -> Result<()> {
        let (_dir, file, name) = missing_file()?;
        let err = read_to_string(file).unwrap_err();
        let msg = err.to_string();
        dbg!(&msg);
        assert!(msg.starts_with("Failed to open"));
        assert!(msg.contains(&name));
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `tempfile`
    fn read_to_string_err() -> Result<()> {
        let (file, name) = existing_file_non_utf8()?;
        let err = read_to_string(file).unwrap_err();
        let msg = err.to_string();
        dbg!(&msg);
        assert!(msg.starts_with("Failed to read contents"));
        assert!(msg.contains(&name));
        Ok(())
    }

    // Cannot test errors, because error behavior is too platform dependent.
    // Check the docs of `std::fs::write` for details.
    #[test]
    #[cfg_attr(miri, ignore)] // Because of `tempfile`
    fn write_ok() -> Result<()> {
        let file = good_file()?;

        write(&file, "mock contents 1")?;
        let contents = read_to_string(&file)?;
        assert_eq!(contents, "mock contents 1");

        write(&file, "mock contents 2")?;
        let contents = read_to_string(&file)?;
        assert_eq!(contents, "mock contents 2");

        // Must be idempotent.
        write(&file, "mock contents 1")?;
        write(&file, "mock contents 1")?;
        let contents = read_to_string(&file)?;
        assert_eq!(contents, "mock contents 1");

        Ok(())
    }

    fn tempdir() -> Result<TempDir> {
        tempfile::tempdir().or_wrap()
    }

    #[cfg(not(windows))] // Windows allows creating a subdir in a readonly dir
    fn tempdir_readonly_with_path() -> Result<(TempDir, String)> {
        let dir = tempdir()?;
        let path = dir.path().display().to_string();

        let mut perms = std::fs::metadata(&dir)
            .unwrap()
            .permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&dir, perms).or_wrap()?;

        Ok((dir, path))
    }

    fn good_file() -> Result<NamedTempFile> {
        let (tempfile, _path) = mktempf(b"Valid file contents\n")?;
        Ok(tempfile)
    }

    fn existing_file_non_utf8() -> Result<(NamedTempFile, String)> {
        mktempf(b"Non-UTF contents: \x00\x9F\x92\x96\n")
    }

    fn missing_file() -> Result<(TempDir, PathBuf, String)> {
        let dir = tempdir()?;
        let name = String::from("unexisting_file");

        let mut path = dir.path().to_path_buf();
        path.push(&name);

        assert!(!path.exists());
        Ok((dir, path, name))
    }

    fn mktempf(content: &[u8]) -> Result<(NamedTempFile, String)> {
        let mut file = NamedTempFile::new().or_wrap()?;

        file.write_all(content).or_wrap()?;

        let name = file
            .path()
            .file_name()
            .ok_or_else(|| err!("Temporary file is missing its path"))?;

        let name = name.to_str().ok_or_else(|| {
            err!("Temporary file has invalid characters: '{name:?}'")
        })?;

        let name = name.to_owned();
        Ok((file, name))
    }
}
