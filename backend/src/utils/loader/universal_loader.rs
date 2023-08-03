use deno_core::anyhow::bail;
use deno_core::error::AnyError;
use deno_core::futures::FutureExt;
use deno_core::resolve_import;
use deno_core::FastString;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceFuture;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
#[cfg(feature = "bundle")]
use deno_graph::source::{LoadFuture, LoadResponse, Loader};
#[cfg(feature = "transpile")]
use deno_transpiler::compile;
use std::path::PathBuf;
use std::pin::Pin;
use std::str;
use std::sync::Arc;

use crate::utils::store::FsModuleStore;
use crate::utils::to_static_str;

use super::{get_source_code, ModuleStore, UniversalModuleLoader};

impl Default for UniversalModuleLoader {
    fn default() -> Self {
        Self {
            store: Some(Arc::new(FsModuleStore::default())),
            compile: true,
        }
    }
}

impl UniversalModuleLoader {
    pub fn new(module_store: Option<Arc<dyn ModuleStore>>, compile: bool) -> Self {
        Self {
            store: module_store,
            compile,
        }
    }

    pub async fn get_and_update_source(
        self,
        m: &ModuleSpecifier,
        #[allow(unused_variables)] minify: bool,
    ) -> Result<String, AnyError> {
        #[allow(unused_mut)]
        let mut code = get_source_code(m).await?;
        #[cfg(feature = "transpile")]
        if self.compile {
            code = compile(m, code, minify)?;
        }
        if let Some(store) = self.store.as_ref() {
            store.put(m.to_string(), code.as_bytes()).await?;
        }
        Ok(code)
    }
}

impl ModuleLoader for UniversalModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        kind: deno_core::ResolutionKind,
    ) -> Result<ModuleSpecifier, deno_core::anyhow::Error> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<&ModuleSpecifier>,
        is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let m = module_specifier.clone();
        let string_specifier = m.to_string();

        let loader = self.clone();
        async move {
            let module_type = get_module_type(&m)?;
            if let Some(store) = loader.store.as_ref() {
                if let Ok(code) = store.get(&string_specifier).await {
                    let code = std::str::from_utf8(&code).unwrap();
                    return Ok(ModuleSource {
                        code: FastString::Static(to_static_str(code)),
                        module_type,
                        module_url_specified: FastString::Static(to_static_str(&string_specifier)),
                        module_url_found: Some(FastString::Static(to_static_str(
                            &string_specifier,
                        ))),
                    });
                }
            }
            let code = loader.get_and_update_source(&m, false).await?;

            Ok(ModuleSource {
                code: FastString::Static(to_static_str(&code)),
                module_type,
                module_url_specified: FastString::Static(to_static_str(&string_specifier)),
                module_url_found: Some(FastString::Static(to_static_str(&string_specifier))),
            })
        }
        .boxed_local()
    }
}

#[cfg(feature = "bundle")]
impl Loader for UniversalModuleLoader {
    fn load(&mut self, specifier: &ModuleSpecifier, _is_dynamic: bool) -> LoadFuture {
        let loader = self.clone();
        let m = specifier.clone();
        async move {
            let code = loader.get_and_update_source(&m, false).await?;
            Ok(Some(LoadResponse::Module {
                content: code.into(),
                specifier: m,
                maybe_headers: None,
            }))
        }
        .boxed_local()
    }
}

fn get_module_type(m: &ModuleSpecifier) -> Result<ModuleType, AnyError> {
    let path = if let Ok(path) = m.to_file_path() {
        path
    } else {
        PathBuf::from(m.path())
    };
    match path.extension() {
        Some(ext) => {
            let lowercase_str = ext.to_str().map(|s| s.to_lowercase());
            match lowercase_str.as_deref() {
                Some("json") => Ok(ModuleType::Json),
                None => bail!("Unknown extension"),
                _ => Ok(ModuleType::JavaScript),
            }
        }
        None => bail!("Unknown media type {:?}", path),
    }
}
