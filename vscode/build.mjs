// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//@ts-check

import { copyFileSync, mkdirSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { build, context } from "esbuild";

const thisDir = dirname(fileURLToPath(import.meta.url));
const libsDir = join(thisDir, "..", "node_modules");

/** @type {import("esbuild").BuildOptions} */
const buildOptions = {
  entryPoints: [
    join(thisDir, "src", "extension.ts"),
    join(thisDir, "src", "compilerWorker.ts"),
    join(thisDir, "src", "debugger/debug-service-worker.ts"),
    join(thisDir, "src", "webview/webview.tsx"),
  ],
  outdir: join(thisDir, "out"),
  bundle: true,
  // minify: true,
  mainFields: ["browser", "module", "main"],
  external: ["vscode"],
  format: "cjs",
  platform: "browser",
  target: ["es2020"],
  sourcemap: "linked",
  //logLevel: "debug",
  define: { "import.meta.url": "undefined" },
};

function getTimeStr() {
  const now = new Date();

  const hh = now.getHours().toString().padStart(2, "0");
  const mm = now.getMinutes().toString().padStart(2, "0");
  const ss = now.getSeconds().toString().padStart(2, "0");
  const mil = now.getMilliseconds().toString().padStart(3, "0");

  return `${hh}:${mm}:${ss}.${mil}`;
}

export function copyWasmToVsCode() {
  // Copy the wasm module into the extension directory
  let qsharpWasm = join(
    thisDir,
    "..",
    "npm",
    "qsharp",
    "lib",
    "web",
    "qsc_wasm_bg.wasm",
  );
  let qsharpDest = join(thisDir, `wasm`);

  console.log("Copying the wasm file to VS Code from: " + qsharpWasm);
  mkdirSync(qsharpDest, { recursive: true });
  copyFileSync(qsharpWasm, join(qsharpDest, "qsc_wasm_bg.wasm"));
}

/**
 *
 * @param {string} [destDir]
 */
export function copyKatex(destDir) {
  let katexBase = join(libsDir, `katex/dist`);
  let katexDest = destDir ?? join(thisDir, `out/katex`);

  console.log("Copying the Katex files over from: " + katexBase);
  mkdirSync(katexDest, { recursive: true });
  copyFileSync(
    join(katexBase, "katex.min.css"),
    join(katexDest, "katex.min.css"),
  );

  // Also copy the GitHub markdown CSS
  copyFileSync(
    join(libsDir, `github-markdown-css/github-markdown-light.css`),
    join(katexDest, "github-markdown-light.css"),
  );
  copyFileSync(
    join(libsDir, `github-markdown-css/github-markdown-dark.css`),
    join(katexDest, "github-markdown-dark.css"),
  );

  const fontsDir = join(katexBase, "fonts");
  const fontsOutDir = join(katexDest, "fonts");

  mkdirSync(fontsOutDir, { recursive: true });

  for (const file of readdirSync(fontsDir)) {
    if (file.endsWith(".woff2")) {
      copyFileSync(join(fontsDir, file), join(fontsOutDir, file));
    }
  }
}

function buildBundle() {
  console.log("Running esbuild");

  build(buildOptions).then(() =>
    console.log(`Built bundle to ${join(thisDir, "out")}`),
  );
}

export async function watchVsCode() {
  console.log("Building vscode extension in watch mode");

  // Plugin to log start/end of build events (mostly to help VS Code problem matcher)
  /** @type {import("esbuild").Plugin} */
  const buildPlugin = {
    name: "Build Events",
    setup(build) {
      build.onStart(() =>
        console.log("VS Code build started @ " + getTimeStr()),
      );
      build.onEnd(() =>
        console.log("VS Code build complete @ " + getTimeStr()),
      );
    },
  };
  let ctx = await context({
    ...buildOptions,
    plugins: [buildPlugin],
    color: false,
  });

  ctx.watch();
}

const thisFilePath = resolve(fileURLToPath(import.meta.url));
if (thisFilePath === resolve(process.argv[1])) {
  // This script being run directly (not imported)
  const isWatch = process.argv.includes("--watch");
  if (isWatch) {
    watchVsCode();
  } else {
    copyWasmToVsCode();
    copyKatex();
    buildBundle();
  }
}
