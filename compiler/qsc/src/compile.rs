// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log::warn;
use miette::{Diagnostic, Report};
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, SourceMap},
    error::WithSource,
};
use qsc_hir::hir::PackageId;
use qsc_passes::{run_core_passes, run_default_passes, PackageType};
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
    process_compile_unit(store, package_type, capabilities, unit)
}

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
    process_compile_unit(store, package_type, capabilities, unit)
}

#[must_use]
#[allow(clippy::module_name_repetitions)]
fn process_compile_unit(
    store: &PackageStore,
    package_type: PackageType,
    capabilities: TargetCapabilityFlags,
    mut unit: CompileUnit,
) -> (CompileUnit, Vec<Error>) {
    let mut errors = Vec::new();
    for error in unit.errors.drain(..) {
        errors.push(WithSource::from_map(&unit.sources, error.into()));
    }

    if errors.is_empty() {
        for error in run_default_passes(store.core(), &mut unit, package_type, capabilities) {
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

/// Compiles the standard library.
///
/// # Panics
///
/// Panics if the standard library does not compile without errors.
#[must_use]
pub fn std(store: &PackageStore, capabilities: TargetCapabilityFlags) -> CompileUnit {
    let mut unit = qsc_frontend::compile::std(store, capabilities);
    let pass_errors = run_default_passes(store.core(), &mut unit, PackageType::Lib, capabilities);
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
