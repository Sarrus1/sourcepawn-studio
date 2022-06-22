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
    wasmPaths.forEach((wasmPath) => {
      fs.copyFileSync(wasmPath, path.join(outDir, path.basename(wasmPath)));
    });
  },
};

if (!fs.existsSync(outDir)) {
  fs.mkdirSync(outDir);
}
const watch = process.argv[2] === "watch";

require("esbuild")
  .build({
    entryPoints: ["./src/spIndex.ts"],
    bundle: true,
    sourcemap: true,
    minify: !watch,
    outfile: `${outDir}/spIndex.js`,
    logLevel: "info",
    external: ["vscode"],
    format: "cjs",
    platform: "node",
    plugins: [treeSitterWasmPlugin],
    watch: watch,
  })
  .catch(() => process.exit(1));
