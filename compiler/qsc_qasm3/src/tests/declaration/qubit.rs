// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use miette::Report;

use crate::tests::{fail_on_compilation_errors, parse, qasm_to_program_fragments};
use crate::{
    tests::{compile_qasm_stmt_to_qsharp, compile_qasm_stmt_to_qsharp_with_semantics},
    QubitSemantics,
};

#[test]
fn quantum() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit[6] q1;
        qubit q2;
    ";

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    Ok(())
}

#[test]
fn single_qubit_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit my_qubit;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let my_qubit = QIR.Runtime.__quantum__rt__qubit_allocate();
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn single_qubit_decl_with_qsharp_semantics() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit my_qubit;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp_with_semantics(source, QubitSemantics::QSharp)?;
    expect![
        "
        use my_qubit = Qubit();
    "
    ]
    .assert_eq(&qsharp);
    Ok(())
}
