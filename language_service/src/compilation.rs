// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::state::Configuration;
use log::trace;
use qsc::{
    ast, compile,
    display::Lookup,
    error::WithSource,
    hir::{self, PackageId},
    incremental::Compiler,
    line_column::{Encoding, Position, Range},
    project, resolve,
    target::Profile,
    CompileUnit, PackageStore, PackageType, PassContext, SourceMap, Span,
};
use qsc_linter::{LintConfig, LintLevel};
use std::sync::Arc;

/// Represents an immutable compilation state that can be used
/// to implement language service features.
#[derive(Debug)]
pub(crate) struct Compilation {
    /// The configuration that was used to compile the package.
    pub configuration: Configuration,
    /// Package store, containing the current package and all its dependencies.
    pub package_store: PackageStore,
    /// The `PackageId` of the user package. User code
    /// is non-library code, i.e. all code except the std and core libs.
    pub user_package_id: PackageId,
    pub project_errors: Vec<project::Error>,
    pub compile_errors: Vec<compile::Error>,
    pub kind: CompilationKind,
}

#[derive(Debug)]
pub(crate) enum CompilationKind {
    /// An open Q# project.
    /// In an `OpenProject` compilation, the user package contains
    /// one or more sources, and a target profile.
    OpenProject { dependencies: Vec<PackageId> },
    /// A Q# notebook. In a notebook compilation, the user package
    /// contains multiple `Source`s, with each source corresponding
    /// to a cell.
    Notebook { source_package_id: PackageId },
}

impl Compilation {
    /// Creates a new `Compilation` by compiling sources.
    pub(crate) fn new(
        sources: &[(Arc<str>, Arc<str>)],
        configuration: Configuration,
        project_errors: Vec<project::Error>,
    ) -> Self {
        if sources.len() == 1 {
            trace!("compiling single-file document {}", sources[0].0);
        } else {
            trace!("compiling package with {} sources", sources.len());
        }

        let source_map = SourceMap::new(sources.iter().map(|(x, y)| (x.clone(), y.clone())), None);

        let mut package_store = PackageStore::new(compile::core());
        let std_package_id = package_store.insert(compile::std(
            &package_store,
            configuration.target_profile.into(),
        ));

        let dependencies = vec![std_package_id];

        let (unit, mut compile_errors) = compile::compile(
            &package_store,
            &dependencies,
            source_map,
            configuration.package_type,
            configuration.target_profile.into(),
            configuration.language_features,
        );

        let user_package_id = package_store.insert(unit);

        run_expensive_analysis(
            &mut compile_errors,
            configuration.target_profile,
            &package_store,
            user_package_id,
            &configuration.lints_config,
        );

        Self {
            package_store,
            user_package_id,
            compile_errors,
            kind: CompilationKind::OpenProject { dependencies },
            configuration,
            project_errors,
        }
    }

    /// Creates a new `Compilation` by compiling sources from notebook cells.
    pub(crate) fn new_notebook<I>(cells: I, configuration: Configuration) -> Self
    where
        I: Iterator<Item = (Arc<str>, Arc<str>)>,
    {
        trace!("compiling notebook");
        let mut compiler = Compiler::new(
            true,
            SourceMap::default(),
            PackageType::Lib,
            configuration.target_profile.into(),
            configuration.language_features,
        )
        .expect("expected incremental compiler creation to succeed");

        let mut errors = Vec::new();
        for (name, contents) in cells {
            trace!("compiling cell {name}");
            let increment = compiler
                .compile_fragments(&name, &contents, |cell_errors| {
                    errors.extend(cell_errors);
                    Ok(()) // accumulate errors without failing
                })
                .expect("compile_fragments_acc_errors should not fail");

            compiler.update(increment);
        }

        let (package_store, source_package_id, user_package_id) = compiler.into_package_store();

        run_expensive_analysis(
            &mut errors,
            configuration.target_profile,
            &package_store,
            user_package_id,
            &configuration.lints_config,
        );

        Self {
            package_store,
            user_package_id,
            compile_errors: errors,
            kind: CompilationKind::Notebook { source_package_id },
            configuration,
            project_errors: Vec::new(),
        }
    }
    pub(crate) fn update(
        mut self,
        sources: &[(Arc<str>, Arc<str>)],
        configuration: Configuration,
    ) -> Self {
        if configuration != self.configuration {
            return Compilation::new(sources, configuration, self.project_errors);
        }

        let source_map = SourceMap::new(sources.iter().map(|(x, y)| (x.clone(), y.clone())), None);

        self.package_store.remove(self.user_package_id);

        if let CompilationKind::OpenProject { dependencies } = &self.kind {
            let (unit, mut errors) = compile::compile(
                &self.package_store,
                dependencies,
                source_map,
                configuration.package_type,
                configuration.target_profile.into(),
                configuration.language_features,
            );

            self.user_package_id = self.package_store.insert(unit);

            run_expensive_analysis(
                &mut errors,
                configuration.target_profile,
                &self.package_store,
                self.user_package_id,
                &configuration.lints_config,
            );

            self.compile_errors = errors;

            self
        } else {
            panic!("expected to find open project");
        }
    }

