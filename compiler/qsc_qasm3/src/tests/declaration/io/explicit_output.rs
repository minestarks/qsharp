// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp_operation;
use expect_test::expect;
use miette::Report;

#[test]
fn bit_array_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output bit[2] c;
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
fn bit_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output bit c;
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
fn bool_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output bool c;
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
fn complex_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output complex[float] c;
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
fn float_implicit_width_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output float f;
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
fn float_explicit_width_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output float[42] f;
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
fn int_explicit_width_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output int[42] i;
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
fn int_implicit_width_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output int i;
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
fn uint_implicit_width_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output uint i;
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
fn uint_explicit_width_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output uint[42] i;
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
fn bigint_explicit_width_is_returned() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output int[65] i;
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
fn qubit_explicit_output_raises_parse_error() {
    let source = r#"
output qubit q;
"#;

    let Err(error) = compile_qasm_to_qsharp_operation(source) else {
        panic!("Expected error")
    };

    assert!(error[0]
        .to_string()
        .contains("QASM3 Parse Error: Quantum type found in input/output declaration."));
}

#[test]
fn order_is_preserved_with_multiple_inputs() -> miette::Result<(), Vec<Report>> {
    let source = r#"
output int[65] bi;
output int[6] i;
output uint[60] ui;
output uint u;
output float f;
output bool b;
output bit c;
output complex[float] cf;
output bit[2] b2;
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
