// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp_operation;
use expect_test::expect;
use miette::Report;

#[test]
fn bit_array_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
bit[2] c;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Result[] {
    mutable c = [Zero, Zero];
    c
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bit_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
bit c;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Result {
    mutable c = Zero;
    c
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bool_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
bool c;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Bool {
    mutable c = false;
    c
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn complex_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
complex[float] c;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Microsoft.Quantum.Math.Complex {
    mutable c = Microsoft.Quantum.Math.Complex(0., 0.);
    c
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn float_implicit_width_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
float f;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Double {
    mutable f = 0.;
    f
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn float_explicit_width_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
float[42] f;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Double {
    mutable f = 0.;
    f
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn int_explicit_width_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
int[42] i;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Int {
    mutable i = 0;
    i
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn int_implicit_width_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
int i;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Int {
    mutable i = 0;
    i
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn uint_implicit_width_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
uint i;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Int {
    mutable i = 0;
    i
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn uint_explicit_width_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
uint[42] i;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : Int {
    mutable i = 0;
    i
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bigint_explicit_width_is_inferred_and_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
int[65] i;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : BigInt {
    mutable i = 0L;
    i
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn order_is_preserved_with_multiple_inputs() -> miette::Result<(), Vec<Report>> {
    let source = r#"
int[65] bi;
int[6] i;
uint[60] ui;
uint u;
float f;
bool b;
bit c;
complex[float] cf;
bit[2] b2;
"#;

    let qsharp = compile_qasm_to_qsharp_operation(source)?;
    expect![
        r#"
@EntryPoint()
operation Test() : (BigInt, Int, Int, Int, Double, Bool, Result, Microsoft.Quantum.Math.Complex, Result[]) {
    mutable bi = 0L;
    mutable i = 0;
    mutable ui = 0;
    mutable u = 0;
    mutable f = 0.;
    mutable b = false;
    mutable c = Zero;
    mutable cf = Microsoft.Quantum.Math.Complex(0., 0.);
    mutable b2 = [Zero, Zero];
    (bi, i, ui, u, f, b, c, cf, b2)
}
"#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
