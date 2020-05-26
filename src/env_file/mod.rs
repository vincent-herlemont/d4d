use std::fmt::{Display, Formatter};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::path::{Path, PathBuf};

pub use read_dir::read_dir;

mod read_dir;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub struct Var {
    name: String,
    value: String,
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}={}", self.name, self.value)
    }
}

impl Var {
    fn new<N, V>(name: N, value: V) -> Self
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        Self {
            name: String::from(name.as_ref()),
            value: String::from(value.as_ref()),
        }
    }

    fn from_line(line: &String) -> Result<Self> {
        let vars: Vec<&str> = line.rsplitn(2, "=").collect();
        match vars.as_slice() {
            [value, name] => {
                let value = value.trim_end();
                let value = value.trim_start();
                let name = name.trim_end();
                let name = name.trim_start();

                if name.contains(char::is_whitespace) {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("space on name \"{}\"", name),
                    ));
                }

                Ok(Var::new(name, value))
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "fail to parse env")),
        }
    }

    fn tuple(&self) -> (String, String) {
        (self.name.to_owned(), self.value.to_owned())
    }
}

#[derive(Debug)]
pub struct Comment {
    value: String,
}

impl Display for Comment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#{}", self.value)
    }
}

impl Comment {
    fn from_line(line: &String) -> Option<Self> {
        let parts: Vec<&str> = line.splitn(2, "#").collect();
        match parts.as_slice() {
            [empty, comment] if empty.is_empty() => Some(Self {
                value: String::from(comment.to_owned()),
            }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Entry {
    Var(Var),
    Comment(Comment),
    Empty,
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::Var(var) => write!(f, "{}", var),
            Entry::Comment(comment) => write!(f, "{}", comment),
            Entry::Empty => writeln!(f, ""),
        }
    }
}

impl Entry {
    fn empty(line: &String) -> Option<Entry> {
        let line = line.trim_start();
        let line = line.trim_end();
        if line.len() > 0 {
            None
        } else {
            Some(Entry::Empty)
        }
    }

    fn comment(line: &String) -> Option<Entry> {
        let comment = Comment::from_line(&line)?;
        Some(Entry::Comment(comment))
    }

    fn var(line: &String) -> Result<Entry> {
        let var = Var::from_line(line)?;
        Ok(Entry::Var(var))
    }
}

#[derive(Debug)]
pub struct Env {
    file: Option<PathBuf>,
    entries: Vec<Entry>,
}

impl Display for Env {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for entry in &self.entries {
            write!(f, "{}", entry)?;
        }
        Ok(())
    }
}

impl Env {
    pub fn new() -> Self {
        Self {
            file: None,
            entries: vec![],
        }
    }

    /// ```
    /// use crate::short::env_file::Env;
    /// let mut env = Env::new();
    ///
    /// env.add("var1","test");
    ///
    /// if let Ok((_,value)) = env.get("var1") {
    ///     assert!(true);
    /// } else {
    ///     assert!(false);
    /// }
    /// ```
    pub fn add<N, V>(&mut self, name: N, value: V)
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        let name = String::from(name.as_ref());
        let value = String::from(value.as_ref());
        let entry = Entry::Var(Var { name, value });
        self.entries.append(&mut vec![entry])
    }

    /// ```
    /// use crate::short::env_file::Env;
    /// let mut env = Env::new();
    ///
    /// env.add("var1","test");
    ///
    /// if let Ok((_,value)) = env.get("var1") {
    ///     assert!(true);
    /// } else {
    ///     assert!(false);
    /// }
    /// assert!(env.get("var2").is_err());
    /// ```
    pub fn get<N: AsRef<str>>(&self, name: N) -> Result<(String, String)> {
        self.entries
            .iter()
            .find_map(|entry| {
                if let Entry::Var(var) = entry {
                    if var.name == String::from(name.as_ref()) {
                        return Some(var.tuple());
                    }
                }
                None
            })
            .ok_or(Error::new(
                ErrorKind::NotFound,
                format!(
                    "fail to found env var {} {}",
                    name.as_ref().to_string(),
                    self.name()
                        .as_ref()
                        .map_or(String::new(), |env| format!(" to env {} ", env))
                ),
            ))
    }

