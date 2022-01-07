import {
  workspace as Workspace,
  window,
  commands,
  OutputChannel,
} from "vscode";
import { URI } from "vscode-uri";
import { basename, extname, join, dirname } from "path";
import { existsSync, mkdirSync } from "fs";
import { execFile } from "child_process";

import { run as uploadToServerCommand } from "./uploadToServer";
import { getAllPossibleIncludeFolderPaths } from "../Backend/spFileHandlers";
import { findMainPath } from "../spUtils";
import { run as refreshPluginsCommand } from "./refreshPlugins";
import { compilerDiagnostics } from "../spLinter";
import { parseSPCompErrors } from "../Misc/parseSPCompErrors";

// Create an OutputChannel variable here but do not initialize yet.
let output: OutputChannel;

/**
 * Callback for the Compile file command.
 * @param  {URI} args URI of the document to be compiled. This will be overrided if MainPathCompilation is set to true.
 * @returns Promise
 */
export async function run(args: URI): Promise<void> {
  const uri = args === undefined ? window.activeTextEditor.document.uri : args;
  const workspaceFolder = Workspace.getWorkspaceFolder(uri);

  const mainPath = findMainPath(uri);
  const alwaysCompileMainPath: boolean = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<boolean>("MainPathCompilation");

  // Decide which file to compile here.
  let fileToCompilePath: string;
  if (alwaysCompileMainPath && mainPath !== undefined) {
    fileToCompilePath = mainPath;
  } else {
    fileToCompilePath = uri.fsPath;
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

  // Add the compiler options from the settings.
  const compilerOptions: string[] = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("compilerOptions");

  let includePaths: string[] = [
    Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
      "SourcemodHome"
    ),
    join(scriptingFolderPath, "include"),
    scriptingFolderPath,
  ];

  // Add the optional includes folders.
  getAllPossibleIncludeFolderPaths(
    URI.file(fileToCompilePath),
    true
  ).forEach((e) => includePaths.push(e));

  let compilerArgs = [fileToCompilePath, "-o", outputDir];

  // Add include paths and compiler options to compiler args.
  includePaths.forEach((path) => compilerArgs.push("-i", path));
  compilerArgs = compilerArgs.concat(compilerOptions);

  // Create Output Channel if it does not exist.
  if (!output) {
    output = window.createOutputChannel("SourcePawn Compiler");
  }

  // Clear previous data in Output Channel and show it.
  output.clear();
  output.show();

  try {
    // Compile in child process.
    execFile(spcomp, compilerArgs, async (error, stdout) => {
      output.append(stdout.toString().trim());
      parseSPCompErrors(
        stdout.toString().trim(),
        compilerDiagnostics,
        fileToCompilePath
      );
      if (
        Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
          "uploadAfterSuccessfulCompile"
        )
      ) {
        await uploadToServerCommand(URI.file(fileToCompilePath));
      }
      if (
        Workspace.getConfiguration("sourcepawn", workspaceFolder).get<string>(
          "refreshServerPlugins"
        ) === "afterCompile"
      ) {
        refreshPluginsCommand(undefined);
      }
    });
  } catch (error) {
    console.log(error);
  }
}
