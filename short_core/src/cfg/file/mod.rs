#![feature(bool_to_option)]

use std::env::var;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use fs_extra::file::{read_to_string, write_all};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::cfg::{GlobalCfg, LocalCfg};
use crate::cfg::file::find::find_local_cfg;
use crate::cfg::global::GLOCAL_FILE_NAME;
use crate::cfg::new::NewCfg;

mod find;

#[derive(Debug)]
pub struct FileCfg<C>
where
    C: Serialize + DeserializeOwned + NewCfg,
{
    path: Option<PathBuf>,
    cfg: C,
}

impl<C> FileCfg<C>
where
    C: Serialize + DeserializeOwned + NewCfg,
{
    pub fn load(path: &PathBuf) -> Result<FileCfg<C>> {
        let str = read_to_string(&path)?;
        let cfg = serde_yaml::from_str(str.as_str())
            .context(format!("fail to parse {}", path.to_string_lossy()))?;
        Ok(Self {
            cfg,
            path: Some(path.to_path_buf()),
        })
    }
    pub fn new(path: &PathBuf, cfg: C) -> Result<FileCfg<C>> {
        if (!path.is_absolute()) {
            return Err(anyhow!("cfg file path must be an abosulte path"));
        }
        Ok(Self {
            path: Some(path.to_path_buf()),
            cfg,
        })
    }
}

pub fn load_or_new_local_cfg(dir: &PathBuf) -> Result<FileCfg<LocalCfg>> {
    let local_cfg_file = var("SHORT_LOCAL_CFG_FILE").map_or("short.yml".to_string(), |v| v);
    let local_cfg_file = dir.join(local_cfg_file);

    let local = load_local_cfg(&local_cfg_file).map_or(
        FileCfg::new(&local_cfg_file, LocalCfg::new())
            .context("fail to create new local cfg file")?,
        |v| v,
    );

    Ok(local)
}

pub fn load_or_new_global_cfg(dir: &PathBuf) -> Result<FileCfg<GlobalCfg>> {
    let global_cfg_dir = var("SHORT_GLOBAL_CFG_DIR").map_or(".short/".to_string(), |v| v);
    let global_dir = dir.join(global_cfg_dir);
    let global_cfg_file = global_dir.join(GLOCAL_FILE_NAME.to_string());

    let global = load_global_cfg(&global_cfg_file)
        .map_or(FileCfg::new(&global_cfg_file, GlobalCfg::new())?, |v| v);

    Ok(global)
}

pub fn load_local_cfg(file: &PathBuf) -> Result<FileCfg<LocalCfg>> {
    let dir = file.parent().context(format!(
        "fail to reach directory of local cfg file {}",
        file.to_string_lossy()
    ))?;
    let file_name = file
        .file_name()
        .context(format!(
            "fail te get file name of local cfg file {}",
            file.to_string_lossy()
        ))?
        .to_str()
        .context(format!(
            "cfg file name mut be contain only utf-8 char : {}",
            file.to_string_lossy()
        ))?
        .to_string();
    let path = find_local_cfg(dir.to_path_buf(), file_name).context("fail to found local cfg")?;
    FileCfg::load(&path)
}

pub fn load_global_cfg(file: &PathBuf) -> Result<FileCfg<GlobalCfg>> {
    FileCfg::load(file)
}

impl From<LocalCfg> for FileCfg<LocalCfg> {
    fn from(cfg: LocalCfg) -> Self {
        Self { cfg, path: None }
    }
}

impl From<GlobalCfg> for FileCfg<GlobalCfg> {
    fn from(cfg: GlobalCfg) -> Self {
        Self { cfg, path: None }
    }
}

impl<C> Display for FileCfg<C>
where
    C: Serialize + DeserializeOwned + NewCfg,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Ok(content) = serde_yaml::to_string(&self.cfg).map_err(|err| fmt::Error {}) {
            write!(f, "{}", content);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use anyhow::{Context, Result};
    use fs_extra::dir::DirEntryAttr::Path;
    use predicates::prelude::Predicate;
    use predicates::str::contains;

    use short_utils::integration_test::environment::IntegrationTestEnvironment;

    use crate::cfg::{EnvPathsCfg, LocalCfg};
    use crate::cfg::file::{FileCfg, load_local_cfg};
    use crate::cfg::NewCfg;

    fn init_env() -> IntegrationTestEnvironment {
        let mut e = IntegrationTestEnvironment::new("cmd_help");
        e.add_file("setup_1/template.yaml", "");
        e.add_file(
            "short.yml",
            r"#---
setups:
  - name: setup_1'
    public_env_directory: 'setup_1/'
    provider:
      name: cloudformation
      template: setup_1/template.yaml
#",
        );
        e.setup();
        e
    }

    #[test]
    fn load() {
        let e = init_env();
        let file_cfg: Result<FileCfg<LocalCfg>> = FileCfg::load(&e.path().join("short.yml"));
        let file_cfg = file_cfg.unwrap();
        let path = file_cfg.path.unwrap().clone();
        assert!(contains("short.yml").eval(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn load_local() {
        let e = init_env();
        let file_local_cfg = load_local_cfg(&e.path().join("setup_1/short.yml"));
        let file_local_cfg = file_local_cfg.unwrap();
        let path = file_local_cfg.path.unwrap().clone();
        assert!(contains("short.yml").eval(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn local_new_file() {
        let e = init_env();
        let local_cfg = LocalCfg::new();

        FileCfg::new(&PathBuf::from("toto"), local_cfg).unwrap_err();

        let local_cfg = LocalCfg::new();
        let file_cfg_local =
            FileCfg::new(&e.path().join(PathBuf::from("toto")), local_cfg).unwrap();
    }

    #[test]
    fn load_file() {
        let e = init_env();
    }
}
