use deno_core::ModuleSpecifier;
use deno_core::{
    error::AnyError,
    serde_json::{self, json, Value},
};
use serde::{Deserialize, Serialize, Serializer};

/// A structure for managing the configuration of TypeScript
#[derive(Debug, Clone)]
pub struct TsConfig(pub Value);

impl TsConfig {
    /// Create a new `TsConfig` with the base being the `value` supplied.
    pub fn new(value: Value) -> Self {
        TsConfig(value)
    }

    /// Merge a serde_json value into the configuration.
    pub fn merge(&mut self, value: &Value) {
        json_merge(&mut self.0, value);
    }
}

impl From<TsConfig> for deno_ast::EmitOptions {
    fn from(config: TsConfig) -> Self {
        let options: EmitConfigOptions = serde_json::from_value(config.0).unwrap();
        let imports_not_used_as_values = match options.imports_not_used_as_values.as_str() {
            "preserve" => deno_ast::ImportsNotUsedAsValues::Preserve,
            "error" => deno_ast::ImportsNotUsedAsValues::Error,
            _ => deno_ast::ImportsNotUsedAsValues::Remove,
        };
        let (transform_jsx, jsx_automatic, jsx_development) = match options.jsx.as_str() {
            "react" => (true, false, false),
            "react-jsx" => (true, true, false),
            "react-jsxdev" => (true, true, true),
            _ => (false, false, false),
        };
        deno_ast::EmitOptions {
            emit_metadata: options.emit_decorator_metadata,
            imports_not_used_as_values,
            inline_source_map: options.inline_source_map,
            inline_sources: options.inline_sources,
            source_map: options.source_map,
            jsx_automatic,
            jsx_development,
            jsx_factory: options.jsx_factory,
            jsx_fragment_factory: options.jsx_fragment_factory,
            jsx_import_source: options.jsx_import_source,
            transform_jsx,
            var_decl_imports: false,
        }
    }
}

pub fn get_ts_config() -> Result<TsConfig, AnyError> {
    Ok(TsConfig::new(json!({
        "checkJs": false,
        "emitDecoratorMetadata": false,
        "importsNotUsedAsValues": "remove",
        "inlineSourceMap": false,
        "inlineSources": false,
        "sourceMap": false,
        "jsx": "react",
        "jsxFactory": "React.createElement",
        "jsxFragmentFactory": "React.Fragment",
    })))
}

/// A function that works like JavaScript's `Object.assign()`.
fn json_merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                json_merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_merge() {
        let mut value_a = json!({
          "a": true,
          "b": "c"
        });
        let value_b = json!({
          "b": "d",
          "e": false,
        });
        json_merge(&mut value_a, &value_b);
        assert_eq!(
            value_a,
            json!({
              "a": true,
              "b": "d",
              "e": false,
            })
        );
    }
}
