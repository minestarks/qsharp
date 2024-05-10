// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

import { Qubit } from '../circuit';
import { RegisterType, RegisterMap, RegisterMetadata } from '../register';
import { leftPadding, startY, registerHeight, classicalRegHeight, annotationBoxHeight } from '../constants';
import { group, text } from './formatUtils';

/**
 * `formatInputs` takes in an array of Qubits and outputs the SVG string of formatted
 * qubit wires and a mapping from register IDs to register metadata (for rendering).
 *
 * @param qubits List of declared qubits.
 *
 * @returns returns the SVG string of formatted qubit wires, a mapping from registers
 *          to y coord and total SVG height.
 */
const formatInputs = (
    qubits: Qubit[],
    annotationRow: boolean,
): { qubitWires: SVGElement; registers: RegisterMap; svgHeight: number; annotationY?: number } => {
    const qubitWires: SVGElement[] = [];
    const registers: RegisterMap = {};

    let currY: number = startY;
    qubits.forEach(({ id, numChildren }) => {
        // Add qubit wire to list of qubit wires
        qubitWires.push(_qubitInput(currY));

        // Create qubit register
        registers[id] = { type: RegisterType.Qubit, y: currY };

        // If there are no attached classical registers, increment y by fixed register height
        if (numChildren == null || numChildren === 0) {
            currY += registerHeight;
            return;
        }

        // Increment current height by classical register height for attached classical registers
        currY += classicalRegHeight;

        // Add classical wires
        registers[id].children = Array.from(Array(numChildren), () => {
            const clsReg: RegisterMetadata = {
                type: RegisterType.Classical,
                y: currY,
            };
            currY += classicalRegHeight;
            return clsReg;
        });
    });

    const info: any = {
        qubitWires: group(qubitWires),
        registers,
        svgHeight: currY,
    };

    if (annotationRow) {
        info.annotationY = info.svgHeight;
        info.svgHeight += annotationBoxHeight;
    }

    return info;
};

/**
 * Generate the SVG text component for the input qubit register.
 *
 * @param y y coord of input wire to render in SVG.
 *
 * @returns SVG text component for the input register.
 */
const _qubitInput = (y: number): SVGElement => {
    const el: SVGElement = text('|0⟩', leftPadding, y, 16);
    el.setAttribute('text-anchor', 'start');
    el.setAttribute('dominant-baseline', 'middle');
    return el;
};

export { formatInputs, _qubitInput };
