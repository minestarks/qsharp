// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(unknown_lints, clippy::empty_docs)]
#![allow(non_snake_case)]

use diagnostic::interpret_errors_into_qsharp_errors;
use katas::check_solution;
use language_service::IOperationInfo;
use num_bigint::BigUint;
use num_complex::Complex64;
use project_system::{into_qsc_args, ProgramConfig};
use qsc::{
    compile::{self},
    format_state_id, get_latex,
    hir::PackageId,
    interpret::{
        self,
        output::{self, Receiver},
        CircuitEntryPoint,
    },
    target::Profile,
    LanguageFeatures, PackageStore, PackageType, SourceContents, SourceMap, SourceName, SparseSim,
    TargetCapabilityFlags,
};
use resource_estimator::{self as re, estimate_entry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt::Write, str::FromStr};
use wasm_bindgen::prelude::*;

mod debug_service;
mod diagnostic;
mod language_service;
mod line_column;
mod logging;
mod project_system;
mod serializable_type;

#[cfg(test)]
mod tests;

thread_local! {
    static STORE_CORE_STD: (PackageStore, PackageId) = {
        let mut store = PackageStore::new(compile::core());
        let std = store.insert(compile::std(&store, Profile::Unrestricted.into()));
        (store, std)
    };
}

#[wasm_bindgen]
#[must_use]
pub fn git_hash() -> String {
    let git_hash = env!("QSHARP_GIT_HASH");
    git_hash.into()
}

#[wasm_bindgen]
pub fn get_qir(program: ProgramConfig) -> Result<String, String> {
    let (source_map, capabilities, language_features) = into_qsc_args(program, None);

    if capabilities == Profile::Unrestricted.into() {
        return Err("Invalid target profile for QIR generation".to_string());
    }

    _get_qir(source_map, language_features, capabilities)
}

pub(crate) fn _get_qir(
    sources: SourceMap,
    language_features: LanguageFeatures,
    capabilities: TargetCapabilityFlags,
) -> Result<String, String> {
    qsc::codegen::get_qir(sources, language_features, capabilities)
        .map_err(interpret_errors_into_qsharp_errors_json)
}

#[wasm_bindgen]
pub fn get_estimates(program: ProgramConfig, params: &str) -> Result<String, String> {
    let (source_map, capabilities, language_features) = into_qsc_args(program, None);

    let mut interpreter = interpret::Interpreter::new(
        true,
        source_map,
        PackageType::Exe,
        capabilities,
        language_features,
    )
    .map_err(|e| e[0].to_string())?;

    estimate_entry(&mut interpreter, params).map_err(|e| match &e[0] {
        re::Error::Interpreter(interpret::Error::Eval(e)) => e.to_string(),
        re::Error::Interpreter(_) => unreachable!("interpreter errors should be eval errors"),
        re::Error::Estimation(e) => e.to_string(),
    })
}

#[wasm_bindgen]
pub fn get_circuit(
    program: ProgramConfig,
    simulate: bool,
    operation: Option<IOperationInfo>,
) -> Result<JsValue, String> {
    let (source_map, capabilities, language_features) = into_qsc_args(program, None);

    let (package_type, entry_point) = match operation {
        Some(p) => {
            let o: language_service::OperationInfo = p.into();
            // lib package - no need to enforce an entry point since the operation is provided.
            (PackageType::Lib, CircuitEntryPoint::Operation(o.operation))
        }
        None => {
            // exe package - the @EntryPoint attribute will be used.
            (PackageType::Exe, CircuitEntryPoint::EntryPoint)
        }
    };

    let mut interpreter = interpret::Interpreter::new(
        true,
        source_map,
        package_type,
        capabilities,
        LanguageFeatures::from_iter(language_features),
    )
    .map_err(interpret_errors_into_qsharp_errors_json)?;

    let circuit = interpreter
        .circuit(entry_point, simulate)
        .map_err(interpret_errors_into_qsharp_errors_json)?;

    serde_wasm_bindgen::to_value(&circuit).map_err(|e| e.to_string())
}

#[allow(clippy::needless_pass_by_value)]
fn interpret_errors_into_qsharp_errors_json(errs: Vec<qsc::interpret::Error>) -> String {
    serde_json::to_string(&interpret_errors_into_qsharp_errors(&errs))
        .expect("serializing errors to json should succeed")
}

