//! The app's own functions, `fns/*.wasm`: WebAssembly core modules
//! (OctoSense ADR 0011) and components (OctoSense ADR 0014).
//!
//! A core module is checked by its header only; the host checks its imports
//! and exports when it loads it. A component is validated here, and its
//! imports must all come from the WASI packages a host scopes to the app
//! ([`ALLOWED_COMPONENT_IMPORTS`]). Whether the manifest may carry one
//! (`requires: ["wasm-components-v1"]`, `storage` for files) is the gate's
//! check ([`crate::gate`]), which also tells reviewers what each component
//! reaches ([`ComponentInfo::reach`]).
//!
//! [`describe`] is `hub component-info`: a file's kind, imports and exports,
//! with each function's parameters and result written as WIT writes them
//! (the same text OctoSense's `wasm.functions` shows).

use serde_json::{json, Value};
use wasmparser::component_types::{ComponentDefinedType, ComponentEntityType, ComponentFuncTypeId, ComponentValType};
use wasmparser::types::TypesRef;
use wasmparser::{Parser, Payload, PrimitiveValType, Validator, WasmFeatures};

/// The WASI packages a component may import: every interface of these, each
/// scoped to its app by the host. Nothing else (`wasi:sockets`, `wasi:http`,
/// non-WASI imports) is admitted.
pub const ALLOWED_COMPONENT_IMPORTS: &[&str] =
    &["wasi:cli/", "wasi:clocks/", "wasi:filesystem/", "wasi:io/", "wasi:random/"];

/// The feature a manifest requires when its `fns/` holds a component.
pub const COMPONENTS_FEATURE: &str = "wasm-components-v1";

/// Whether `bytes` starts like a WebAssembly core module, version 1.
pub fn is_module(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\0asm\x01\0\0\0")
}

/// Whether `bytes` starts like a WebAssembly component (the preamble's
/// version and layer fields are `0d 00 01 00`).
pub fn is_component(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\0asm\x0d\0\x01\0")
}

/// One function a component exports: `name`, or `interface.name` for a
/// function in an exported interface.
#[derive(Clone, Debug, PartialEq)]
pub struct Export {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub result: Option<String>,
}

/// A validated component's imports and exports.
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentInfo {
    /// Every top-level import, as `package:namespace/interface@version`.
    pub imports: Vec<String>,
    /// Its functions, sorted by name.
    pub exports: Vec<Export>,
}

impl ComponentInfo {
    /// The imports outside [`ALLOWED_COMPONENT_IMPORTS`].
    pub fn refused_imports(&self) -> Vec<&str> {
        self.imports
            .iter()
            .map(String::as_str)
            .filter(|name| !ALLOWED_COMPONENT_IMPORTS.iter().any(|p| name.starts_with(p)))
            .collect()
    }

    /// Whether it imports `wasi:filesystem`: the app's own files, which the
    /// host gives it only with the `storage` capability.
    pub fn uses_files(&self) -> bool {
        self.imports.iter().any(|name| name.starts_with("wasi:filesystem/"))
    }

    /// What it reaches, for a reviewer: "the clock, random numbers, files in
    /// its app folder; no network or other apps".
    pub fn reach(&self) -> String {
        let mut parts = Vec::new();
        if self.imports.iter().any(|n| n.starts_with("wasi:clocks/")) {
            parts.push("the clock");
        }
        if self.imports.iter().any(|n| n.starts_with("wasi:random/")) {
            parts.push("random numbers");
        }
        parts.push(if self.uses_files() { "files in its app folder" } else { "no files" });
        format!("{}; no network or other apps", parts.join(", "))
    }
}

/// Validates a component and reads its imports and exports.
pub fn inspect_component(bytes: &[u8]) -> Result<ComponentInfo, String> {
    if !is_component(bytes) {
        return Err("not a WebAssembly component".into());
    }
    let mut validator = Validator::new_with_features(WasmFeatures::default() | WasmFeatures::COMPONENT_MODEL);
    let types = validator
        .validate_all(bytes)
        .map_err(|e| format!("not a valid WebAssembly component: {e}"))?;
    let types = types.as_ref();
    let (imports, export_names) = top_level_names(bytes)?;
    let mut exports = Vec::new();
    for name in export_names {
        match types.component_entity_type_of_export(&name) {
            Some(ComponentEntityType::Func(id)) => exports.push(function(&types, name.clone(), id)),
            Some(ComponentEntityType::Instance(id)) => {
                let short = short_name(&name).to_string();
                for (fname, entity) in types[id].exports.iter() {
                    if let ComponentEntityType::Func(fid) = entity {
                        exports.push(function(&types, format!("{short}.{fname}"), *fid));
                    }
                }
            }
            _ => {}
        }
    }
    exports.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(ComponentInfo { imports, exports })
}

/// `hub component-info`: what a function file is, imports and exports.
pub fn describe(bytes: &[u8]) -> Result<Value, String> {
    if is_component(bytes) {
        let info = inspect_component(bytes)?;
        let exports: Vec<Value> = info
            .exports
            .iter()
            .map(|e| json!({"name": e.name, "params": e.params, "result": e.result}))
            .collect();
        return Ok(json!({"kind": "component", "imports": info.imports, "exports": exports}));
    }
    if is_module(bytes) {
        return describe_module(bytes);
    }
    Err("not a WebAssembly core module or component".into())
}