    pub(crate) fn update_notebook<I>(mut self, cells: I, configuration: Configuration) -> Self
    where
        I: Iterator<Item = (Arc<str>, Arc<str>)>,
    {
        if configuration != self.configuration {
            return Compilation::new_notebook(cells, configuration);
        }

        self.package_store.remove(self.user_package_id);

        if let CompilationKind::Notebook { source_package_id } = &self.kind {
            let mut compiler = Compiler::from(
                self.package_store,
                *source_package_id,
                configuration.target_profile.into(),
                configuration.language_features,
            );

            let mut errors = Vec::new();
            for (name, contents) in cells {
                trace!("compiling cell {name}");
                let increment = compiler
                    .compile_fragments(&name, &contents, |cell_errors| {
                        errors.extend(cell_errors);
                        Ok(()) // accumulate errors without failing
                    })
                    .expect("compile_fragments_acc_errors should not fail");

                compiler.update(increment);
            }

            let (package_store, source_package_id, user_package_id) = compiler.into_package_store();

            run_expensive_analysis(
                &mut errors,
                configuration.target_profile,
                &package_store,
                user_package_id,
                &configuration.lints_config,
            );

            Self {
                package_store,
                user_package_id,
                compile_errors: errors,
                project_errors: Vec::new(),
                configuration,
                kind: CompilationKind::Notebook { source_package_id },
            }
        } else {
            panic!("expected to find notebook");
        }
    }

    /// Analyzes the package and pushes errors.
    /// Performs RCA nd lint passes.
    pub fn run_expensive_analysis(&mut self, target_profile: Profile, lints_config: &[LintConfig]) {
        let unit = self
            .package_store
            .get(self.user_package_id)
            .expect("expected to find user package");

        if !self.project_errors.is_empty() {
            return;
        }

        run_fir_passes(
            &mut self.compile_errors,
            target_profile,
            &self.package_store,
            self.user_package_id,
            unit,
        );

        run_linter_passes(
            &mut self.compile_errors,
            &self.package_store,
            unit,
            lints_config,
        );
    }

    /// Gets the `CompileUnit` associated with user (non-library) code.
    pub fn user_unit(&self) -> &CompileUnit {
        self.package_store
            .get(self.user_package_id)
            .expect("expected to find user package")
    }

    /// Maps a source position from the user package
    /// to a package (`SourceMap`) offset.
    pub(crate) fn source_position_to_package_offset(
        &self,
        source_name: &str,
        source_position: Position,
        position_encoding: Encoding,
    ) -> u32 {
        let unit = self.user_unit();

        let source = unit
            .sources
            .find_by_name(source_name)
            .expect("source should exist in the user source map");

        let mut offset =
            source_position.to_utf8_byte_offset(position_encoding, source.contents.as_ref());

        let len = u32::try_from(source.contents.len()).expect("source length should fit into u32");
        if offset > len {
            // This can happen if the document contents are out of sync with the client's view.
            // we don't want to accidentally return an offset into the next file -
            // remap to the end of the current file.
            trace!(
                "offset {offset} out of bounds for {}, using end offset instead",
                source.name
            );
            offset = len;
        }

        source.offset + offset
    }

    pub(crate) fn source_range_to_package_span(
        &self,
        source_name: &str,
        source_range: Range,
        position_encoding: Encoding,
    ) -> Span {
        let lo = self.source_position_to_package_offset(
            source_name,
            source_range.start,
            position_encoding,
        );
        let hi = self.source_position_to_package_offset(
            source_name,
            source_range.end,
            position_encoding,
        );
        Span { lo, hi }
    }

    /// Gets the span of the whole source file.
    pub(crate) fn package_span_of_source(&self, source_name: &str) -> Span {
        let unit = self.user_unit();

        let source = unit
            .sources
            .find_by_name(source_name)
            .expect("source should exist in the user source map");

        let len = u32::try_from(source.contents.len()).expect("source length should fit into u32");

        Span {
            lo: source.offset,
            hi: source.offset + len,
        }
    }

    /// Regenerates the compilation with the same sources but the passed in workspace configuration options.
    pub fn recompile(&mut self, configuration: Configuration) {
        let sources = self
            .user_unit()
            .sources
            .iter()
            .map(|source| (source.name.clone(), source.contents.clone()));

        let new = match self.kind {
            CompilationKind::OpenProject { .. } => Self::new(
                &sources.collect::<Vec<_>>(),
                configuration,
                Vec::new(), // project errors will stay the same
            ),
            CompilationKind::Notebook { .. } => Self::new_notebook(sources, configuration),
        };

        self.package_store = new.package_store;
        self.user_package_id = new.user_package_id;
        self.compile_errors = new.compile_errors;
        self.configuration = new.configuration;
    }
}

