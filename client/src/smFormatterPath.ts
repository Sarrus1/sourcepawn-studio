import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";
import * as child from 'child_process';


export function clangFormat(file : vscode.TextDocument, enc : string, style) {
	let args = [`-style=${style}` ,file.uri.fsPath];
  let result = spawnClangFormat(args, ['ignore', 'pipe', process.stderr]);
  if (result) {
    return result;
  } else {
		console.error("Formatting failed.");
  }
}

/**
 * Spawn the clang-format binary with given arguments.
 */
function spawnClangFormat(args, stdio) {
  let nativeBinary;

  try {
    nativeBinary = getNativeBinary();
  } catch (e) {
    return;
  }
	try {
    let clangFormatProcess = child.execFileSync(nativeBinary, args);
		return clangFormatProcess.toString();
  } catch (e) {
    console.error("Error", e);
    return;
  }    
}

function getNativeBinary() {
  let nativeBinary;
  const platform = os.platform();
  const arch = os.arch();
	let myExtDir : string = vscode.extensions.getExtension ("Sarrus.sourcepawn-vscode").extensionPath;
  if (platform === 'win32') {
    nativeBinary = path.join(myExtDir, "/bin/win32/clang-format.exe");
  } else {
		nativeBinary = path.join(myExtDir, `/bin/${platform}_${arch}/clang-format`);
  }

  if (fs.existsSync(nativeBinary)) {
    return nativeBinary;
  }

  // Let arm64 macOS fall back to x64
  if (platform === 'darwin' && arch === 'arm64') {
    nativeBinary = path.join(myExtDir,`/bin/darwin_x64/clang-format`);
    if (fs.existsSync(nativeBinary)) {
      return nativeBinary;
    }
  }
  const message = 'This module doesn\'t bundle the clang-format executable for your platform. ' +
      `(${platform}_${arch})\n` +
      'Please let the author know on GitHub.\n';
  throw new Error(message);
}