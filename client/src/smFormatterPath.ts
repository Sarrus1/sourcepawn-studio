const fs = require('fs');
const os = require('os');
const path = require('path');
import * as vscode from "vscode";
import * as child from 'child_process';

/**
 * Starts a child process running the native clang-format binary.
 *
 * @param file a Vinyl virtual file reference
 * @param enc the encoding to use for reading stdout
 * @param style valid argument to clang-format's '-style' flag
 * @param done callback invoked when the child process terminates
 * @returns {stream.Readable} the formatted code as a Readable stream
 */
export function clangFormat(file : vscode.TextDocument, enc : string, style, done) {
	let args = [`-style=${style}`, file.uri.fsPath];
  let result = spawnClangFormat(args, done, ['ignore', 'pipe', process.stderr]);
  if (result) {
    return result;
  } else {
		console.error("Formatting failed.");
  }
}

/**
 * Spawn the clang-format binary with given arguments.
 */
function spawnClangFormat(args, done, stdio) {
  // WARNING: This function's interface should stay stable across versions for the cross-version
  // loading below to work.
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


/**
 * @returns the native `clang-format` binary for the current platform
 * @throws when the `clang-format` executable can not be found
 */
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
      'Consider installing it with your native package manager instead.\n';
  throw new Error(message);
}