/// Analyzes the package and pushes errors.
/// Performs RCA nd lint passes.
/// These should only be performed if there are no errors in the compilation.
fn run_expensive_analysis(
    errors: &mut Vec<WithSource<compile::ErrorKind>>,
    target_profile: Profile,
    package_store: &PackageStore,
    user_package_id: PackageId,
    lints_config: &[LintConfig],
) {
    let unit = package_store
        .get(user_package_id)
        .expect("expected to find user package");

    run_fir_passes(errors, target_profile, package_store, user_package_id, unit);

    run_linter_passes(errors, package_store, unit, lints_config);
}

/// Runs the passes required for code generation
/// appending any errors to the `errors` vector.
/// This function only runs passes if there are no compile
/// errors in the package and if the target profile is not `Base`
/// or `Unrestricted`.
fn run_fir_passes(
    errors: &mut Vec<WithSource<compile::ErrorKind>>,
    target_profile: Profile,
    package_store: &PackageStore,
    package_id: PackageId,
    unit: &CompileUnit,
) {
    if !errors.is_empty() {
        // can't run passes on a package with errors
        return;
    }

    if target_profile == Profile::Unrestricted {
        // no point in running passes on unrestricted profile
        return;
    }

    let (fir_store, fir_package_id) = qsc::lower_hir_to_fir(package_store, package_id);
    let caps_results =
        PassContext::run_fir_passes_on_fir(&fir_store, fir_package_id, target_profile.into());
    if let Err(caps_errors) = caps_results {
        for err in caps_errors {
            let err = WithSource::from_map(&unit.sources, compile::ErrorKind::Pass(err));
            errors.push(err);
        }
    }
}

/// Compute new lints and append them to the errors Vec.
/// Lints are only computed if the errors vector is empty. For performance
/// reasons we don't want to waste time running lints every few keystrokes,
/// if the user is in the middle of typing a statement, for example.
fn run_linter_passes(
    errors: &mut Vec<WithSource<compile::ErrorKind>>,
    package_store: &PackageStore,
    unit: &CompileUnit,
    config: &[LintConfig],
) {
    if errors.is_empty() {
        let lints = qsc::linter::run_lints(package_store, unit, Some(config));
        let lints = lints
            .into_iter()
            .filter(|lint| !matches!(lint.level, LintLevel::Allow))
            .map(|lint| WithSource::from_map(&unit.sources, qsc::compile::ErrorKind::Lint(lint)));
        errors.extend(lints);
    }
}

impl Lookup for Compilation {
    /// Looks up the type of a node in user code
    fn get_ty(&self, id: ast::NodeId) -> Option<&hir::ty::Ty> {
        self.user_unit().ast.tys.terms.get(id)
    }

    /// Looks up the resolution of a node in user code
    fn get_res(&self, id: ast::NodeId) -> Option<&resolve::Res> {
        self.user_unit().ast.names.get(id)
    }

    /// Returns the hir `Item` node referred to by `item_id`,
    /// along with the `Package` and `PackageId` for the package
    /// that it was found in.
    fn resolve_item_relative_to_user_package(
        &self,
        item_id: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId) {
        self.resolve_item(self.user_package_id, item_id)
    }

    /// Returns the hir `Item` node referred to by `res`.
    /// `Res`s can resolve to external packages, and the references
    /// are relative, so here we also need the
    /// local `PackageId` that the `res` itself came from.
    fn resolve_item_res(
        &self,
        local_package_id: PackageId,
        res: &hir::Res,
    ) -> (&hir::Item, hir::ItemId) {
        match res {
            hir::Res::Item(item_id) => {
                let (item, _, resolved_item_id) = self.resolve_item(local_package_id, item_id);
                (item, resolved_item_id)
            }
            _ => panic!("expected to find item"),
        }
    }

    /// Returns the hir `Item` node referred to by `item_id`.
    /// `ItemId`s can refer to external packages, and the references
    /// are relative, so here we also need the local `PackageId`
    /// that the `ItemId` originates from.
    fn resolve_item(
        &self,
        local_package_id: PackageId,
        item_id: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId) {
        // If the `ItemId` contains a package id, use that.
        // Lack of a package id means the item is in the
        // same package as the one this `ItemId` reference
        // came from. So use the local package id passed in.
        let package_id = item_id.package.unwrap_or(local_package_id);
        let package = &self
            .package_store
            .get(package_id)
            .expect("package should exist in store")
            .package;
        (
            package
                .items
                .get(item_id.item)
                .expect("item id should exist"),
            package,
            hir::ItemId {
                package: Some(package_id),
                item: item_id.item,
            },
        )
    }
}
