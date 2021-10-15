import { workspace as Workspace, window, commands } from "vscode";
import { basename, extname, join } from "path";
import { existsSync, mkdirSync } from "fs";
import { platform } from "os";
import { run as uploadToServerCommand } from "./uploadToServer";

export async function run(args: any) {
  let activeDocumentPath: string;
  let workspaceFolder = Workspace.getWorkspaceFolder(args);
  let mainPath: string =
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get<string>(
      "MainPath"
    ) || "";
  let mainPathCompile: boolean = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<boolean>("MainPathCompilation");
  try {
    activeDocumentPath =
      mainPathCompile && mainPath != "" ? mainPath : args.document.uri.fsPath;
  } catch {
    activeDocumentPath =
      mainPathCompile && mainPath != ""
        ? mainPath
        : window.activeTextEditor.document.uri.fsPath;
  }
  let scriptingPath = activeDocumentPath.replace(/[\w\-. ]+$/, "");
  let activeDocumentName = basename(activeDocumentPath);
  activeDocumentName = activeDocumentName.replace(".sp", ".smx");
  let activeDocumentExt = extname(activeDocumentPath);

  // Don't compile if it's not a .sp file.
  if (activeDocumentExt != ".sp") {
    window.showErrorMessage("Not a .sp file, aborting");
    return;
  }
  const spcomp =
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get<string>(
      "SpcompPath"
    ) || "";

  if (!spcomp) {
    window
      .showErrorMessage(
        "SourceMod compiler not found in the project. You need to set the spCompPath setting to be able to compile a plugin.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return;
  }

  // Open a terminal window
  let terminals = window.terminals;
  let terminal;
  // Try to open current terminal window instead of opening a new one.
  if (!terminals) {
    terminal = window.createTerminal("SourcePawn compile");
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
      terminal = window.createTerminal("SourcePawn compile");
    }
  }
  terminal.show();

  // Create plugins folder if it doesn't exist.
  let pluginsFolderPath: string;
  if (scriptingPath.replace(/(?:\\\\|\\)$/, "").endsWith("scripting")) {
    pluginsFolderPath = join(scriptingPath, "../", "plugins/");
  } else {
    pluginsFolderPath = join(scriptingPath, "compiled/");
  }
  let outputDir: string =
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
      "outputDirectoryPath"
    ) || "";
  if (outputDir === "") {
    outputDir = pluginsFolderPath;
    if (!existsSync(outputDir)) {
      mkdirSync(outputDir);
    }
  } else {
    if (!existsSync(outputDir)) {
      let workspaceFolder = Workspace.workspaceFolders[0];
      outputDir = join(workspaceFolder.uri.fsPath, outputDir);
      if (!existsSync(outputDir)) {
        window
          .showErrorMessage(
            "The output directory does not exist.",
            "Open Settings"
          )
          .then((choice) => {
            if (choice === "Open Settings") {
              commands.executeCommand(
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
  let command = (platform() == "win32" ? "." : "").concat(
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
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
      "SourcemodHome"
    ) || "",
    "'",
    " -i=" + "'",
    join(scriptingPath, "include") || "",
    "'",
    " -i=" + "'",
    scriptingPath,
    "'"
  );
  let compilerOptions: string[] = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("compilerOptions");
  // Add a space at the beginning of every element, for security.
  for (let i = 0; i < compilerOptions.length; i++) {
    command += " " + compilerOptions[i];
  }

  let includes_dirs: string[] = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("optionalIncludeDirsPaths");
  // Add the optional includes folders.
  for (let includes_dir of includes_dirs) {
    if (includes_dir != "") {
      command += " -i=" + "'" + includes_dir + "'";
    }
  }

  try {
    terminal.sendText(command);
    if (
      Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
        "uploadAfterSuccessfulCompile"
      )
    ) {
      await uploadToServerCommand(undefined);
    }
  } catch (error) {
    console.log(error);
  }
}
