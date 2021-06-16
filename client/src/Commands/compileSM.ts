import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";

export async function run(args: any) {
  let activeDocumentPath: string;
  try {
    activeDocumentPath = args.document.uri.fsPath;
  } catch {
    activeDocumentPath = vscode.window.activeTextEditor.document.uri.fsPath;
  }
  let scriptingPath = path.dirname(activeDocumentPath);
  let activeDocumentName = path.basename(activeDocumentPath);
  activeDocumentName = activeDocumentName.replace(".sp", ".smx");
  let activeDocumentExt = path.extname(activeDocumentPath);

  // Don't compile if it's not a .sp file.
  if (activeDocumentExt != ".sp") {
    vscode.window.showErrorMessage("Not a .sp file, aborting");
    return;
  }
  const spcomp =
    vscode.workspace.getConfiguration("sourcepawn").get<string>("SpcompPath") ||
    "";

  if (!spcomp) {
    vscode.window
      .showErrorMessage(
        "SourceMod compiler not found in the project. You need to set the spCompPath setting to be able to compile a plugin.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand(
            "workbench.action.openSettings",
						"@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return;
  }

  // Open a terminal window
  let terminals = vscode.window.terminals;
  let terminal;
  // Try to open current terminal window instead of opening a new one.
  if (!terminals) {
    terminal = vscode.window.createTerminal("SourcePawn compile");
  } else {
    let found: boolean = false;
    for (let terminal_elt of terminals) {
      if (terminal_elt.name.includes("SourcePawn compile")) {
        terminal = terminal_elt;
        found = true;
        break;
      }
    }
    if (!found) {
      terminal = vscode.window.createTerminal("SourcePawn compile");
    }
  }
  terminal.show();

  // Create plugins folder if it doesn't exist.
  let pluginsFolderPath: string;
  if (scriptingPath.endsWith("scripting")) {
    pluginsFolderPath = path.join(scriptingPath, "../", "plugins/");
  } else {
    pluginsFolderPath = path.join(scriptingPath, "compiled/");
  }
  if (!fs.existsSync(pluginsFolderPath)) {
    fs.mkdirSync(pluginsFolderPath);
  }
  let outputDir: string =
    vscode.workspace
      .getConfiguration("sourcepawn")
      .get("outputDirectoryPath") || "";
  if (outputDir == "") {
    outputDir = pluginsFolderPath;
  } else {
    if (!fs.existsSync(outputDir)) {
      let workspaceFolder = vscode.workspace.workspaceFolders[0];
      outputDir = path.join(workspaceFolder.uri.fsPath, outputDir);
      if (!fs.existsSync(outputDir)) {
        vscode.window
          .showErrorMessage(
            "The output directory does not exist.",
            "Open Settings"
          )
          .then((choice) => {
            if (choice === "Open Settings") {
              vscode.commands.executeCommand(
								"workbench.action.openSettings",
								"@ext:sarrus.sourcepawn-vscode"
              );
            }
          });
        return;
      }
    }
  }
  outputDir += activeDocumentName;
  let command = (os.platform() == "win32" ? "." : "").concat(
    // Compiler path
    "'" + spcomp + "'",

    // Seperate compiler and script path
    " ",

    // Script path (script to compile)
    "'" + activeDocumentPath + "'",
    // Output path for the smx file
    " -o=" + "'" + outputDir + "'",

    // Set the path for sm_home
    " -i=" + "'",
    vscode.workspace.getConfiguration("sourcepawn").get("SourcemodHome") || "",
    "'",
    " -i=" + "'",
    scriptingPath + "/include" || "",
    "'"
  );
  let compilerOptions: string[] = vscode.workspace
    .getConfiguration("sourcepawn")
    .get("compilerOptions");
  // Add a space at the beginning of every element, for security.
  for (let i = 0; i < compilerOptions.length; i++) {
    command += " " + compilerOptions[i];
  }

  let includes_dirs: string[] = vscode.workspace
    .getConfiguration("sourcepawn")
    .get("optionalIncludeDirsPaths");
  // Add the optional includes folders.
  for (let includes_dir of includes_dirs) {
    if (includes_dir != "") {
      command += " -i=" + "'" + includes_dir + "'";
    }
  }

  try {
    terminal.sendText(command);
    if (
      vscode.workspace
        .getConfiguration("sourcepawn")
        .get("refreshAfterSuccessfulUpload")
    ) {
      await vscode.commands.executeCommand("sourcepawn-vscode.uploadToServer");
    }
  } catch (error) {
    console.log(error);
  }
}
