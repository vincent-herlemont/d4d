use crate::project::global::{GlobalProject, GlobalProjects};
use crate::project::local::{LocalProject, LocalProjects};
use serde::export::Formatter;
use std::fmt;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use utils::error::Error;
use utils::result::Result;

pub use super::project::global::CurrentProject;
use crate::project::provider::{AwsCfg, ProviderCfg};

pub mod global;
pub mod local;
pub mod provider;

#[derive(Debug)]
pub struct Projects {
    temp_current_project: Option<CurrentProject>,
    local: LocalProjects,
    global: GlobalProjects,
}

#[derive(Debug)]
pub struct Project<'a> {
    local: &'a LocalProject,
    global: &'a GlobalProject,
}

impl<'a> Project<'a> {
    pub fn name(&self) -> String {
        self.local.name()
    }

    /// Return public env directory as an absolute path
    pub fn public_env_directory_abs(&self) -> Result<PathBuf> {
        let public_env_directory = self.local.public_env_directory()?;
        let path = self.global.path()?;
        Ok(path.join(public_env_directory))
    }

    /// Return private env directory as an absolute path
    pub fn private_env_directory_abs(&self) -> Result<PathBuf> {
        let private_env_directory = self.global.private_env_directory()?;
        let path = self.global.path()?;
        Ok(path.join(private_env_directory))
    }

    /// Return project directory as an absolute path
    pub fn project_directory_abs(&self) -> Result<PathBuf> {
        self.global.path()
    }

    /// Return template file as an absolute path
    pub fn template_file_abs(&self) -> Result<PathBuf> {
        let project_path = self.project_directory_abs()?;
        let template_path = self.local.provider().aws()?.template_path()?;
        Ok(project_path.join(template_path))
    }

    /// Return template file relative to project path
    pub fn template_file_rel(&self) -> Result<PathBuf> {
        let project_path = self.project_directory_abs()?;
        let template_file = self.template_file_abs()?;
        Ok(template_file
            .strip_prefix(project_path)
            .map_err(|err| {
                Error::wrap(
                    format!(
                        r#"fail to get relative template path for {} : {}
 - check global projects configuration
"#,
                        self.name(),
                        template_file.to_string_lossy()
                    ),
                    Error::from(err),
                )
            })?
            .to_path_buf())
    }

    pub fn aws(&self) -> Result<&AwsCfg> {
        if let ProviderCfg::ConfAws(aws_cfg) = self.local.provider() {
            Ok(aws_cfg)
        } else {
            Err(Error::new(format!(
                "aws provider not found for {}",
                self.name()
            )))
        }
    }
}

impl<'a> Display for Project<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "- {}", self.name())
    }
}

impl Projects {
    pub fn load<CD, HD>(current_dir: CD, home_dir: HD) -> Result<Projects>
    where
        CD: AsRef<Path>,
        HD: AsRef<Path>,
    {
        let local = LocalProjects::load(current_dir)?;
        let mut global = GlobalProjects::load(home_dir)?;
        global.sync(&local)?;
        Ok(Projects {
            local,
            global,
            temp_current_project: None,
        })
    }

    pub fn init(current_dir: &PathBuf) -> Result<()> {
        LocalProjects::new(current_dir)?;
        Ok(())
    }

    pub fn add<N, P>(&mut self, project_name: N, template_path: P) -> Result<Project>
    where
        N: AsRef<str>,
        P: AsRef<Path>,
    {
        let template_path = template_path
            .as_ref()
            .strip_prefix(self.local.current_dir())
            .map_err(|err| {
                Error::wrap(
                    format!(
                        "fail to create template_path {}",
                        template_path.as_ref().to_string_lossy()
                    ),
                    Error::from(err),
                )
            })?
            .to_path_buf();

        let public_env_directory = template_path
            .parent()
            .ok_or(format!(
                "fail to get directory of template : {}",
                template_path.to_string_lossy()
            ))?
            .to_path_buf();

        let mut aws_cfg = AwsCfg::new();
        aws_cfg.set_template_path(template_path);

        let global = self.global.add(&project_name, self.local.current_dir())?;
        let local = self.local.add(
            &project_name,
            public_env_directory,
            ProviderCfg::ConfAws(aws_cfg),
        )?;
        Ok(Project { global, local })
    }

    pub fn found<P: AsRef<str>>(&self, project_name: P) -> Result<Project> {
        if let (Some(global), Some(local)) = (
            self.global.get(&project_name),
            self.local.get(&project_name),
        ) {
            Ok(Project { global, local })
        } else {
            Err(Error::new(format!(
                "fail to found project {}",
                project_name.as_ref()
            )))
        }
    }

    pub fn list(&self) -> Vec<Project> {
        self.local
            .get_all()
            .iter()
            .filter_map(|local_project| {
                let project_name = local_project.name();
                if let Some(global_project) = self.global.get(&project_name) {
                    return Some(Project {
                        global: global_project,
                        local: local_project,
                    });
                }
                None
            })
            .collect()
    }

    pub fn set_temporary_current_project(&mut self, current_project: CurrentProject) {
        self.temp_current_project = Some(current_project);
    }

    pub fn current_project(&self) -> Result<Project> {
        let project_name = match &self.temp_current_project {
            Some(current_project) => current_project.name()?,
            None => self.global.current_project()?.name()?,
        };
        self.found(project_name)
    }

    pub fn current_env(&self) -> Result<String> {
        let env = match &self.temp_current_project {
            Some(current_project) => current_project.env()?,
            None => self.global.current_project()?.env()?,
        };
        Ok(env)
    }

    pub fn set_current_project_name<P: AsRef<str>>(&mut self, project_name: P) {
        self.global.set_current_project_name(project_name)
    }

    pub fn set_current_env_name<E: AsRef<str>>(&mut self, env: E) -> Result<()> {
        self.global.set_current_env_name(env)
    }

    pub fn save(&self) -> Result<()> {
        // TODO : save local too
        self.global.save()
    }

    pub fn fake() -> Projects {
        Projects {
            global: GlobalProjects::fake(),
            local: LocalProjects::fake(),
            temp_current_project: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::project::{CurrentProject, Projects};
    use std::path::PathBuf;

    #[test]
    fn current_template() {
        let projects = Projects::fake();
        let current_template_file = projects
            .current_project()
            .unwrap()
            .template_file_abs()
            .unwrap();
        assert_eq!(
            current_template_file,
            PathBuf::from("/path/to/local/project_test.tpl")
        );
    }

    #[test]
    fn current_project() {
        let projects = Projects::fake();
        let current_project = projects.current_project().unwrap();
        assert_eq!(current_project.name(), String::from("project_test"));
    }

    #[test]
    fn current_env() {
        let projects = Projects::fake();
        let current_env = projects.current_env().unwrap();
        assert_eq!(current_env, String::from("env_test"));
    }

    #[test]
    fn temporary_current_project() {
        let mut projects = Projects::fake();
        projects.set_temporary_current_project(CurrentProject::new("project_test_bis"));

        let current_project = projects.current_project().unwrap();
        assert_eq!(current_project.name(), String::from("project_test_bis"));
        assert!(projects.current_env().is_err());

        let mut projects = Projects::fake();
        projects.set_temporary_current_project(
            CurrentProject::new("project_test_bis").set_env("watever_env"),
        );
        assert_eq!(projects.current_env().unwrap(), String::from("watever_env"));
    }

    #[test]
    fn list_projects() {
        let projects = Projects::fake();
        let lists = projects.list();
        assert_eq!(lists.len(), 2);
    }
}
