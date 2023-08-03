use askama::Template;
use deno_core::{error::AnyError, ModuleSpecifier};
use deno_graph::{GraphKind, ModuleGraph};
use derive_builder::Builder;
use std::{fs::OpenOptions, sync::Arc};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::utils::ModuleStore;

mod config;
pub mod hook;
pub mod loader;
pub mod minify;
pub mod options;
pub mod output;

pub use config::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleType {
    MainModule,
    /// Return the emitted contents of the program as a single "flattened" ES
    /// module.
    Module,
    /// Return the emitted contents of the program as a single script that
    /// executes the program using an immediately invoked function execution
    /// (IIFE).
    Classic,
}

#[derive(Builder, Clone)]
#[builder(default, pattern = "owned")]
pub struct BundleOptions {
    pub bundle_type: BundleType,
    pub ts_config: TsConfig,
    pub emit_ignore_directives: bool,
    pub module_store: Option<Arc<dyn ModuleStore>>,
    pub minify: bool,
}

#[derive(Template)]
#[template(path = "layout.j2", escape = "none")]
struct BundledJs {
    body: String,
    bundle_type: BundleType,
}

pub async fn bundle(
    module_specifier: ModuleSpecifier,
    out_file: Option<PathBuf>,
) -> Result<(), AnyError> {
    let roots = vec![module_specifier.clone()];
    let mut graph = ModuleGraph::new(GraphKind::All);
    graph.build(roots, &mut loader, Default::default()).await;
    let bundle_output = bundle_module_graph(graph.as_ref())?;
    if let Some(out_file) = out_file {
        let output_bytes = bundle_output.code.as_bytes();
        let output_len = output_bytes.len();
        write_file(&out_file, output_bytes, 0o644)?;
        if let Some(bundle_map) = bundle_output.maybe_map {
            let map_bytes = bundle_map.as_bytes();
            let map_len = map_bytes.len();
            let ext = if let Some(curr_ext) = out_file.extension() {
                format!("{}.map", curr_ext.to_string_lossy())
            } else {
                "map".to_string()
            };
            let map_out_file = out_file.with_extension(ext);
            write_file(&map_out_file, map_bytes, 0o644)?;
        }
    } else {
        println!("{}", bundle_output.code);
    }
    Ok(())
}

fn bundle_module_graph(graph: &deno_graph::ModuleGraph) -> Result<deno_emit::BundleEmit, AnyError> {
    let ts_config = get_ts_config()?;
    deno_emit::bundle_graph(
        graph,
        deno_emit::BundleOptions {
            bundle_type: deno_emit::BundleType::Module,
            emit_options: ts_config.into(),
            emit_ignore_directives: true,
        },
    )
}

fn write_file<T: AsRef<[u8]>>(filename: &Path, data: T, mode: u32) -> std::io::Result<()> {
    write_file_2(filename, data, true, mode, true, false)
}

fn write_file_2<T: AsRef<[u8]>>(
    filename: &Path,
    data: T,
    update_mode: bool,
    mode: u32,
    is_create: bool,
    is_append: bool,
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .read(false)
        .write(true)
        .append(is_append)
        .truncate(!is_append)
        .create(is_create)
        .open(filename)?;

    if update_mode {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = mode & 0o777;
            let permissions = PermissionsExt::from_mode(mode);
            file.set_permissions(permissions)?;
        }
        #[cfg(not(unix))]
        let _ = mode;
    }

    file.write_all(data.as_ref())
}
