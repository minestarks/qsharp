**Inputs:** $3$ qubits in an arbitrary superposition state $\ket{\psi}$, stored in an array of length $3$.

**Goal:** Apply the following matrix to the system. This matrix can be represented as applying $3$ single-qubit gates.

$$
Q =
\begin{bmatrix}
0 & -i & 0 & 0 & 0 & 0 & 0 & 0 \\ 
i & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\ 
0 & 0 & 0 & -i & 0 & 0 & 0 & 0 \\ 
0 & 0 & i & 0 & 0 & 0 & 0 & 0 \\ 
0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\ 
0 & 0 & 0 & 0 & -1 & 0 & 0 & 0 \\ 
0 & 0 & 0 & 0 & 0 & 0 & 0 & 1 \\ 
0 & 0 & 0 & 0 & 0 & 0 & -1 & 0
\end{bmatrix}
$$

> We recommend to keep a list of common quantum gates on hand.

<details>
    <summary><b>Need a hint?</b></summary>
    <p>Start by noticing that the top right and bottom left quadrants of the matrix are filled with $0$s, and the bottom right quadrant equals to the top left one, multiplied by $i$. Does this look like a tensor product of a 1-qubit and 2-qubit matrices? Which ones?</p>
</details>