fn interpret_errors_into_qsharp_errors_json_value(
    errors: &[interpret::Error],
) -> serde_json::Value {
    serde_json::to_value(interpret_errors_into_qsharp_errors(errors))
        .expect("json serialization should succeed")
}

#[wasm_bindgen]
#[must_use]
pub fn get_library_source_content(name: &str) -> Option<String> {
    STORE_CORE_STD.with(|(store, std)| {
        for id in [PackageId::CORE, *std] {
            if let Some(source) = store
                .get(id)
                .expect("package should be in store")
                .sources
                .find_by_name(name)
            {
                return Some(source.contents.to_string());
            }
        }

        None
    })
}

#[wasm_bindgen]
pub fn get_ast(
    code: &str,
    language_features: Vec<String>,
    profile: &str,
) -> Result<String, String> {
    let language_features = LanguageFeatures::from_iter(language_features);
    let sources = SourceMap::new([("code".into(), code.into())], None);
    let profile =
        Profile::from_str(profile).map_err(|()| format!("Invalid target profile {profile}"))?;
    let package = STORE_CORE_STD.with(|(store, std)| {
        let (unit, _) = compile::compile(
            store,
            &[*std],
            sources,
            PackageType::Exe,
            profile.into(),
            language_features,
        );
        unit.ast.package
    });
    Ok(format!("{package}"))
}

#[wasm_bindgen]
pub fn get_hir(
    code: &str,
    language_features: Vec<String>,
    profile: &str,
) -> Result<String, String> {
    let language_features = LanguageFeatures::from_iter(language_features);
    let sources = SourceMap::new([("code".into(), code.into())], None);
    let profile =
        Profile::from_str(profile).map_err(|()| format!("Invalid target profile {profile}"))?;
    let package = STORE_CORE_STD.with(|(store, std)| {
        let (unit, _) = compile::compile(
            store,
            &[*std],
            sources,
            PackageType::Exe,
            profile.into(),
            language_features,
        );
        unit.package
    });
    Ok(package.to_string())
}

struct CallbackReceiver<F>
where
    F: FnMut(&str),
{
    event_cb: F,
}

