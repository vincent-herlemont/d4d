use crate::asset::Asset;

/// Get all [`Asset`]
pub fn get_assets() -> Vec<Asset> {
    vec![
        Asset::new(
            "assets/other_conf.yaml",
            include_str!("assets/other_conf.yaml"),
        ),
        Asset::new(
            "assets/valid_aws_template.yaml",
            include_str!("assets/valid_aws_template.yaml"),
        ),
        Asset::new(
            "assets/altered_aws_template.yaml",
            include_str!("assets/altered_aws_template.yaml"),
        ),
        Asset::new("assets/test/test.js", include_str!("./assets/test/test.js")),
        Asset::new(
            "assets/tpl_certificate/certificate.yaml",
            include_str!("assets/tpl_certificate/certificate.yaml"),
        ),
    ]
}
