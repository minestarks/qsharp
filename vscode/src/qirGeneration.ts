// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import * as vscode from "vscode";
import { getCompilerWorker, log, ProgramConfig } from "qsharp-lang";
import { EventType, sendTelemetryEvent } from "./telemetry";
import { getRandomGuid } from "./utils";
import { getTarget, getEnablePreviewQirGen, setTarget } from "./config";
import { getActiveProgram } from "./programConfig";

const generateQirTimeoutMs = 30000;

let compilerWorkerScriptPath: string;

export class QirGenerationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "QirGenerationError";
  }
}

export async function getQirForActiveWindow(): Promise<string> {
  let result = "";
  const program = await getActiveProgram();
  if (!program.success) {
    throw new QirGenerationError(program.errorMsg);
  }

  const { languageFeatures, sources, profile } = program.programConfig;

  // Check that the current target is base profile, and current doc has no errors.
  const enablePreviewQirGen = getEnablePreviewQirGen();
  if (profile !== "base") {
    const allowed = profile === "adaptive_ri" && enablePreviewQirGen;
    if (!allowed) {
      const result = await vscode.window.showWarningMessage(
        "Submitting to Azure is only supported when targeting the QIR base profile.",
        { modal: true },
        { title: "Change the QIR target profile and continue", action: "set" },
        { title: "Cancel", action: "cancel", isCloseAffordance: true },
      );
      if (result?.action !== "set") {
        throw new QirGenerationError(
          "Submitting to Azure is only supported when targeting the QIR base profile. " +
            "Please update the QIR target via the status bar selector or extension settings.",
        );
      } else {
        setTarget("base");
      }
    }
  }

  for (const source of sources) {
    const diagnostics = await vscode.languages.getDiagnostics(
      vscode.Uri.parse(source[0]),
    );
    if (diagnostics?.length > 0) {
      throw new QirGenerationError(
        "The current program contains errors that must be fixed before submitting to Azure",
      );
    }
  }
  // Create a temporary worker just to get the QIR, as it may loop/panic during codegen.
  // Let it run for max 10 seconds, then terminate it if not complete.
  const worker = getCompilerWorker(compilerWorkerScriptPath);
  const compilerTimeout = setTimeout(() => {
    worker.terminate();
  }, generateQirTimeoutMs);
  try {
    const associationId = getRandomGuid();
    const start = performance.now();
    sendTelemetryEvent(EventType.GenerateQirStart, { associationId }, {});
    if (enablePreviewQirGen) {
      languageFeatures.push("preview-qir-gen");
    }
    const config = {
      sources,
      languageFeatures,
      profile: getTarget(),
    } as ProgramConfig;
    result = await worker.getQir(config);

    sendTelemetryEvent(
      EventType.GenerateQirEnd,
      { associationId },
      { qirLength: result.length, timeToCompleteMs: performance.now() - start },
    );
    clearTimeout(compilerTimeout);
  } catch (e: any) {
    log.error("Codegen error. ", e.toString());
    throw new QirGenerationError(
      `Code generation failed due to error: "${e.toString()}". Please ensure the code is compatible with the QIR base profile ` +
        "by setting the target QIR profile to 'base' and fixing any errors.",
    );
  } finally {
    worker.terminate();
  }

  return result;
}

export function initCodegen(context: vscode.ExtensionContext) {
  compilerWorkerScriptPath = vscode.Uri.joinPath(
    context.extensionUri,
    "./out/compilerWorker.js",
  ).toString();

  context.subscriptions.push(
    vscode.commands.registerCommand("qsharp-vscode.getQir", async () => {
      try {
        const qir = await getQirForActiveWindow();
        const qirDoc = await vscode.workspace.openTextDocument({
          language: "llvm",
          content: qir,
        });
        await vscode.window.showTextDocument(qirDoc);
      } catch (e: any) {
        log.error("QIR generation failed. ", e);
        if (e.name === "QirGenerationError") {
          vscode.window.showErrorMessage(e.message);
        }
      }
    }),
  );
}
