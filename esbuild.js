const fs = require("fs");
const path = require("path");

const outDir = "./dist";

const treeSitterWasmPlugin = {
  name: "treeSitterWasm",
  setup(build) {
    const wasmPaths = [
      "./bin/tree-sitter.wasm",
      "./bin/tree-sitter-sourcepawn.wasm",
    ];
    // build.onEnd(result => {
    wasmPaths.forEach((wasmPath) => {
      fs.copyFileSync(wasmPath, path.join(outDir, path.basename(wasmPath)));
    });
    // })
  },
};

if (!fs.existsSync(outDir)) {
  fs.mkdir(outDir);
}

require("esbuild")
  .build({
    entryPoints: ["./src/spIndex.ts"],
    bundle: true,
    sourcemap: true,
    minify: true,
    outfile: `${outDir}/spIndex.js`,
    logLevel: "info",
    external: ["vscode"],
    format: "cjs",
    platform: "node",
    plugins: [treeSitterWasmPlugin],
    watch: !!process.env.ESBUILD_WATCH,
  })
  .catch(() => process.exit(1));
