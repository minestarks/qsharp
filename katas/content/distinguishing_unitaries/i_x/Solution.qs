namespace Kata {
    operation DistinguishIX (unitary : (Qubit => Unit is Adj + Ctl)) : Int {
        use q = Qubit();
        unitary(q);
        return MResetZ(q) == Zero ? 0 | 1;
    }
}
