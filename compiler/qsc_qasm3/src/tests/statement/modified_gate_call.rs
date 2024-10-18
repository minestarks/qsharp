// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::compile_qasm_to_qsharp;
use expect_test::expect;
use miette::Report;

#[test]
fn adj_x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        inv @ x q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        Adjoint X(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn adj_adj_x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        inv @ inv @ x q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        Adjoint Adjoint X(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn multiple_controls_on_x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[3] q;
        qubit f;
        ctrl(3) @ x q[1], q[0], q[2], f;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(3);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled X([q[1], q[0], q[2]], f);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn repeated_multi_controls_on_x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        qubit[3] r;
        qubit f;
        ctrl(2) @ ctrl(3) @ x q[1], r[0], q[0], f, r[1], r[2];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(2);
        let r = QIR.Runtime.AllocateQubitArray(3);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled Controlled X([q[1], r[0]], ([q[0], f, r[1]], r[2]));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn repeated_multi_controls_on_x_gate_can_be_mixed_with_inv() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        qubit[3] r;
        qubit f;
        ctrl(2) @ inv @ ctrl(3) @ inv @ x q[1], r[0], q[0], f, r[1], r[2];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(2);
        let r = QIR.Runtime.AllocateQubitArray(3);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled Adjoint Controlled Adjoint X([q[1], r[0]], ([q[0], f, r[1]], r[2]));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn multiple_controls_on_cx_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[4] q;
        qubit f;
        ctrl(3) @ cx q[1], q[0], q[2], f, q[3];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(4);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled CNOT([q[1], q[0], q[2]], (f, q[3]));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn multiple_controls_on_crx_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[4] q;
        qubit f;
        ctrl(3) @ inv @ crx(0.5) q[1], q[0], q[2], f, q[3];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(4);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Controlled Adjoint Controlled Rx([q[1], q[0], q[2]], ([f], (0.5, q[3])));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn neg_ctrl_can_be_applied_and_wrapped_in_another_modifier() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[4] q;
        qubit f;
        inv @ negctrl(3) @ inv @ crx(0.5) q[1], q[0], q[2], f, q[3];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(4);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        Adjoint ApplyControlledOnInt(0, Adjoint Controlled Rx, [q[1], q[0], q[2]], ([f], (0.5, q[3])));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn neg_ctrl_can_wrap_another_neg_crtl_modifier() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[6] q;
        qubit f;
        negctrl(3) @ negctrl(2) @ crx(0.5) q[1], q[0], q[2], q[3], q[4], f, q[5];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.AllocateQubitArray(6);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        ApplyControlledOnInt(0, ApplyControlledOnInt, [q[1], q[0], q[2]], (0, Controlled Rx, [q[3], q[4]], ([f], (0.5, q[5]))));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn modifiers_can_be_repeated_many_times() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[6] q;
        qubit f;
        pow(1) @ pow(1) @ pow(1) @ crx(0.5) f, q[5];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        operation __Pow__ < 'T > (N : Int, op : ('T => Unit is Adj), target : 'T) : Unit is Adj {
            let op = if N > 0 {
                () => op(target)
            } else {
                () => Adjoint op(target)
            };
            for _ in 1..Microsoft.Quantum.Math.AbsI(N) {
                op()
            }
        }
        let q = QIR.Runtime.AllocateQubitArray(6);
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        __Pow__(1, __Pow__, (1, __Pow__, (1, Controlled Rx, ([f], (0.5, q[5])))));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn pow_can_be_applied_on_a_simple_gate() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit f;
        pow(2) @ x f;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        operation __Pow__ < 'T > (N : Int, op : ('T => Unit is Adj), target : 'T) : Unit is Adj {
            let op = if N > 0 {
                () => op(target)
            } else {
                () => Adjoint op(target)
            };
            for _ in 1..Microsoft.Quantum.Math.AbsI(N) {
                op()
            }
        }
        let f = QIR.Runtime.__quantum__rt__qubit_allocate();
        __Pow__(2, X, (f));
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
