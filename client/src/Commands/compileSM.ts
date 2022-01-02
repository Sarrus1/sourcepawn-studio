import { workspace as Workspace, window, commands, Terminal } from "vscode";
import { URI } from "vscode-uri";
import { basename, extname, join, dirname } from "path";
import { existsSync, mkdirSync } from "fs";
import { platform } from "os";

import { run as uploadToServerCommand } from "./uploadToServer";
import { getAllPossibleIncludeFolderPaths } from "../Backend/spFileHandlers";
import { findMainPath } from "../spUtils";
import { run as refreshPluginsCommand } from "./refreshPlugins";

/**
 * Callback for the Compile file command.
 * @param  {URI} args URI of the document to be compiled. This will be overrided if MainPathCompilation is set to true.
 * @returns Promise
 */
export async function run(args: URI): Promise<void> {
  const workspaceFolder = Workspace.getWorkspaceFolder(
    args === undefined ? window.activeTextEditor.document.uri : args
  );
  const mainPath = findMainPath(args);
  const alwaysCompileMainPath: boolean = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<boolean>("MainPathCompilation");

  // Decide which file to compile here.
  let fileToCompilePath: string;
  if (alwaysCompileMainPath && mainPath !== "") {
    fileToCompilePath = mainPath;
  } else if (args !== undefined) {
    fileToCompilePath = args.fsPath;
  } else {
    fileToCompilePath = window.activeTextEditor.document.uri.fsPath;
  }

  const scriptingFolderPath = dirname(fileToCompilePath);

  // Don't compile if it's not a .sp file.
  if (extname(fileToCompilePath) !== ".sp") {
    window.showErrorMessage("Not a .sp file, aborting");
    return;
  }

  // Invoke the compiler.
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

  // Try to reuse the previous terminal window instead of opening a new one.
  const terminals = window.terminals;
  let terminal: Terminal;
  if (terminals.length === 0) {
    terminal = window.createTerminal("SourcePawn compile");
  } else {
    terminal = terminals.find((e) => e.name === "SourcePawn compile");
    if (terminal === undefined) {
      terminal = window.createTerminal("SourcePawn compile");
    }
  }
  terminal.show();

  // Decide where to output the compiled file.
  const pluginsFolderPath = join(scriptingFolderPath, "../", "plugins/");
  let outputDir: string =
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
      "outputDirectoryPath"
    ) || pluginsFolderPath;
  if (outputDir === pluginsFolderPath) {
    if (!existsSync(outputDir)) {
      mkdirSync(outputDir);
    }
  } else {
    // If the outputDirectoryPath setting is not empty, make sure it exists before trying to write to it.
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
  outputDir += basename(fileToCompilePath, ".sp");

  let command = (platform() == "win32" ? "." : "").concat(
    // Compiler path
    "'" + spcomp + "'",
    // Script path (script to compile)
    " '" + fileToCompilePath + "'",
    // Output path for the smx file
    " -o=" + "'" + outputDir + "'",
    // Set the path for sm_home
    " -i=" + "'",
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
      "SourcemodHome"
    ) || "",
    "'",
    " -i=" + "'",
    join(scriptingFolderPath, "include") || "",
    "'",
    " -i=" + "'",
    scriptingFolderPath,
    "'"
  );

  // Add the compiler options from the settings.
  const compilerOptions: string[] = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("compilerOptions");

  // Add a space at the beginning of every element, for security.
  command += " " + compilerOptions.join(" ");

  // Add the optional includes folders.
  getAllPossibleIncludeFolderPaths(URI.file(fileToCompilePath), true).forEach(
    (e) => (command += ` -i='${e}'`)
  );

  try {
    terminal.sendText(command);
    if (
      Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
        "uploadAfterSuccessfulCompile"
      )
    ) {
      await uploadToServerCommand(URI.file(fileToCompilePath));
      if (
        Workspace.getConfiguration("sourcepawn", workspaceFolder).get<string>(
          "refreshServerPlugins"
        ) === "afterCompile"
      ) {
        refreshPluginsCommand(undefined);
      }
    }
  } catch (error) {
    console.log(error);
  }
}