/// A core module's imports (`module.name`) and exported functions with their
/// core types.
fn describe_module(bytes: &[u8]) -> Result<Value, String> {
    let mut validator = Validator::new_with_features(WasmFeatures::default());
    let types = validator
        .validate_all(bytes)
        .map_err(|e| format!("not a valid WebAssembly module: {e}"))?;
    let types = types.as_ref();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.map_err(|e| e.to_string())? {
            Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    let import = import.map_err(|e| e.to_string())?;
                    imports.push(format!("{}.{}", import.module, import.name));
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.map_err(|e| e.to_string())?;
                    if export.kind != wasmparser::ExternalKind::Func {
                        continue;
                    }
                    let ty = types.core_function_at(export.index);
                    let func = types[ty].unwrap_func();
                    let params: Vec<Value> = func
                        .params()
                        .iter()
                        .enumerate()
                        .map(|(i, t)| json!([format!("p{i}"), t.to_string()]))
                        .collect();
                    let result = match func.results() {
                        [] => Value::Null,
                        [one] => json!(one.to_string()),
                        many => json!(format!(
                            "({})",
                            many.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
                        )),
                    };
                    exports.push(json!({"name": export.name, "params": params, "result": result}));
                }
            }
            _ => {}
        }
    }
    Ok(json!({"kind": "module", "imports": imports, "exports": exports}))
}

/// The top-level component's import names and function or instance export
/// names, in order. Nested modules and components (`parse_all` flattens
/// them) are skipped by depth.
fn top_level_names(bytes: &[u8]) -> Result<(Vec<String>, Vec<String>), String> {
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut depth = 0usize;
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.map_err(|e| e.to_string())? {
            Payload::ModuleSection { .. } | Payload::ComponentSection { .. } => depth += 1,
            Payload::End(_) => depth = depth.saturating_sub(1),
            Payload::ComponentImportSection(reader) if depth == 0 => {
                for import in reader {
                    imports.push(import.map_err(|e| e.to_string())?.name.0.to_string());
                }
            }
            Payload::ComponentExportSection(reader) if depth == 0 => {
                for export in reader {
                    let export = export.map_err(|e| e.to_string())?;
                    if matches!(
                        export.kind,
                        wasmparser::ComponentExternalKind::Func | wasmparser::ComponentExternalKind::Instance
                    ) {
                        exports.push(export.name.0.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    Ok((imports, exports))
}

fn function(types: &TypesRef<'_>, name: String, id: ComponentFuncTypeId) -> Export {
    let ty = &types[id];
    Export {
        name,
        params: ty
            .params
            .iter()
            .map(|(n, t)| (n.to_string(), wit(types, t)))
            .collect(),
        result: ty.result.as_ref().map(|t| wit(types, t)),
    }
}

/// `my:pkg/markdown@0.1.0` → `markdown`.
fn short_name(full: &str) -> &str {
    let tail = full.rsplit('/').next().unwrap_or(full);
    tail.split('@').next().unwrap_or(tail)
}

fn primitive(p: &PrimitiveValType) -> &'static str {
    match p {
        PrimitiveValType::Bool => "bool",
        PrimitiveValType::S8 => "s8",
        PrimitiveValType::U8 => "u8",
        PrimitiveValType::S16 => "s16",
        PrimitiveValType::U16 => "u16",
        PrimitiveValType::S32 => "s32",
        PrimitiveValType::U32 => "u32",
        PrimitiveValType::S64 => "s64",
        PrimitiveValType::U64 => "u64",
        PrimitiveValType::F32 => "f32",
        PrimitiveValType::F64 => "f64",
        PrimitiveValType::Char => "char",
        PrimitiveValType::String => "string",
        PrimitiveValType::ErrorContext => "error-context",
    }
}

/// A value type as WIT writes it: records, variants and enums by their shape,
/// the same text OctoSense's `wasm.functions` shows.
fn wit(types: &TypesRef<'_>, ty: &ComponentValType) -> String {
    let id = match ty {
        ComponentValType::Primitive(p) => return primitive(p).to_string(),
        ComponentValType::Type(id) => *id,
    };
    let join = |items: Vec<String>| items.join(", ");
    match &types[id] {
        ComponentDefinedType::Primitive(p) => primitive(p).to_string(),
        ComponentDefinedType::Record(record) => format!(
            "record {{ {} }}",
            join(record.fields.iter().map(|(n, t)| format!("{n}: {}", wit(types, t))).collect())
        ),
        ComponentDefinedType::Variant(variant) => format!(
            "variant {{ {} }}",
            join(
                variant
                    .cases
                    .iter()
                    .map(|(n, case)| match &case.ty {
                        Some(t) => format!("{n}({})", wit(types, t)),
                        None => n.to_string(),
                    })
                    .collect()
            )
        ),
        ComponentDefinedType::List(t) => format!("list<{}>", wit(types, t)),
        ComponentDefinedType::Tuple(tuple) => {
            format!("tuple<{}>", join(tuple.types.iter().map(|t| wit(types, t)).collect()))
        }
        ComponentDefinedType::Flags(names) => {
            format!("flags {{ {} }}", join(names.iter().map(|n| n.to_string()).collect()))
        }
        ComponentDefinedType::Enum(names) => {
            format!("enum {{ {} }}", join(names.iter().map(|n| n.to_string()).collect()))
        }
        ComponentDefinedType::Option(t) => format!("option<{}>", wit(types, t)),
        ComponentDefinedType::Result { ok, err } => match (ok, err) {
            (Some(ok), Some(err)) => format!("result<{}, {}>", wit(types, ok), wit(types, err)),
            (Some(ok), None) => format!("result<{}>", wit(types, ok)),
            (None, Some(err)) => format!("result<_, {}>", wit(types, err)),
            (None, None) => "result".into(),
        },
        _ => "resource".into(),
    }
}
