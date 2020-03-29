use crate::project::provider::{AwsCfg, ProviderCfg};
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use utils::error::Error;

use utils::result::Result;

const PROJECT_FILE_NAME: &'static str = "d4d.yaml";

fn local_file_path<P: AsRef<Path>>(root: P) -> PathBuf {
    root.as_ref().join(PROJECT_FILE_NAME)
}

fn local_file_exist<P: AsRef<Path>>(root: P) -> Result<()> {
    let path = local_file_path(root);
    if path.exists() {
        Err(Error::new(format!(
            "project file exists {}",
            path.to_string_lossy()
        )))
    } else {
        Ok(())
    }
}

fn save_local_file<P: AsRef<Path>>(root: P, local_projects: &LocalProjects) -> Result<()> {
    let path = local_file_path(root);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    let buf = BufWriter::new(file);
    serde_yaml::to_writer(buf, &local_projects).map_err(|err| {
        Error::wrap(
            format!("fail to save project file {}", path.to_string_lossy()),
            Error::from(err),
        )
    })
}

fn read_local_file<P: AsRef<Path>>(root: P) -> Result<LocalProjects> {
    let current_dir = PathBuf::from(root.as_ref());
    let local_file = local_file_path(&current_dir);
    let file = OpenOptions::new()
        .read(true)
        .open(&local_file)
        .map_err(|err| {
            Error::wrap(
                format!("fail to open project file {}", local_file.to_string_lossy()),
                Error::from(err),
            )
        })?;
    let buf = BufReader::new(file);
    serde_yaml::from_reader(buf)
        .map_err(|err| {
            Error::wrap(
                format!("fail to read project file {}", local_file.to_string_lossy()),
                Error::from(err),
            )
        })
        .map(|local_projects: LocalProjects| LocalProjects {
            current_dir,
            ..local_projects
        })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalProject {
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    public_env_directory: Option<PathBuf>,

    provider: ProviderCfg,
}

impl LocalProject {
    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn public_env_directory(&self) -> Result<PathBuf> {
        self.public_env_directory.clone().ok_or(Error::new(format!(
            "public_env_directory not found for {}",
            self.name()
        )))
    }

    pub fn provider(&self) -> &ProviderCfg {
        &self.provider
    }
}

impl Display for LocalProject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalProjects {
    #[serde(skip)]
    current_dir: PathBuf,

    #[serde(rename = "projects")]
    all: Vec<Box<LocalProject>>,
}

impl Display for LocalProjects {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for local_project in &self.all {
            writeln!(f, " - {}", local_project)?;
        }
        Ok(())
    }
}

impl LocalProjects {
    pub fn load<P: AsRef<Path>>(current_dir: P) -> Result<LocalProjects> {
        let current_dir = current_dir.as_ref().to_path_buf();
        read_local_file(&current_dir)
    }

    pub fn new<P: AsRef<Path>>(current_dir: P) -> Result<LocalProjects> {
        let local_projects = LocalProjects {
            current_dir: current_dir.as_ref().to_owned(),
            all: vec![],
        };
        local_file_exist(&current_dir)?;
        save_local_file(&current_dir, &local_projects)?;
        Ok(local_projects)
    }

    pub fn add<N, PED>(
        &mut self,
        name: N,
        public_env_directory: PED,
        provider: ProviderCfg,
    ) -> Result<()>
    where
        N: AsRef<str>,
        PED: AsRef<Path>,
    {
        let name = name.as_ref().to_string();
        if let Some(_) = self.get(&name) {
            return Err(Error::new(format!("project {} already exists", &name)));
        }
        self.all.push(Box::new(LocalProject {
            name,
            public_env_directory: Some(PathBuf::from(public_env_directory.as_ref())),
            provider,
        }));

        if let Err(err) = save_local_file(&self.current_dir, self) {
            Err(Error::wrap(
                format!(
                    "fail to save local file : {}",
                    self.current_dir.to_string_lossy()
                ),
                err,
            ))
        } else {
            Ok(())
        }
    }

    pub fn get<P: AsRef<str>>(&self, project_name: P) -> Option<&LocalProject> {
        self.all.iter().find_map(|local_project| {
            if local_project.name == project_name.as_ref() {
                Some(local_project.as_ref())
            } else {
                None
            }
        })
    }

    pub fn fake() -> Self {
        let mut aws_cfg_1 = AwsCfg::new("us-east-1");
        aws_cfg_1.set_template_path("./project_test.tpl");

        let mut aws_cfg_2 = AwsCfg::new("us-east-1");
        aws_cfg_2.set_template_path("./project_test_bis.tpl");

        Self {
            current_dir: PathBuf::from("/path/to/local"),
            all: vec![
                Box::new(LocalProject {
                    name: String::from("project_test"),
                    public_env_directory: None,
                    provider: ProviderCfg::ConfAws(aws_cfg_1),
                }),
                Box::new(LocalProject {
                    name: String::from("project_test_bis"),
                    public_env_directory: None,
                    provider: ProviderCfg::ConfAws(aws_cfg_2),
                }),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::project::local::{local_file_path, read_local_file, save_local_file, LocalProjects};
    use insta::assert_yaml_snapshot;
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use std::path::PathBuf;
    use utils::asset::Assets;
    use utils::test::before;

    #[test]
    fn test_read_local_file() {
        let mut assets = HashMap::new();
        assets.insert(
            "d4d.yaml",
            r#"
projects:
    - name: test_1
      provider:
        name: aws
        region: us-east-3
    - name: test_2
      provider:
        name: aws
        region: us-east-3
        template_path: "./test_template.yaml"
        "#,
        );
        let config = before("test_save_local_file", Assets::Static(assets));
        let local_projects = read_local_file(&config.tmp_dir).unwrap();
        assert_eq!(&local_projects.current_dir, &config.tmp_dir);
        assert_yaml_snapshot!(local_projects);
    }

    #[test]
    fn test_save_local_file() {
        let config = before("test_save_local_file", Assets::Static(HashMap::new()));
        let local_projects = LocalProjects::fake();
        let r = save_local_file(&config.tmp_dir, &local_projects);
        assert!(r.is_ok());
        let content = read_to_string(local_file_path(&config.tmp_dir)).unwrap();
        assert_eq!(
            content,
            r#"---
projects:
  - name: project_test
    provider:
      name: aws
      region: us-east-1
      template_path: "./project_test.tpl"
  - name: project_test_bis
    provider:
      name: aws
      region: us-east-1
      template_path: "./project_test_bis.tpl""#
        );

        // Overwrite
        let local_projects = LocalProjects {
            current_dir: PathBuf::new(),
            all: vec![],
        };
        let r = save_local_file(&config.tmp_dir, &local_projects);
        assert!(r.is_ok());
        let content = read_to_string(local_file_path(&config.tmp_dir)).unwrap();
        assert_eq!(content, String::from("---\nprojects: []"));
    }
}
