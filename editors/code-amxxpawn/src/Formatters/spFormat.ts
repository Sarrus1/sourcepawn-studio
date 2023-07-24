import {
  DocumentFormattingEditProvider,
  TextDocument,
  FormattingOptions,
  CancellationToken,
  ProviderResult,
  TextEdit,
  workspace as Workspace,
  Position,
  Range,
  extensions,
  window,
} from "vscode";
import { openSync, writeSync, closeSync, existsSync } from "fs";
import { platform, arch } from "os";
import { join } from "path";
import { execFileSync } from "child_process";

export class SMDocumentFormattingEditProvider
  implements DocumentFormattingEditProvider
{
  public provideDocumentFormattingEdits(
    document: TextDocument,
    options: FormattingOptions,
    token: CancellationToken
  ): ProviderResult<TextEdit[]> {
    // Get the user's settings.
    const insertSpaces: boolean =
      Workspace.getConfiguration("editor").get("insertSpaces") || false;
    const UseTab: string = insertSpaces ? "Never" : "Always";
    const tabSize: number =
      Workspace.getConfiguration("editor").get("tabSize") || 2;

    const workspaceFolder = Workspace.getWorkspaceFolder(document.uri);
    const defaultStyles: string[] =
      Workspace.getConfiguration("amxxpawn", workspaceFolder).get(
        "formatterSettings"
      ) || [];

    let default_style: string = "{" + defaultStyles.join(", ") + "}";

    // Apply user settings
    default_style = default_style
      .replace(/\${TabSize}/g, tabSize.toString())
      .replace(/\${UseTab}/g, UseTab);
    const start = new Position(0, 0);
    const end = new Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    const range = new Range(start, end);
    const tempFile = join(__dirname, "temp_format.sma");
    const file = openSync(tempFile, "w", 0o765);
    writeSync(file, document.getText());
    closeSync(file);
    let text = this.clangFormat(tempFile, "utf-8", default_style);

    // If process failed,
    if (text === undefined) {
      window.showErrorMessage(
        "The formatter failed to run, check the console for more details."
      );
      return undefined;
    }
    text = fixFormatting(text);
    return [new TextEdit(range, text)];
  }

  Callback(e) {
    console.error(e);
  }

  clangFormat(path: string, enc: string, style): string | undefined {
    const args = [`-style=${style}`, path];
    const result = this.spawnClangFormat(args, [
      "ignore",
      "pipe",
      process.stderr,
    ]);
    if (result) {
      return result;
    } else {
      console.error("Formatting failed.");
      return undefined;
    }
  }

  spawnClangFormat(args, stdio) {
    let nativeBinary;

    try {
      nativeBinary = this.getNativeBinary();
    } catch (e) {
      return undefined;
    }
    try {
      const clangFormatProcess = execFileSync(nativeBinary, args);
      return clangFormatProcess.toString();
    } catch (e) {
      console.error("Error", e);
      return undefined;
    }
  }

  getNativeBinary() {
    let nativeBinary;
    const sysPlatform = platform();
    const sysArch = arch();
    const ext = extensions.getExtension("Sarrus.amxxpawn-vscode");
    if (ext === undefined) {
      throw Error("Extension not found.");
    }
    const myExtDir = ext.extensionPath;
    if (sysPlatform === "win32") {
      nativeBinary = join(myExtDir, "/bin/win32/clang-format.exe");
    } else {
      nativeBinary = join(
        myExtDir,
        `/bin/${sysPlatform}_${sysArch}/clang-format`
      );
    }

    if (existsSync(nativeBinary)) {
      return nativeBinary;
    }

    // Let arm64 macOS fall back to x64
    if (sysPlatform === "darwin" && sysArch === "arm64") {
      nativeBinary = join(myExtDir, `/bin/darwin_x64/clang-format`);
      if (existsSync(nativeBinary)) {
        return nativeBinary;
      }
    }
    const message =
      "This module doesn't bundle the clang-format executable for your platform. " +
      `(${sysPlatform}_${sysArch})\n` +
      "Please let the author know on GitHub.\n";
    throw new Error(message);
  }
}

function fixFormatting(text: string): string {
  // clang-format gets confused with 'public' so we have to replace it manually.
  text = text.replace(/(?:(\*\/|\/\/.*)\r?\n)\s*public\r?\n/gm, "$1\npublic ");
  text = text.replace(/(?!(\*\/|\/\/.*)\r?\n)\s*public\r?\n/gm, "\n\npublic ");

  // clang-format also messes up the myinfo array.
  text = text.replace(
    /(public\s+Plugin\s+myinfo\s*=)\s*(\{[^}{]+)(\})/m,
    "$1\n$2\n$3"
  );

  // clang-format messes up the trailing } of the myinfo array.
  text = text.replace(/\n{2,}\};/, "\n};");

  return text;
}
