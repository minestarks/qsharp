// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::tests_common::{
    check, check_for_exe, CALL_DYNAMIC_FUNCTION, CALL_DYNAMIC_OPERATION,
    CALL_TO_CYCLIC_FUNCTION_WITH_CLASSICAL_ARGUMENT, CALL_TO_CYCLIC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
    CALL_TO_CYCLIC_OPERATION_WITH_CLASSICAL_ARGUMENT,
    CALL_TO_CYCLIC_OPERATION_WITH_DYNAMIC_ARGUMENT, CALL_UNRESOLVED_FUNCTION,
    LOOP_WITH_DYNAMIC_CONDITION, MEASUREMENT_WITHIN_DYNAMIC_SCOPE, MINIMAL,
    RETURN_WITHIN_DYNAMIC_SCOPE, USE_CLOSURE_FUNCTION, USE_DYNAMICALLY_SIZED_ARRAY,
    USE_DYNAMIC_BIG_INT, USE_DYNAMIC_BOOLEAN, USE_DYNAMIC_DOUBLE, USE_DYNAMIC_FUNCTION,
    USE_DYNAMIC_INDEX, USE_DYNAMIC_INT, USE_DYNAMIC_OPERATION, USE_DYNAMIC_PAULI,
    USE_DYNAMIC_QUBIT, USE_DYNAMIC_RANGE, USE_DYNAMIC_STRING, USE_DYNAMIC_UDT,
    USE_ENTRY_POINT_INT_ARRAY_IN_TUPLE, USE_ENTRY_POINT_STATIC_BIG_INT,
    USE_ENTRY_POINT_STATIC_BOOL, USE_ENTRY_POINT_STATIC_DOUBLE, USE_ENTRY_POINT_STATIC_INT,
    USE_ENTRY_POINT_STATIC_INT_IN_TUPLE, USE_ENTRY_POINT_STATIC_PAULI,
    USE_ENTRY_POINT_STATIC_RANGE, USE_ENTRY_POINT_STATIC_STRING,
};
use expect_test::{expect, Expect};
use qsc_data_structures::target::TargetCapabilityFlags;

fn check_profile(source: &str, expect: &Expect) {
    check(source, expect, TargetCapabilityFlags::empty());
}

fn check_profile_for_exe(source: &str, expect: &Expect) {
    check_for_exe(source, expect, TargetCapabilityFlags::empty());
}

