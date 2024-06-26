import * as vscode from "vscode";
import * as fs from "fs";
import { execFile } from "child_process";
import { getConfig, Section } from "../configUtils";

export async function run(args: any): Promise<void> {
  const panel = vscode.window.createWebviewPanel(
    "sourcepawnDoctor",
    "SourcePawn Doctor",
    vscode.ViewColumn.One,
    {}
  );

  const doctor = new Doctor();
  doctor.runDiagnostics();

  const updateWebview = () => {
    panel.webview.html = doctor.toWebview();
  };

  // Set initial content
  updateWebview();

  // And schedule updates to the content every second
  const interval = setInterval(updateWebview, 100);

  panel.onDidDispose(
    () => {
      // When the panel is closed, cancel any future updates to the webview content
      clearInterval(interval);
    },
    null,
    null
  );
}

enum DiagnosticState {
  OK,
  Warning,
  Error,
  None,
}

class Doctor {
  // Settings
  compilerPath: string | undefined = undefined;
  isSPCompSet = DiagnosticState.None;
  isSPCompInstalled = DiagnosticState.None;
  isSPCompRunnable = DiagnosticState.None;
  SPCompVersion: String = undefined;

  isSMInstalled = DiagnosticState.None;

  constructor() {}

  toWebview(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Cat Coding</title>
</head>
<body>
    <h1>SourcePawn Doctor</h1>
    <h2>Compiler (spcomp)</h2>
    <ul>
      ${this.spCompToWebView()}
    </ul>
    <h2>Includes</h2>
    <ul>
      ${this.includeDirectoriesToWebView()}
    </ul>

    <h2>Additional help</h2>
    <p>If all the above are green and the extension is still not behaving as expected, try the following:</p>
    <ul>
      <li>Restart the SourcePawn Language Server (Hover your mouse on the "sourcepawn-studio" logo on the bottom left of the screen and click on restart).</li>
      <li>Reload VSCode (CTRL+Shift+P and type "Reload Window").</li>
      <li>Look in the logs for errors (Hover your mouse on the "sourcepawn-studio" logo on the bottom left of the screen and click on Open Logs). You can set the verbosity of the server to "trace" in the "sourcepawn.trace.server" setting.</li>
      <li>Try to reproduce the issue in a new project.</li>
      <li>If the extension is still not working properly, try contacting Sarrus on Discord (sarrus_).</li>
      </ul>
</body>
</html>`;
  }

  async runDiagnostics() {
    this.checkSettings();
    this.checkIncludeDirectories();
    this.checkSpComp();
  }

  spCompToWebView(): string {
    const diagnostics = [];
    switch (this.isSPCompSet) {
      case DiagnosticState.OK:
        diagnostics.push(
          `✅ "SourcePawnLanguageServer.compiler.path" is set (value: ${this.compilerPath}).`
        );
        break;
      case DiagnosticState.Error:
        diagnostics.push(
          '❌ "SourcePawnLanguageServer.compiler.path" is empty. You should set it to the path of the "spcomp" executable.'
        );
        break;
      case DiagnosticState.None:
        diagnostics.push(
          '🩺 Checking if "SourcePawnLanguageServer.compiler.path" is set.'
        );
        break;
    }

    switch (this.isSPCompInstalled) {
      case DiagnosticState.OK:
        diagnostics.push(
          `✅ "SourcePawnLanguageServer.compiler.path" points to a file.`
        );
        break;
      case DiagnosticState.Error:
        diagnostics.push(
          `❌ "SourcePawnLanguageServer.compiler.path" does not point to a file.`
        );
        break;
      case DiagnosticState.None:
        diagnostics.push(
          '🩺 Checking if "SourcePawnLanguageServer.compiler.path" points to a file.'
        );
        break;
    }

    switch (this.isSPCompRunnable) {
      case DiagnosticState.OK:
        diagnostics.push(
          `✅ "SourcePawnLanguageServer.compiler.path" is executable v${this.SPCompVersion}.`
        );
        break;
      case DiagnosticState.Error:
        diagnostics.push(
          `❌ "SourcePawnLanguageServer.compiler.path" is not executable.`
        );
        break;
      case DiagnosticState.None:
        diagnostics.push(
          '🩺 Checking if "SourcePawnLanguageServer.compiler.path" is executable.'
        );
        break;
    }

    return diagnostics.map((d) => `<li>${d}</li>`).join("\n");
  }

  async checkSpComp() {
    this.compilerPath = getConfig(Section.LSP, "compiler.path");
    if (!this.compilerPath) {
      this.isSPCompSet = DiagnosticState.Error;
      this.isSPCompInstalled = DiagnosticState.Error;
      this.isSPCompRunnable = DiagnosticState.Error;
      return;
    }
    this.isSPCompSet = DiagnosticState.OK;
    fs.stat(this.compilerPath, (err, _stats) => {
      if (err) {
        this.isSPCompInstalled = DiagnosticState.Error;
        this.isSPCompRunnable = DiagnosticState.Error;
        return;
      }
      if (!_stats?.isFile()) {
        this.isSPCompInstalled = DiagnosticState.Error;
        this.isSPCompRunnable = DiagnosticState.Error;
        return;
      }
      this.isSPCompInstalled = DiagnosticState.OK;
      let command = this.compilerPath;
      let args = ["-h"];
      if (process.arch === "arm64" && process.platform === "darwin") {
        command = "arch";
        args = ["-x86_64", this.compilerPath, "-h"];
      }
      execFile(command, args, (err, stdout, stderr) => {
        if (err) {
          let match = stdout.match(/SourcePawn Compiler (.+)/);
          if (match !== undefined) {
            this.isSPCompRunnable = DiagnosticState.OK;
            this.SPCompVersion = match[1];
            return;
          }
          this.isSPCompRunnable = DiagnosticState.Error;
          return;
        }
        this.isSPCompRunnable = DiagnosticState.OK;
      });
    });
  }

  includeDirectoriesToWebView(): string {
    const diagnostics = [];
    switch (this.isSMInstalled) {
      case DiagnosticState.OK:
        diagnostics.push(
          '✅ "SourcePawnLanguageServer.includeDirectories" contains at least one entry that contains "sourcemod.inc".'
        );
        break;
      case DiagnosticState.Error:
        diagnostics.push(
          '❌ "SourcePawnLanguageServer.includeDirectories" contains at least one invalid entry".'
        );
        break;
      case DiagnosticState.Warning:
        diagnostics.push(
          '⚠️ "SourcePawnLanguageServer.includeDirectories" contains at least one entry that was not scanned properly.'
        );
        break;
      case DiagnosticState.None:
        diagnostics.push(
          '🩺 Checking if "SourcePawnLanguageServer.includeDirectories" is set.'
        );
        break;
    }

    return diagnostics.map((d) => `<li>${d}</li>`).join("\n");
  }

  async checkIncludeDirectories() {
    const includeDirectories: string[] = getConfig(
      Section.LSP,
      "includeDirectories"
    );
    if (!includeDirectories) {
      this.isSMInstalled = DiagnosticState.Error;
      return;
    }
    includeDirectories.forEach((dir) => {
      if (this.isSMInstalled !== DiagnosticState.None) return;
      fs.stat(dir, (err, _stats) => {
        if (err) {
          this.isSMInstalled = DiagnosticState.Warning;
          return;
        }
        if (!_stats?.isDirectory()) {
          this.isSMInstalled = DiagnosticState.Error;
          return;
        }
        fs.readdir(dir, (err, files) => {
          if (err) {
            this.isSMInstalled = DiagnosticState.Error;
            return;
          }
          files.forEach((file) => {
            if (file === "sourcemod.inc") {
              this.isSMInstalled = DiagnosticState.OK;
              return;
            }
          });
        });
      });
    });
  }

  async checkSettings() {
    this.checkSpComp();
    this.checkIncludeDirectories();
  }
}

export function buildDoctorStatusBar() {
  const doctorStatusBar = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left
  );
  doctorStatusBar.show();
  doctorStatusBar.tooltip = new vscode.MarkdownString(
    "SourcePawn Doctor helps you diagnose why the extension is not working.",
    true
  );
  doctorStatusBar.tooltip.isTrusted = true;
  doctorStatusBar.text = "$(lightbulb-autofix) SourcePawn Doctor";
  doctorStatusBar.command = "sourcepawn-vscode.doctor";
}
