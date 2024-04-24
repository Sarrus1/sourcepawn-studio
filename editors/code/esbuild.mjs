import * as fs from "fs";
import * as path from "path";
import * as esbuild from "esbuild";
import { createRequire } from "module";
const require = createRequire(path.resolve("./"));

const outDir = "./dist";

const wasmPlugin = {
  name: "wasmPlugin",
  setup(_) {
    const wasmPaths = ["./node_modules/valve_kv_tools/valve_kv_tools_bg.wasm"];
    wasmPaths.forEach((wasmPath) => {
      fs.copyFileSync(wasmPath, path.join(outDir, path.basename(wasmPath)));
    });
  },
};

if (!fs.existsSync(outDir)) {
  fs.mkdirSync(outDir);
}

const watch = process.argv[2] === "watch";

let ctx = await esbuild.build({
  entryPoints: ["./src/spIndex.ts"],
  bundle: true,
  sourcemap: true,
  minify: !watch,
  outfile: `${outDir}/spIndex.js`,
  logLevel: "info",
  external: ["vscode"],
  format: "cjs",
  platform: "node",
  loader: { ".node": "file" },
  plugins: [wasmPlugin],
});

if (watch) {
  await ctx.watch();
  console.log("watching...");
}
