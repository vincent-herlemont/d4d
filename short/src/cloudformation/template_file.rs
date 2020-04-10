//! Provide in the same struct [`ContentFile`] and Cloudformation [`Template`]
use crate::cloudformation::template::{ContentTemplate, Template};
use std::cmp::Ordering;
use std::path::PathBuf;
use utils::error::Error;
use utils::fs::ContentFile;
use utils::path::filter_extensions;
use utils::result::unwrap_partition;

#[allow(dead_code)]
pub static YAML_EXTENSIONS: [&str; 2] = ["yaml", "yml"];
#[allow(dead_code)]
pub static TEMPLATE_VERSION: &str = "2010-09-09";

/// The configuration of the cloudformation inspector.
#[derive(Debug)]
struct InspectorConfig {
    path: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TemplateFile {
    pub content_file: ContentFile,
    pub template: Template,
}

impl Ord for TemplateFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.content_file.cmp(&other.content_file)
    }
}

impl PartialOrd for TemplateFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.content_file.cmp(&other.content_file))
    }
}

/// Return a list of items [`Result<File>`] matching with AWS cloudformation [`Template`]
///
/// # Errors
///
/// Return [`Vec`] of [`Error::Io`] and/or [`Error::SerdeYaml`].
#[allow(dead_code)]
fn from_paths(paths: &[PathBuf]) -> (Vec<TemplateFile>, Vec<Error>) {
    let paths = filter_extensions(&paths, &YAML_EXTENSIONS);

    let (content_files, mut errors) =
        ContentFile::read_contain_multi(&paths, |line| line.contains(TEMPLATE_VERSION));

    let results: (Vec<_>, Vec<_>) = content_files
        .into_iter()
        .map(
            |content_file| match serde_yaml::from_str::<ContentTemplate>(&content_file.contents) {
                Ok(content_template) => Ok(TemplateFile {
                    content_file,
                    template: Template {
                        nested: vec![],
                        content_template,
                    },
                }),
                Err(e) => Err(Error::from(e)),
            },
        )
        .partition(Result::is_ok);

    let (files, mut error_files) = unwrap_partition(results);

    errors.append(&mut error_files);

    (files, errors)
}

#[cfg(test)]
mod tests {
    use crate::assets::get_all;
    use crate::cloudformation::template::{ContentTemplate, Template};
    use crate::cloudformation::template_file::{from_paths, TemplateFile};
    use utils::assert_find;
    use utils::assert_not_find;
    use utils::asset::default_assets;
    use utils::error::Error;
    use utils::path::retrieve;
    use utils::test::before;

    #[allow(unreachable_patterns)]
    #[test]
    fn from_path_test() {
        let config = before("from_path_test", default_assets(get_all()));
        let paths = retrieve(&config.tmp_dir).expect("fail to get paths");
        let (files, errors) = from_paths(&paths);
        assert_find!(files,TemplateFile{template,..},
            template == &Template {
            content_template: ContentTemplate {
                aws_template_format_version: String::from("2010-09-09"),
                description: Some(String::from("certificate example")),
            },
            nested: vec![],
        });
        assert_eq!(errors.len(), 2);
        assert_find!(errors, Error::Io(_));
        assert_find!(errors, Error::SerdeYaml(_));
        assert_not_find!(errors, Error::Other(_));
    }
}