#[test]
fn minimal_program_yields_no_errors() {
    check_profile(
        MINIMAL,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn use_of_dynamic_boolean_yields_error() {
    check_profile(
        USE_DYNAMIC_BOOLEAN,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 104,
                        hi: 116,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_int_yields_errors() {
    check_profile(
        USE_DYNAMIC_INT,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 246,
                        hi: 271,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 246,
                        hi: 271,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_pauli_yields_errors() {
    // In the case of if expressions, if either the condition or the blocks yield errors, the errors yielded by the
    // whole if expression are not surfaced to avoid too error churn.
    // For this reason, the "use of dynamic pauli" error is not yielded in this case.
    check_profile(
        USE_DYNAMIC_PAULI,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 104,
                        hi: 116,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_range_yields_errors() {
    check_profile(
        USE_DYNAMIC_RANGE,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 108,
                        hi: 137,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 108,
                        hi: 137,
                    },
                ),
                UseOfDynamicRange(
                    Span {
                        lo: 108,
                        hi: 137,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_double_yields_errors() {
    check_profile(
        USE_DYNAMIC_DOUBLE,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 246,
                        hi: 284,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 246,
                        hi: 284,
                    },
                ),
                UseOfDynamicDouble(
                    Span {
                        lo: 246,
                        hi: 284,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_qubit_yields_errors() {
    check_profile(
        USE_DYNAMIC_QUBIT,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 105,
                        hi: 123,
                    },
                ),
                UseOfDynamicQubit(
                    Span {
                        lo: 146,
                        hi: 162,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_big_int_yields_errors() {
    check_profile(
        USE_DYNAMIC_BIG_INT,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 247,
                        hi: 285,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 247,
                        hi: 285,
                    },
                ),
                UseOfDynamicBigInt(
                    Span {
                        lo: 247,
                        hi: 285,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_string_yields_errors() {
    check_profile(
        USE_DYNAMIC_STRING,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 130,
                        hi: 144,
                    },
                ),
                UseOfDynamicString(
                    Span {
                        lo: 130,
                        hi: 144,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamically_sized_array_yields_errors() {
    check_profile(
        USE_DYNAMICALLY_SIZED_ARRAY,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 104,
                        hi: 136,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 104,
                        hi: 136,
                    },
                ),
                UseOfDynamicallySizedArray(
                    Span {
                        lo: 104,
                        hi: 136,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_udt_yields_errors() {
    check_profile(
        USE_DYNAMIC_UDT,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 283,
                        hi: 335,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 283,
                        hi: 335,
                    },
                ),
                UseOfDynamicDouble(
                    Span {
                        lo: 283,
                        hi: 335,
                    },
                ),
                UseOfDynamicUdt(
                    Span {
                        lo: 283,
                        hi: 335,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_function_yields_errors() {
    // In the case of if expressions, if either the condition or the blocks yield errors, the errors yielded by the
    // whole if expression are not surfaced to avoid too much error churn.
    // For this reason, the "use of dynamic function" error is not yielded in this case.
    check_profile(
        USE_DYNAMIC_FUNCTION,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 142,
                        hi: 154,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_operation_yields_errors() {
    // In the case of if expressions, if either the condition or the blocks yield errors, the errors yielded by the
    // whole if expression are not surfaced to avoid too much error churn.
    // For this reason, the "use of dynamic operation" error is not yielded in this case.
    check_profile(
        USE_DYNAMIC_OPERATION,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 142,
                        hi: 154,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_cyclic_function_with_classical_argument_yields_no_errors() {
    check_profile(
        CALL_TO_CYCLIC_FUNCTION_WITH_CLASSICAL_ARGUMENT,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn call_cyclic_function_with_dynamic_argument_yields_errors() {
    check_profile(
        CALL_TO_CYCLIC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 211,
                        hi: 243,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 211,
                        hi: 243,
                    },
                ),
                CallToCyclicFunctionWithDynamicArg(
                    Span {
                        lo: 211,
                        hi: 243,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_cyclic_operation_with_classical_argument_yields_errors() {
    check_profile(
        CALL_TO_CYCLIC_OPERATION_WITH_CLASSICAL_ARGUMENT,
        &expect![[r#"
            [
                CyclicOperationSpec(
                    Span {
                        lo: 15,
                        hi: 23,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 187,
                        hi: 199,
                    },
                ),
                CallToCyclicOperation(
                    Span {
                        lo: 187,
                        hi: 199,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_cyclic_operation_with_dynamic_argument_yields_errors() {
    check_profile(
        CALL_TO_CYCLIC_OPERATION_WITH_DYNAMIC_ARGUMENT,
        &expect![[r#"
            [
                CyclicOperationSpec(
                    Span {
                        lo: 15,
                        hi: 23,
                    },
                ),
                UseOfDynamicBool(
                    Span {
                        lo: 212,
                        hi: 244,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 212,
                        hi: 244,
                    },
                ),
                CallToCyclicOperation(
                    Span {
                        lo: 212,
                        hi: 244,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_to_dynamic_function_yields_errors() {
    check_profile(
        CALL_DYNAMIC_FUNCTION,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 142,
                        hi: 154,
                    },
                ),
                UseOfDynamicBool(
                    Span {
                        lo: 180,
                        hi: 188,
                    },
                ),
                UseOfDynamicDouble(
                    Span {
                        lo: 180,
                        hi: 188,
                    },
                ),
                UseOfDynamicArrowFunction(
                    Span {
                        lo: 180,
                        hi: 188,
                    },
                ),
                CallToDynamicCallee(
                    Span {
                        lo: 180,
                        hi: 188,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_to_dynamic_operation_yields_errors() {
    check_profile(
        CALL_DYNAMIC_OPERATION,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 142,
                        hi: 154,
                    },
                ),
                UseOfDynamicBool(
                    Span {
                        lo: 176,
                        hi: 181,
                    },
                ),
                UseOfDynamicArrowOperation(
                    Span {
                        lo: 176,
                        hi: 181,
                    },
                ),
                CallToDynamicCallee(
                    Span {
                        lo: 176,
                        hi: 181,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_to_unresolved_allowed() {
    check_profile(
        CALL_UNRESOLVED_FUNCTION,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn measurement_within_dynamic_scope_yields_errors() {
    check_profile(
        MEASUREMENT_WITHIN_DYNAMIC_SCOPE,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 99,
                        hi: 110,
                    },
                ),
                MeasurementWithinDynamicScope(
                    Span {
                        lo: 137,
                        hi: 141,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_index_yields_errors() {
    check_profile(
        USE_DYNAMIC_INDEX,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 246,
                        hi: 271,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 246,
                        hi: 271,
                    },
                ),
                UseOfDynamicBool(
                    Span {
                        lo: 319,
                        hi: 323,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 319,
                        hi: 323,
                    },
                ),
                UseOfDynamicIndex(
                    Span {
                        lo: 319,
                        hi: 323,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn return_within_dynamic_scope_yields_errors() {
    check_profile(
        RETURN_WITHIN_DYNAMIC_SCOPE,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 98,
                        hi: 109,
                    },
                ),
                ReturnWithinDynamicScope(
                    Span {
                        lo: 128,
                        hi: 136,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn loop_with_dynamic_condition_yields_errors() {
    check_profile(
        LOOP_WITH_DYNAMIC_CONDITION,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 106,
                        hi: 118,
                    },
                ),
                UseOfDynamicBool(
                    Span {
                        lo: 141,
                        hi: 159,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 141,
                        hi: 159,
                    },
                ),
                UseOfDynamicRange(
                    Span {
                        lo: 141,
                        hi: 159,
                    },
                ),
                LoopWithDynamicCondition(
                    Span {
                        lo: 141,
                        hi: 159,
                    },
                ),
                UseOfDynamicBool(
                    Span {
                        lo: 150,
                        hi: 156,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 150,
                        hi: 156,
                    },
                ),
                UseOfDynamicRange(
                    Span {
                        lo: 150,
                        hi: 156,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_closure_allowed() {
    check_profile(
        USE_CLOSURE_FUNCTION,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn use_of_static_int_return_from_entry_point_errors() {
    check_profile_for_exe(
        USE_ENTRY_POINT_STATIC_INT,
        &expect![[r#"
        [
            UseOfIntOutput(
                Span {
                    lo: 63,
                    hi: 66,
                },
            ),
        ]
    "#]],
    );
}

#[test]
fn use_of_static_double_return_from_entry_point_errors() {
    check_profile_for_exe(
        USE_ENTRY_POINT_STATIC_DOUBLE,
        &expect![[r#"
        [
            UseOfDoubleOutput(
                Span {
                    lo: 63,
                    hi: 66,
                },
            ),
        ]
    "#]],
    );
}

#[test]
fn use_of_static_string_return_from_entry_point_errors() {
    check_profile_for_exe(
        USE_ENTRY_POINT_STATIC_STRING,
        &expect![[r#"
        [
            UseOfAdvancedOutput(
                Span {
                    lo: 63,
                    hi: 66,
                },
            ),
        ]
    "#]],
    );
}

#[test]
fn use_of_static_bool_return_from_entry_point_errors() {
    check_profile_for_exe(
        USE_ENTRY_POINT_STATIC_BOOL,
        &expect![[r#"
        [
            UseOfBoolOutput(
                Span {
                    lo: 63,
                    hi: 66,
                },
            ),
        ]
    "#]],
    );
}

#[test]
fn use_of_static_big_int_return_from_entry_point_errors() {
    check_profile_for_exe(
        USE_ENTRY_POINT_STATIC_BIG_INT,
        &expect![[r#"
        [
            UseOfAdvancedOutput(
                Span {
                    lo: 63,
                    hi: 66,
                },
            ),
        ]
    "#]],
    );
}

#[test]
fn use_of_static_pauli_return_from_entry_point_errors() {
    check_profile_for_exe(
        USE_ENTRY_POINT_STATIC_PAULI,
        &expect![[r#"
        [
            UseOfAdvancedOutput(
                Span {
                    lo: 63,
                    hi: 66,
                },
            ),
        ]
    "#]],
    );
}

#[test]
fn use_of_static_range_return_from_entry_point_errors() {
    check_profile_for_exe(
        USE_ENTRY_POINT_STATIC_RANGE,
        &expect![[r#"
        [
            UseOfAdvancedOutput(
                Span {
                    lo: 63,
                    hi: 66,
                },
            ),
        ]
    "#]],
    );
}

#[test]
fn use_of_static_int_in_tuple_return_from_entry_point_errors() {
    check_profile_for_exe(
        USE_ENTRY_POINT_STATIC_INT_IN_TUPLE,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 63,
                        hi: 66,
                    },
                ),
                UseOfIntOutput(
                    Span {
                        lo: 63,
                        hi: 66,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_static_sized_array_in_tuple_error() {
    check_profile_for_exe(
        USE_ENTRY_POINT_INT_ARRAY_IN_TUPLE,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 63,
                        hi: 66,
                    },
                ),
                UseOfIntOutput(
                    Span {
                        lo: 63,
                        hi: 66,
                    },
                ),
            ]
        "#]],
    );
}
