use deno_core::serde_json::json;
fn main() {
    let ts_config = json!({
        "allowImportingTsExtensions": true,
        "checkJs": false,
        "emitDecoratorMetadata": false,
        "importsNotUsedAsValues": "remove",
        "inlineSourceMap": false,
        "inlineSources": false,
        "sourceMap": false,
        "jsx": "react",
        "jsxFactory": "React.createElement",
        "jsxFragmentFactory": "React.Fragment",
    });
    println!("{}", ts_config)
}
