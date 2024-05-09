namespace Kata.Verification {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    operation MultiControls(controls : Qubit[], target : Qubit, controlBits : Bool[]) : Unit is Adj + Ctl {
        within {
            for index in 0 .. Length(controls) - 1 {
                if controlBits[index] == false {
                    X(controls[index]);
                }
            }
        } apply {
            Controlled X(controls,target);
        }
    }

    operation CheckSolution() : Bool {
        for i in 0 .. (2 ^ 4) - 1 {
            let bits = IntAsBoolArray(i, 4);
            let solution = register => Kata.MultiControls(Most(register), Tail(register), bits);
            let reference = register => MultiControls(Most(register), Tail(register), bits);
            if not CheckOperationsAreEqual(5, solution, reference) {
                Message("Incorrect.");
                Message($"The test case for controlBits = {bits} did not pass.");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}