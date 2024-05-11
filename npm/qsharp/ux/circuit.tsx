// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as qviz from "@microsoft/quantum-viz.js/lib";
import { useEffect, useRef, useState } from "preact/hooks";
import { CircuitProps } from "./data.js";
import { Spinner } from "./spinner.js";

// For perf reasons we set a limit on how many gates/qubits
// we attempt to render. This is still a lot higher than a human would
// reasonably want to look at, but it takes about a second to
// render a circuit this big on a mid-grade laptop so we allow it.
const MAX_OPERATIONS = 10000;
const MAX_QUBITS = 1000;

// This component is shared by the Python widget and the VS Code panel
export function Circuit(props: {
  circuit: qviz.Circuit;
  mdRender: (input: string) => string;
}) {
  const circuit = props.circuit;
  const unrenderable =
    circuit.qubits.length === 0 ||
    circuit.operations.length > MAX_OPERATIONS ||
    circuit.qubits.length > MAX_QUBITS;

  return (
    <div>
      {unrenderable ? (
        <Unrenderable
          qubits={props.circuit.qubits.length}
          operations={props.circuit.operations.length}
        />
      ) : (
        <ZoomableCircuit {...props} />
      )}
    </div>
  );
}

function ZoomableCircuit(props: {
  circuit: qviz.Circuit;
  mdRender: (input: string) => string;
}) {
  const circuitDiv = useRef<HTMLDivElement>(null);
  const [zoomLevel, setZoomLevel] = useState(100);
  const [rendering, setRendering] = useState(true);

  useEffect(() => {
    // Enable "rendering" text while the circuit is being drawn
    setRendering(true);
    const container = circuitDiv.current!;
    container.innerHTML = "";
  }, [props.circuit]);

  useEffect(() => {
    if (rendering) {
      const container = circuitDiv.current!;
      // Draw the circuit - may take a while for large circuits
      const svg = renderCircuit(props.circuit, props.mdRender, container);
      // Calculate the initial zoom level based on the container width
      const initialZoom = calculateZoomToFit(container, svg as SVGElement);
      // Set the initial zoom level
      setZoomLevel(initialZoom);
      // Resize the SVG to fit
      updateWidth();
      // Disable "rendering" text
      setRendering(false);
    }
  }, [rendering]);

  useEffect(() => {
    updateWidth();
  }, [zoomLevel]);

  return (
    <div>
      <div>
        {rendering ? null : (
          <ZoomControl zoom={zoomLevel} onChange={setZoomLevel} />
        )}
      </div>
      <div>
        {rendering
          ? `Rendering diagram with ${props.circuit.operations.length} gates...`
          : ""}
      </div>
      <div class="qs-circuit" ref={circuitDiv}></div>
    </div>
  );

  function updateWidth() {
    const svg = circuitDiv.current?.querySelector(".qviz");
    if (svg) {
      // The width attribute contains the true width, generated by qviz.
      // We'll leave this attribute untouched, so we can use it again if the
      // zoom level is ever updated.
      const width = svg.getAttribute("width")!;

      // We'll set the width in the style attribute to (true width * zoom level).
      // This value takes precedence over the true width in the width attribute.
      svg.setAttribute(
        "style",
        `max-width: ${width}; width: ${(parseInt(width) * (zoomLevel || 100)) / 100}; height: auto`,
      );
    }
  }

  function renderCircuit(
    circuit: qviz.Circuit,
    mdRender: (input: string) => string,
    container: HTMLDivElement,
  ) {
    qviz.drawWithMd(circuit, container, mdRender);

    // quantum-viz hardcodes the styles in the SVG.
    // Remove the style elements -- we'll define the styles in our own CSS.
    const styleElements = container.querySelectorAll("style");
    styleElements?.forEach((tag) => tag.remove());

    // Render the markdown in the annotations
    const texts = container.querySelectorAll(".annotation-text");
    texts.forEach((text) => {
      if (text.innerHTML === "annotation") {
        text.innerHTML = "";
        return;
      }
      console.log(text.innerHTML);
      const rendered = mdRender(text.innerHTML);

      const foreignObject =
        text.parentElement!.querySelectorAll(".annotation-box")[0];

      if (foreignObject) {
        text.innerHTML = "";
        foreignObject.innerHTML += `
        <div xmlns="http://www.w3.org/1999/xhtml" style="text-align: center">
          ${rendered}
        </div>
      `;
      }

      console.log(rendered);
    });

    return container.getElementsByClassName("qviz")[0]!;
  }

  function calculateZoomToFit(container: HTMLDivElement, svg: SVGElement) {
    const containerWidth = container.clientWidth;
    // width and height are the true dimensions generated by qviz
    const width = parseInt(svg.getAttribute("width")!);
    const height = svg.getAttribute("height")!;

    svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
    const zoom = Math.min(Math.ceil((containerWidth / width) * 100), 100);
    return zoom;
  }
}

function Unrenderable(props: { qubits: number; operations: number }) {
  const errorDiv =
    props.qubits === 0 ? (
      <div>
        <p>No circuit to display. No qubits have been allocated.</p>
      </div>
    ) : props.operations > MAX_OPERATIONS ? (
      <div>
        <p>
          This circuit has too many gates to display. It has {props.operations}{" "}
          gates, but the maximum supported is {MAX_OPERATIONS}.
        </p>
      </div>
    ) : props.qubits > MAX_QUBITS ? (
      <div>
        <p>
          This circuit has too many qubits to display. It has {props.qubits}{" "}
          qubits, but the maximum supported is {MAX_QUBITS}.
        </p>
      </div>
    ) : undefined;

  return <div class="qs-circuit-error">{errorDiv}</div>;
}

function ZoomControl(props: {
  zoom: number;
  onChange: (zoom: number) => void;
}) {
  return (
    <p>
      <label htmlFor="qs-circuit-zoom">Zoom </label>
      <input
        id="qs-circuit-zoom"
        type="number"
        min="10"
        max="100"
        step="10"
        value={props.zoom}
        onInput={(e) =>
          props.onChange(parseInt((e.target as HTMLInputElement).value) || 0)
        }
      />
      %
    </p>
  );
}

// This component is exclusive to the VS Code panel
export function CircuitPanel(props: CircuitProps) {
  const error = props.errorHtml ? (
    <div>
      <p>
        {props.circuit
          ? "The program encountered a failure. See the error(s) below."
          : "A circuit could not be generated for this program. See the error(s) below."}
        <br />
      </p>
      <div dangerouslySetInnerHTML={{ __html: props.errorHtml }}></div>
    </div>
  ) : null;

  return (
    <div class="qs-circuit-panel">
      <div>
        <h1>
          {props.title} {props.simulated ? "(Trace)" : ""}
        </h1>
      </div>
      <div class="qs-circuit-error">{error}</div>
      <p>{props.targetProfile}</p>
      <p>
        {
          props.simulated
            ? "WARNING: This diagram shows the result of tracing a dynamic circuit, and may change from run to run."
            : "\xa0" // nbsp to keep line height consistent
        }
      </p>
      <p>
        Learn more at{" "}
        <a href="https://aka.ms/qdk.circuits">https://aka.ms/qdk.circuits</a>
      </p>
      {props.calculating ? (
        <div>
          <Spinner />
        </div>
      ) : null}
      {props.circuit ? (
        <Circuit circuit={props.circuit} mdRender={props.mdRender}></Circuit>
      ) : null}
    </div>
  );
}