impl<F> Receiver for CallbackReceiver<F>
where
    F: FnMut(&str),
{
    fn state(
        &mut self,
        state: Vec<(BigUint, Complex64)>,
        qubit_count: usize,
    ) -> Result<(), output::Error> {
        let mut dump_json = String::new();
        write!(dump_json, r#"{{"type": "DumpMachine","state": {{"#)
            .expect("writing to string should succeed");
        let (last, most) = state
            .split_last()
            .expect("state should always have at least one entry");
        for state in most {
            write!(
                dump_json,
                r#""{}": [{}, {}],"#,
                format_state_id(&state.0, qubit_count),
                state.1.re,
                state.1.im
            )
            .expect("writing to string should succeed");
        }
        write!(
            dump_json,
            r#""{}": [{}, {}]}}, "#,
            format_state_id(&last.0, qubit_count),
            last.1.re,
            last.1.im
        )
        .expect("writing to string should succeed");

        let json_latex = serde_json::to_string(&get_latex(&state, qubit_count))
            .expect("serialization should succeed");
        write!(dump_json, r#" "stateLatex": {json_latex} }} "#)
            .expect("writing to string should succeed");
        (self.event_cb)(&dump_json);
        Ok(())
    }

    fn message(&mut self, msg: &str) -> Result<(), output::Error> {
        let msg_json = json!({"type": "Message", "message": msg});
        (self.event_cb)(&msg_json.to_string());
        Ok(())
    }
}
fn run_internal_with_features<F>(
    sources: SourceMap,
    event_cb: F,
    shots: u32,
    language_features: LanguageFeatures,
    capabilities: TargetCapabilityFlags,
) -> Result<(), Box<interpret::Error>>
where
    F: FnMut(&str),
{
    let mut out = CallbackReceiver { event_cb };
    let mut interpreter = match interpret::Interpreter::new(
        true,
        sources,
        PackageType::Exe,
        capabilities,
        language_features,
    ) {
        Ok(interpreter) => interpreter,
        Err(err) => {
            // TODO: still wonky as all heck
            let e = err[0].clone();
            let es = interpret_errors_into_qsharp_errors_json(err);
            let msg = json!(
                {"type": "Result", "success": false, "result": es});
            (out.event_cb)(&msg.to_string());
            return Err(Box::new(e));
        }
    };

    for _ in 0..shots {
        let result = interpreter.eval_entry_with_sim(&mut SparseSim::new(), &mut out);
        let mut success = true;
        let msg: serde_json::Value = match result {
            Ok(value) => serde_json::Value::String(value.to_string()),
            Err(errors) => {
                success = false;
                interpret_errors_into_qsharp_errors_json_value(&errors)
            }
        };

        let msg_string = json!({"type": "Result", "success": success, "result": msg}).to_string();
        (out.event_cb)(&msg_string);
    }
    Ok(())
}

#[wasm_bindgen]
pub fn run(
    program: ProgramConfig,
    expr: &str,
    event_cb: &js_sys::Function,
    shots: u32,
) -> Result<bool, JsValue> {
    let (source_map, capabilities, language_features) = into_qsc_args(program, Some(expr.into()));

    if !event_cb.is_function() {
        return Err(JsError::new("Events callback function must be provided").into());
    }

    let event_cb = |msg: &str| {
        // See example at https://rustwasm.github.io/wasm-bindgen/reference/receiving-js-closures-in-rust.html
        let _ = event_cb.call1(&JsValue::null(), &JsValue::from(msg));
    };
    match run_internal_with_features(source_map, event_cb, shots, language_features, capabilities) {
        Ok(()) => Ok(true),
        Err(e) => Err(JsError::from(e).into()),
    }
}

fn check_exercise_solution_internal(
    solution_code: &str,
    exercise_sources: Vec<(SourceName, SourceContents)>,
    event_cb: impl Fn(&str),
) -> bool {
    let source_name = "solution";
    let mut sources = vec![(source_name.into(), solution_code.into())];
    for exercise_source in exercise_sources {
        sources.push(exercise_source);
    }
    let mut out = CallbackReceiver { event_cb };
    let result = check_solution(sources, &mut out);
    let mut runtime_success = true;
    let (exercise_success, msg) = match result {
        Ok(value) => (value, serde_json::Value::String(value.to_string())),
        Err(errors) => {
            runtime_success = false;
            (
                false,
                interpret_errors_into_qsharp_errors_json_value(&errors),
            )
        }
    };
    let msg_string =
        json!({"type": "Result", "success": runtime_success, "result": msg}).to_string();
    (out.event_cb)(&msg_string);
    exercise_success
}

#[wasm_bindgen]
#[must_use]
pub fn check_exercise_solution(
    solution_code: &str,
    exercise_sources_js: JsValue,
    event_cb: &js_sys::Function,
) -> bool {
    let exercise_soruces_strs: Vec<String> = serde_wasm_bindgen::from_value(exercise_sources_js)
        .expect("Deserializing code dependencies should succeed");
    let mut exercise_sources: Vec<(SourceName, SourceContents)> = vec![];
    for (index, code) in exercise_soruces_strs.into_iter().enumerate() {
        exercise_sources.push((index.to_string().into(), code.into()));
    }
    check_exercise_solution_internal(solution_code, exercise_sources, |msg: &str| {
        let _ = event_cb.call1(&JsValue::null(), &JsValue::from_str(msg));
    })
}

serializable_type! {
    DocFile,
    {
        filename: String,
        metadata: String,
        contents: String,
    },
    r#"export interface IDocFile {
        filename: string;
        metadata: string;
        contents: string;
    }"#,
    IDocFile
}

#[wasm_bindgen]
#[must_use]
pub fn generate_docs(additional_program: Option<ProgramConfig>) -> Vec<IDocFile> {
    let docs = if let Some((source_map, capabilities, language_features)) =
        additional_program.map(|p| into_qsc_args(p, None))
    {
        qsc_doc_gen::generate_docs::generate_docs(
            Some(source_map),
            Some(capabilities),
            Some(language_features),
        )
    } else {
        qsc_doc_gen::generate_docs::generate_docs(None, None, None)
    };

    let mut result: Vec<IDocFile> = vec![];

    for (name, metadata, contents) in docs {
        result.push(
            DocFile {
                filename: name.to_string(),
                metadata: metadata.to_string(),
                contents: contents.to_string(),
            }
            .into(),
        );
    }

    result
}

#[wasm_bindgen(typescript_custom_section)]
const TARGET_PROFILE: &'static str = r#"
export type TargetProfile = "base" | "adaptive_ri" | "unrestricted";
"#;

#[wasm_bindgen(typescript_custom_section)]
const LANGUAGE_FEATURES: &'static str = r#"
export type LanguageFeatures = "v2-preview-syntax";
"#;
