//! Helper for test related of d4d domain.
use crate::asset::{to_dir, Assets};
use std::collections::HashMap;
use std::env::current_exe;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use tempdir::TempDir;

#[derive(Debug)]
pub struct Config {
    pub tmp_dir: PathBuf,
    temp_dir: TempDir,
}

#[derive(Debug)]
pub struct ConfigCli {
    // Config
    pub tmp_dir: PathBuf,
    temp_dir: TempDir,

    pub tmp_home_dir: PathBuf,
    pub tmp_project_dir: PathBuf,
    pub exec_path: PathBuf,
}

const HOME: &'static str = "home/.keep";
const PROJECT: &'static str = "project/.keep";

impl ConfigCli {
    pub fn command(&self) -> Command {
        self.other_command(&self.exec_path)
    }
    pub fn other_command<S: AsRef<OsStr>>(&self, program: S) -> Command {
        let mut command = Command::new(program);
        command.current_dir(&self.tmp_project_dir);
        command.env("HOME", &self.tmp_home_dir);
        command
    }
}

impl Config {
    pub fn cli<S: AsRef<str>>(self, carte_name: S) -> ConfigCli {
        let mut assets = HashMap::new();
        assets.insert(HOME, "");
        assets.insert(PROJECT, "");

        to_dir(&self.tmp_dir, Assets::All(assets)).expect("fail to copy cli assets");

        let home_dir = self.tmp_dir.join(HOME).parent().unwrap().to_path_buf();
        let project_dir = self.tmp_dir.join(PROJECT).parent().unwrap().to_path_buf();

        let current_exec = current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
            .join(carte_name.as_ref());

        ConfigCli {
            temp_dir: self.temp_dir,
            tmp_dir: self.tmp_dir,

            tmp_home_dir: home_dir,
            tmp_project_dir: project_dir,
            exec_path: current_exec,
        }
    }
}

/// Return [`InspectorConfig`], create temporary directory and copy asset on it.
///
/// The temporary directory is owned by [`InspectorConfig.path`].
///
pub fn before(test_name: &str, assets: Assets) -> Config {
    let test_name = format!("{}.{}", "d4d", test_name);

    // Create temporary directory.
    let temp_dir = TempDir::new(test_name.as_str()).expect("fail to create temporary directory");

    // Copy assets to it.
    to_dir(temp_dir.path(), assets).expect("fail to copy assets");

    Config {
        tmp_dir: temp_dir.path().to_path_buf(),
        temp_dir,
    }
}

/// Assert that patter value or|and expression is present on an vector.
///
/// # Notice
/// The macro don't take the ownership of a vector.
///
/// You have to add [`#[allow(unreachable_patterns)]`] to avoid warning
///
/// # Example
/// ```
/// use utils::assert_find;
/// use std::panic::catch_unwind;
/// let v = vec![1,2,3,4];
///
/// assert_find!(v, 2);
/// catch_unwind(|| {
///  assert_find!(v, 8); // assertion failed: can not found {8}  in {v}
/// });
/// assert_find!(v, el, el < &&5);
/// catch_unwind(|| {
///  assert_find!(v, el, el > &&5); // assertion failed: can not found {el} with expresion {el > &&5} in {v}
/// });
///
/// ```
#[macro_export]
macro_rules! assert_find {
    ($v:ident,$arms:pat) => {
        assert_find!($v, $arms, true, true)
    };
    ($v:ident,$arms:pat,$e:expr) => {
        assert_find!($v, $arms, $e, true)
    };
    ($v:ident,$arms:pat,$e:expr,$b:expr) => {
        if (&$v)
            .iter()
            .find(|el| match el {
                $arms => $e,
                _ => false,
            })
            .is_some()
            != $b
        {
            panic!(format!(
                "assertion failed: {} {{{}}} {} in {{{}}}",
                if ($b) {
                    String::from("can not found")
                } else {
                    String::from("found")
                },
                String::from(stringify!($arms)),
                if (stringify!($e) == "true" || stringify!($e) == "false") {
                    String::from("")
                } else {
                    String::from(format!(
                        "with expresion {{{}}}",
                        String::from(stringify!($e))
                    ))
                },
                String::from(stringify!($v))
            ));
        };
    };
}

/// Assert that patter value or|and expression is not present on an vector.
///
/// # Notice
/// The macro don't take the ownership of a vector.
///
/// You have to add [`#[allow(unreachable_patterns)]`] to avoid warning
///
/// # Example
/// ```
/// use std::panic::catch_unwind;
/// use crate::utils::assert_not_find;
/// use crate::utils::assert_find; // TODO : found a way to not have to import assert_find macro.
/// let v = vec![1,2,3,4];
///
/// assert_not_find!(v, 8);
/// catch_unwind(|| {
///  assert_not_find!(v, 2); // assertion failed: found {2}  in {v}
/// });
/// assert_not_find!(v, el, el > &&5);
/// catch_unwind(|| {
///  assert_not_find!(v, el, el < &&5); // assertion failed: found {el} with expresion {el < &&5} in {v}
/// });
/// ```
#[macro_export]
macro_rules! assert_not_find {
    ($v:ident,$arms:pat) => {
        assert_find!($v, $arms, true, false)
    };
    ($v:ident,$arms:pat,$e:expr) => {
        assert_find!($v, $arms, $e, false)
    };
}

#[cfg(test)]
mod tests {

    #[allow(unreachable_patterns)]
    #[test]
    fn assert_find_macro_test() {
        let v = vec![1, 2, 3, 4];
        assert_find!(v, 2);
        // assert_find!(v, 8); // assertion failed: can not found {8}  in {v}
        assert_find!(v, el, el < &&5);
        // assert_find!(v, el, el > &&5); // assertion failed: can not found {el} with expresion {el > &&5} in {v}

        assert_not_find!(v, 8);
        // assert_not_find!(v, 2); // assertion failed: found {2}  in {v}
        assert_not_find!(v, el, el > &&5);
        //assert_not_find!(v, el, el < &&5); // assertion failed: found {el} with expresion {el < &&5} in {v}
    }
}
