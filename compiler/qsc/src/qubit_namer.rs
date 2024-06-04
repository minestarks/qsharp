// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::{
    ast::{Package, Pat, PatKind, Stmt, StmtKind},
    visit::{walk_stmt, Visitor},
};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use std::{
    fmt::{self, Display, Formatter},
    rc::Rc,
};

// TODO: see if we can take advantage of the LocalVarId/Variable mapping
#[derive(Default)]
pub struct QubitTable {
    pub spans: IndexMap<usize, Span>,
    pub arg_sources: IndexMap<usize, String>,
    // a qubit can have multiple assignment sites (qubit reuse after release)
    // and the same assignment site can allocate multiple unique qubits
    //  - e.g. tuples an arrays,  use qs = Qubit[3]; use (a,b) = (Qubit(), Qubit()))
    //  - loops , calls, etc
    pub names: IndexMap<usize, Vec<String>>,
}

impl Display for QubitTable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut qubits = self.spans.iter().peekable();
        while let Some((qubit, span)) = qubits.next() {
            let arg_source = self
                .arg_sources
                .get(qubit)
                .map_or(String::new(), Clone::clone);
            let name = self
                .names
                .get(qubit)
                .map_or(String::new(), |names| names.join(","));
            write!(
                f,
                "Qubit{qubit}: {span} arg_source: {arg_source} name: {name}"
            )?;
            if qubits.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}

pub fn name_qubits(table: &mut QubitTable, package: &Package) {
    let mut namer = QubitNamer { table };
    namer.visit_package(package);
}

struct QubitNamer<'a> {
    table: &'a mut QubitTable,
}

impl QubitNamer<'_> {
    fn get_qubit_ids_within_span(&self, span: Span) -> Vec<usize> {
        let mut qubits = Vec::new();
        for (id, qubit_span) in self.table.spans.iter() {
            if span.lo <= qubit_span.lo && qubit_span.hi <= span.hi {
                qubits.push(id);
            }
        }
        qubits
    }
}

impl<'a, 'b> Visitor<'a> for QubitNamer<'b> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let StmtKind::Qubit(_, pat, _, _) = stmt.kind.as_ref() {
            let assigned_names = name_from_pat(pat);

            let qubit_ids = self.get_qubit_ids_within_span(stmt.span);

            assert!(!assigned_names.is_empty(), "qubit stmt must yield a name");

            let array = qubit_ids.len() > 1 && assigned_names.len() == 1;

            for (qubits_in_the_same_span, id) in qubit_ids.into_iter().enumerate() {
                // make tuples work
                // TODO: there are so many ways to break this
                let name = if array {
                    format!(
                        "{}[{}]",
                        assigned_names
                            .last()
                            .expect("should have at least one name"),
                        qubits_in_the_same_span
                    )
                } else if assigned_names.len() > qubits_in_the_same_span {
                    assigned_names[qubits_in_the_same_span].to_string()
                } else {
                    assigned_names
                        .last()
                        .expect("should have at least one name")
                        .to_string()
                };

                let names = if let Some(names) = self.table.names.get_mut(id) {
                    names
                } else {
                    self.table.names.insert(id, Vec::new());
                    self.table
                        .names
                        .get_mut(id)
                        .expect("just inserted fresh vec")
                };

                names.push(name);
            }
        }

        walk_stmt(self, stmt);
    }

    // fn visit_qubit_init(&mut self, init: &'a QubitInit) {
    //     info!("visiting qubit init {}", init);
    //     let qubit_ids = self.get_qubit_ids_within_span(init.span);
    //     for id in qubit_ids {
    //         self.table.names.insert(id, self.curr_stmt.clone());
    //     }
    // }
}

fn name_from_pat(pat: &Pat) -> Vec<Rc<str>> {
    match pat.kind.as_ref() {
        PatKind::Bind(ident, _) => vec![ident.name.clone()],
        PatKind::Discard(_) => vec!["_".into()],
        PatKind::Elided => vec!["...".into()],
        PatKind::Paren(pat) => name_from_pat(pat),
        PatKind::Tuple(pats) => pats
            .iter()
            .flat_map(|p| match p.kind.as_ref() {
                // TODO: they should be tho
                PatKind::Tuple(_) => panic!("nested tuples not yet supported in qubit init"),
                _ => name_from_pat(p),
            })
            .collect(),
        PatKind::Err => vec!["<error>".into()],
    }
}
