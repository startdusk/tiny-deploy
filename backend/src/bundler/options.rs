use std::sync::Arc;

use crate::utils::FsModuleStore;
use deno_ast::swc;

use crate::bundler::{config::get_ts_config, BundleOptions, BundleType};

impl Default for BundleType {
    fn default() -> Self {
        BundleType::Module
    }
}

impl From<BundleType> for swc::bundler::ModuleType {
    fn from(bundle_type: BundleType) -> Self {
        match bundle_type {
            BundleType::Classic => Self::Iife,
            BundleType::Module => Self::Es,
            BundleType::MainModule => Self::Es,
        }
    }
}

impl Default for BundleOptions {
    fn default() -> Self {
        Self {
            bundle_type: BundleType::Module,
            ts_config: get_ts_config().unwrap(),
            emit_ignore_directives: false,
            module_store: Some(Arc::new(FsModuleStore::default())),
            minify: true,
        }
    }
}
