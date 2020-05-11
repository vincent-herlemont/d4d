use std::cell::{RefCell};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

pub use setup::LocalSetupCfg;
pub use setup_provider::LocalSetupProviderCfg;
pub use setup_provider::LocalSetupProviderCloudformationCfg;

use crate::cfg::{EnvPathCfg, EnvPathsCfg};
use crate::cfg::setup::{SetupsCfg};

mod setup;
mod setup_provider;

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalCfg {
    setups: Rc<RefCell<Vec<Rc<RefCell<LocalSetupCfg>>>>>,
}

impl LocalCfg {
    pub fn new() -> Self {
        Self {
            setups: Rc::new(RefCell::new(vec![])),
        }
    }
}

impl SetupsCfg for LocalCfg {
    type Setup = LocalSetupCfg;

    fn get_setups(&self) -> Rc<RefCell<Vec<Rc<RefCell<Self::Setup>>>>> {
        Rc::clone(&self.setups)
    }
}

impl EnvPathsCfg for LocalCfg {
    fn env_paths_dyn(&self) -> Vec<Rc<RefCell<dyn EnvPathCfg>>> {
        self.setups
            .borrow()
            .iter()
            .map(|e| Rc::clone(e) as Rc<RefCell<dyn EnvPathCfg>>)
            .collect()
    }
}

#[cfg(test)]
mod test {
    
    use std::path::PathBuf;
    

    use crate::cfg::{EnvPathCfg, EnvPathsCfg};
    use crate::cfg::{LocalCfg, LocalSetupCfg, LocalSetupProviderCfg};
    use crate::cfg::setup::SetupsCfg;

    #[test]
    fn local_update_public_env_dir() {
        let provider_cfg = LocalSetupProviderCfg::None;
        let setup_cfg = LocalSetupCfg::new("setup".into(), provider_cfg);

        let mut local_cfg = LocalCfg::new();
        local_cfg.add_setup(setup_cfg);

        let env_path = local_cfg.env_paths();
        assert_eq!(env_path, vec![PathBuf::new()]);

        {
            let setup_cfg_1 = local_cfg.get_setup(&"setup".into()).unwrap();
            setup_cfg_1
                .borrow_mut()
                .set_env_path_op(Some("./env_dir/".into()));
        }

        let env_path = local_cfg.env_paths();
        assert_eq!(env_path, vec![PathBuf::from("./env_dir/")]);

        local_cfg.remove_by_name_setup(&"setup".into());
        assert!(local_cfg.get_setup(&"setup".into()).is_none());
    }
}
