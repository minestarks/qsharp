// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log::warn;
use miette::{Diagnostic, Report};
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, SourceMap},
    error::WithSource,
};
use qsc_hir::{global, hir::PackageId};
use qsc_passes::{run_core_passes, run_default_passes, PackageType};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};
use thiserror::Error;

pub type Error = WithSource<ErrorKind>;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
/// `ErrorKind` represents the different kinds of errors that can occur in the compiler.
/// Each variant of the enum corresponds to a different stage of the compilation process.
pub enum ErrorKind {
    /// `Frontend` variant represents errors that occur during the frontend stage of the compiler.
    /// These errors are typically related to syntax and semantic checks.
    Frontend(#[from] qsc_frontend::compile::Error),

    /// `Pass` variant represents errors that occur during the `qsc_passes` stage of the compiler.
    /// These errors are typically related to optimization, transformation, code generation, passes,
    /// and static analysis passes.
    Pass(#[from] qsc_passes::Error),

    /// `Lint` variant represents lints generated during the linting stage. These diagnostics are
    /// typically emited from the language server and happens after all other compilation passes.
    Lint(#[from] qsc_linter::Lint),
}

/// Compiles a package from its AST representation.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn compile_ast(
    store: &PackageStore,
    dependencies: &[PackageId],
    ast_package: qsc_ast::ast::Package,
    sources: SourceMap,
    package_type: PackageType,
    capabilities: TargetCapabilityFlags,
) -> (CompileUnit, Vec<Error>) {
    let unit = qsc_frontend::compile::compile_ast(
        store,
        dependencies,
        ast_package,
        sources,
        capabilities,
        vec![],
    );
    process_compile_unit(store, package_type, unit)
}

/// Compiles a package from its source representation.
#[must_use]
pub fn compile(
    store: &PackageStore,
    dependencies: &[PackageId],
    sources: SourceMap,
    package_type: PackageType,
    capabilities: TargetCapabilityFlags,
    language_features: LanguageFeatures,
) -> (CompileUnit, Vec<Error>) {
    let unit = qsc_frontend::compile::compile(
        store,
        dependencies,
        sources,
        capabilities,
        language_features,
    );
    process_compile_unit(store, package_type, unit)
}

#[must_use]
#[allow(clippy::module_name_repetitions)]
fn process_compile_unit(
    store: &PackageStore,
    package_type: PackageType,
    mut unit: CompileUnit,
) -> (CompileUnit, Vec<Error>) {
    let mut errors = Vec::new();
    for error in unit.errors.drain(..) {
        errors.push(WithSource::from_map(&unit.sources, error.into()));
    }

    if errors.is_empty() {
        for error in run_default_passes(store.core(), &mut unit, package_type) {
            errors.push(WithSource::from_map(&unit.sources, error.into()));
        }
    }

    if !errors.is_empty() {
        warn!("errors in compilation: {errors:?}");
    }

    (unit, errors)
}

/// Compiles the core library.
///
/// # Panics
///
/// Panics if the core library does not compile without errors.
#[must_use]
pub fn core() -> CompileUnit {
    let mut unit = qsc_frontend::compile::core();
    let pass_errors = run_core_passes(&mut unit);
    if pass_errors.is_empty() {
        unit
    } else {
        for error in pass_errors {
            let report = Report::new(WithSource::from_map(&unit.sources, error));
            eprintln!("{report:?}");
        }

        panic!("could not compile core library")
    }
}

thread_local! {
    static CORE: (Rc<CompileUnit>, Rc<global::Table>) = {
        let core = Rc::new(core());
        let table = Rc::new(global::iter_package(Some(PackageId::CORE), &core.package).collect());
        (core,table)
    };

    static STD: RefCell<FxHashMap<TargetCapabilityFlags, Rc<CompileUnit>>> = RefCell::default();
}

#[must_use]
fn cached_core() -> (Rc<CompileUnit>, Rc<global::Table>) {
    CORE.with(|(unit, table)| (unit.clone(), table.clone()))
}

#[must_use]
fn cached_std(store: &PackageStore, capabilities: TargetCapabilityFlags) -> Rc<CompileUnit> {
    STD.with(|cache| {
        cache
            .borrow_mut()
            .entry(capabilities)
            .or_insert_with(|| Rc::new(std(store, capabilities)))
            .clone()
    })
}

#[must_use]
pub fn new_std_core(capabilities: TargetCapabilityFlags) -> (PackageStore, PackageId) {
    let (core, table) = cached_core();
    let mut package_store = PackageStore::with_cached_core(core, table);
    let std_package_id = package_store.cached_insert(cached_std(&package_store, capabilities));
    (package_store, std_package_id)
}

/// Compiles the standard library.
///
/// # Panics
///
/// Panics if the standard library does not compile without errors.
#[must_use]
pub fn std(store: &PackageStore, capabilities: TargetCapabilityFlags) -> CompileUnit {
    let mut unit = qsc_frontend::compile::std(store, capabilities);
    let pass_errors = run_default_passes(store.core(), &mut unit, PackageType::Lib);
    if pass_errors.is_empty() {
        unit
    } else {
        for error in pass_errors {
            let report = Report::new(WithSource::from_map(&unit.sources, error));
            eprintln!("{report:?}");
        }

        panic!("could not compile standard library")
    }
}
