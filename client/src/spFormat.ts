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
  implements DocumentFormattingEditProvider {
  public provideDocumentFormattingEdits(
    document: TextDocument,
    options: FormattingOptions,
    token: CancellationToken
  ): ProviderResult<TextEdit[]> {
    const result = [];
    // Get the user's settings.
    let insert_spaces: boolean = Workspace.getConfiguration("editor").get(
      "insertSpaces"
    );
    let UseTab: string = insert_spaces ? "Never" : "Always";
    let tabSize: string = Workspace.getConfiguration("editor").get("tabSize");

    let workspaceFolder = Workspace.getWorkspaceFolder(document.uri);
    let default_styles: string[] = Workspace.getConfiguration(
      "sourcepawn",
      workspaceFolder
    ).get("formatterSettings");

    let default_style: string = "{" + default_styles.join(", ") + "}";

    // Apply user settings
    default_style = default_style
      .replace(/\${TabSize}/g, tabSize)
      .replace(/\${UseTab}/g, UseTab);
    const start = new Position(0, 0);
    const end = new Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    const range = new Range(start, end);
    const tempFile = join(__dirname, "temp_format.sp");
    let file = openSync(tempFile, "w", 0o765);
    writeSync(file, document.getText());
    closeSync(file);
    let text: string = this.clangFormat(tempFile, "utf-8", default_style);

    // If process failed,
    if (text === "") {
      window.showErrorMessage(
        "The formatter failed to run, check the console for more details."
      );
      return;
    }
    text = fixFormatting(text);
    result.push(new TextEdit(range, text));
    return result;
  }

  Callback(e) {
    console.error(e);
  }

  clangFormat(path: string, enc: string, style) {
    let args = [`-style=${style}`, path];
    let result = this.spawnClangFormat(args, [
      "ignore",
      "pipe",
      process.stderr,
    ]);
    if (result) {
      return result;
    } else {
      console.error("Formatting failed.");
    }
  }

  spawnClangFormat(args, stdio) {
    let nativeBinary;

    try {
      nativeBinary = this.getNativeBinary();
    } catch (e) {
      return;
    }
    try {
      let clangFormatProcess = execFileSync(nativeBinary, args);
      return clangFormatProcess.toString();
    } catch (e) {
      console.error("Error", e);
      return;
    }
  }

  getNativeBinary() {
    let nativeBinary;
    const sysPlatform = platform();
    const sysArch = arch();
    let myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
      .extensionPath;
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
  text = text.replace(/^\s*public\n/gm, "\npublic ");

  // clang-format also messes up the myinfo array.
  text = text.replace(
    /(public\s+Plugin\s+myinfo\s*=)\s*(\{[^}{]+)(\})/m,
    "$1\n$2\n$3"
  );

  // clang-format messes up the trailing } of the myinfo array.
  text = text.replace(/\n{2,}\};/, "\n};");

  return text;
}
