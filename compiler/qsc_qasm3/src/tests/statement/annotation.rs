// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn simulatable_intrinsic_can_be_applied_to_gate() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        @SimulatableIntrinsic
        gate my_h q {
            h q;
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        @SimulatableIntrinsic()
        operation my_h(q : Qubit) : Unit {
            H(q);
        }
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn unknown_annotation_raises_error() {
    let source = r#"
        include "stdgates.inc";
        @SomeUnknownAnnotation
        gate my_h q {
            h q;
        }
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected an error");
    };
    expect![r#"Unexpected annotation: @SomeUnknownAnnotation."#].assert_eq(&errors[0].to_string());
}

#[test]
fn annotation_without_target_in_global_scope_raises_error() {
    let source = r#"
        int i;
        @SimulatableIntrinsic
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected an error");
    };
    expect![r#"Annotation missing target statement."#].assert_eq(&errors[0].to_string());
}

#[test]
fn annotation_without_target_in_block_scope_raises_error() {
    let source = r#"
        int i;
        if (0 == 1) {
            @SimulatableIntrinsic
        }
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected an error");
    };
    expect![r#"Annotation missing target statement."#].assert_eq(&errors[0].to_string());
}
