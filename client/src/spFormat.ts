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
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as child from "child_process";

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

    let default_styles: string[] = Workspace.getConfiguration("sourcepawn").get(
      "formatterSettings"
    );

    let default_style: string = "{" + default_styles.join(", ") + "}";

    // Apply user settings
    default_style = default_style
      .replace(/\${TabSize}/, tabSize)
      .replace(/\${UseTab}/, UseTab);
    const start = new Position(0, 0);
    const end = new Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    const range = new Range(start, end);
		const tempFile = path.join(__dirname, "temp_format.sp");
		let file = fs.openSync(tempFile, "w", 0o765);
		fs.writeSync(file, document.getText());
		fs.closeSync(file);
    let text: string = this.clangFormat(tempFile, "utf-8", default_style);

    // If process failed,
    if (text === "") {
      window.showErrorMessage(
        "The formatter failed to run, check the console for more details."
      );
      return;
    }
    // clang-format gets confused with 'public' so we have to replace it manually.
    text = text.replace(/^ *public\s*\n/gm, "public ");
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
      let clangFormatProcess = child.execFileSync(nativeBinary, args);
      return clangFormatProcess.toString();
    } catch (e) {
      console.error("Error", e);
      return;
    }
  }

  getNativeBinary() {
    let nativeBinary;
    const platform = os.platform();
    const arch = os.arch();
    let myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
      .extensionPath;
    if (platform === "win32") {
      nativeBinary = path.join(myExtDir, "/bin/win32/clang-format.exe");
    } else {
      nativeBinary = path.join(
        myExtDir,
        `/bin/${platform}_${arch}/clang-format`
      );
    }

    if (fs.existsSync(nativeBinary)) {
      return nativeBinary;
    }

    // Let arm64 macOS fall back to x64
    if (platform === "darwin" && arch === "arm64") {
      nativeBinary = path.join(myExtDir, `/bin/darwin_x64/clang-format`);
      if (fs.existsSync(nativeBinary)) {
        return nativeBinary;
      }
    }
    const message =
      "This module doesn't bundle the clang-format executable for your platform. " +
      `(${platform}_${arch})\n` +
      "Please let the author know on GitHub.\n";
    throw new Error(message);
  }
}