    pub fn is_set<N, V>(&self, name: N, value: V) -> bool
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        self.get(name)
            .map_or(false, |(_, env_value)| env_value == value.as_ref())
    }

    pub fn add_empty_line(&mut self) {
        self.entries.append(&mut vec![Entry::Empty]);
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file_name_err = || {
            Error::new(
                ErrorKind::InvalidData,
                format!(
                    "fail to read file env file name {}",
                    path.to_string_lossy().trim()
                ),
            )
        };

        let _file_name = path
            .file_name()
            .ok_or(file_name_err())?
            .to_str()
            .ok_or(file_name_err())?
            .to_string();

        let file = OpenOptions::new().read(true).open(&path)?;
        let mut buf_reader = BufReader::new(file);
        let mut env = Env::from_reader(&mut buf_reader)?;
        env.file = Some(path);
        Ok(env)
    }

    pub fn from_reader(cursor: &mut dyn BufRead) -> Result<Self> {
        let mut entries = vec![];
        for line in cursor.lines() {
            let line = line?;
            if let Some(empty) = Entry::empty(&line) {
                entries.append(&mut vec![empty]);
            } else if let Some(comment) = Entry::comment(&line) {
                entries.append(&mut vec![comment]);
            } else {
                let var = Entry::var(&line)?;
                entries.append(&mut vec![var]);
            }
        }

        Ok(Env {
            file: None,
            entries,
        })
    }

    pub fn set_path(&mut self, path: &PathBuf) {
        self.file = Some(path.to_owned());
    }

    pub fn file_name(&self) -> Result<String> {
        if let Some(file) = &self.file {
            if let Some(file_name) = file.file_name() {
                let file_name_err = || {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("fail to read file env file name {:?}", file_name),
                    )
                };
                let file_name = file_name.to_str().ok_or(file_name_err())?.to_string();
                return Ok(file_name);
            }
        }
        Err(Error::new(
            ErrorKind::NotFound,
            format!("env file not found",),
        ))
    }

    pub fn name(&self) -> Result<String> {
        let file_name = self.file_name()?;
        let name = file_name.trim_start_matches('.');
        return Ok(name.to_string());
    }

    pub fn iter(&self) -> EnvIterator {
        EnvIterator {
            index: 0,
            env: &self,
        }
    }
}

#[derive(Debug)]
pub struct EnvIterator<'a> {
    env: &'a Env,
    index: usize,
}

impl<'a> Iterator for EnvIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(var) = self.env.entries.get(self.index) {
            self.index += 1;
            if let Entry::Var(var) = var {
                return Some(var.tuple());
            } else {
                return self.next();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::env_file::Env;
    use std::io::Cursor;

    #[test]
    fn env_iterator() {
        let mut env = Env::new();
        env.add("name1", "value1");
        env.add_empty_line();
        env.add("name2", "value2");

        let mut iter = env.iter();

        if let Some((name, value)) = iter.next() {
            assert_eq!(name, "name1");
            assert_eq!(value, "value1");
        } else {
            assert!(false);
        }

        if let Some((name, value)) = iter.next() {
            assert_eq!(name, "name2");
            assert_eq!(value, "value2");
        } else {
            assert!(false);
        }

        assert!(iter.next().is_none());
    }

    #[test]
    fn name() {
        let mut env = Env::new();
        assert!(env.name().is_err());
        env.set_path(&"/test-env".into());
        let file_name = env.file_name().unwrap();
        assert_eq!(file_name, "test-env");
        let name = env.name().unwrap();
        assert_eq!(name, "test-env");

        // trim dot
        let mut env = Env::new();
        env.set_path(&"test/.test-env".into());
        let file_name = env.file_name().unwrap();
        assert_eq!(file_name, ".test-env");
        let name = env.name().unwrap();
        assert_eq!(name, "test-env");
    }

    #[test]
    fn is_set() {
        let mut env = Env::new();
        env.add("name1", "value1");
        let is_set = env.is_set("name1", "value1");
        assert!(is_set);
        let is_set = env.is_set("name1", "value2");
        assert!(!is_set);
    }

    #[test]
    fn empty() {
        let mut content = Cursor::new(br#""#);
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "")
    }

    #[test]
    fn once_var() {
        let mut content = Cursor::new(br#"A=a"#);
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "A=a\n")
    }

    #[test]
    fn name_end_with_space() {
        let mut content = Cursor::new(br#"A=a "#);
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "A=a\n")
    }

    #[test]
    fn name_start_with_space() {
        let mut content = Cursor::new(br#"A= a"#);
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "A=a\n")
    }

    #[test]
    fn value_end_with_space() {
        let mut content = Cursor::new(br#"A =a"#);
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "A=a\n");
    }

    #[test]
    fn value_start_with_space() {
        let mut content = Cursor::new(br#" A=a"#);
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "A=a\n");
    }

    #[test]
    fn value_with_space_inside() {
        let mut content = Cursor::new(br#"A B=a"#);
        let env = Env::from_reader(&mut content);
        assert!(env.is_err());
    }

    #[test]
    fn empty_comment() {
        let mut content = Cursor::new(br#"#"#);
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "#\n")
    }

    #[test]
    fn comment() {
        let mut content = Cursor::new(br#"#test"#);
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "#test\n")
    }

    #[test]
    fn multi_var() {
        let mut content = Cursor::new(
            br#"A=a
    B=b"#,
        );
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "A=a\nB=b\n")
    }

    #[test]
    fn multi_var_and_comment() {
        let mut content = Cursor::new(
            br#"A=a
#test
B=b"#,
        );
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "A=a\n#test\nB=b\n")
    }

    #[test]
    fn empty_lines() {
        let mut content = Cursor::new(
            br#"

"#,
        );
        let env = Env::from_reader(&mut content).unwrap();
        assert_eq!(format!("{}", env), "\n\n")
    }
}