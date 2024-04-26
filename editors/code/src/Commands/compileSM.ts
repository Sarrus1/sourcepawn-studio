import {
  workspace,
  window,
  commands,
  OutputChannel,
  ViewColumn,
} from "vscode";
import { URI } from "vscode-uri";
import { basename, join, dirname, resolve } from "path";
import { existsSync, mkdirSync } from "fs";
import { execFile } from "child_process";

import { run as uploadToServerCommand } from "./uploadToServer";
import { run as runServerCommands } from "./runServerCommands";
import { getCtxFromUri, lastActiveEditor } from "../spIndex";
import { alwaysCompileMainPath, getMainCompilationFile, isSPFile } from "../spUtils";

// Create an OutputChannel variable here but do not initialize yet.
let output: OutputChannel;

/**
 * Callback for the Compile file command.
 * @param  {URI} args URI of the document to be compiled. This will be overrided if MainPathCompilation is set to true.
 * @returns Promise
 */
export async function run(args: URI): Promise<void> {
  let fileToCompilePath: string;

  // If we always compile the main path, we always ignore the path of the current editor
  if (await alwaysCompileMainPath()) {
    fileToCompilePath = await getMainCompilationFile()
  }
  // Else, we take the arguments, or we take the last active editor's path
  else {
    if (args) {
      fileToCompilePath = args.fsPath;
    }
    else {
      fileToCompilePath = lastActiveEditor.document.uri.fsPath;
    }
  }

  // Don't compile if it's not a .sp file.
  if (!isSPFile(fileToCompilePath)) {
    window.showErrorMessage("Not a .sp file, aborting");
    return;
  }

  const workspaceFolder = workspace.getWorkspaceFolder(URI.file(fileToCompilePath));
  const scriptingFolderPath = dirname(fileToCompilePath);

  // Invoke the compiler.
  const spcomp =
    workspace.getConfiguration(
      "SourcePawnLanguageServer",
      workspaceFolder
    ).get<string>("spcompPath") || "";

  // Return if compiler not found
  if (!spcomp) {
    window
      .showErrorMessage(
        "Sourcemod compiler not found in the project. You need to set the spCompPath setting to be able to compile a plugin.",
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
    workspace.getConfiguration("sourcepawn", workspaceFolder).get(
      "outputDirectoryPath"
    ) || pluginsFolderPath;
  if (outputDir === pluginsFolderPath) {
    if (!existsSync(outputDir)) {
      mkdirSync(outputDir);
    }
  } else {
    // If the outputDirectoryPath setting is not empty, make sure it exists before trying to write to it.
    if (!existsSync(outputDir)) {
      const workspaceFolder = workspace.workspaceFolders[0];
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
  outputDir += basename(fileToCompilePath, ".sp") + ".smx";

  // Add the compiler options from the settings.
  const compilerArguments: string[] = workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("compilerArguments");

  const includePaths: string[] = [
    join(scriptingFolderPath, "include"),
    scriptingFolderPath,
  ];

  workspace.getConfiguration("SourcePawnLanguageServer", workspaceFolder)
    .get<string[]>("includeDirectories")
    .map((e) =>
      resolve(
        workspaceFolder === undefined ? "" : workspaceFolder.uri.fsPath,
        e
      )
    )
    .forEach((e) => includePaths.push(e));

  let compilerArgs = [fileToCompilePath, `-o${outputDir}`];

  // Add include paths and compiler options to compiler args.
  includePaths.forEach((path) => compilerArgs.push(`-i${path}`));
  compilerArgs = compilerArgs.concat(compilerArguments);

  // Create Output Channel if it does not exist.
  if (!output) {
    output = window.createOutputChannel("SourcePawn Compiler");
  }

  // Clear previous data in Output Channel and show it.
  output.clear();
  output.show();

  try {
    const ctx = getCtxFromUri(URI.file(fileToCompilePath));
    ctx?.setSpcompStatus({ quiescent: false });
    // Compile in child process.
    let spcompCommand = spcomp;
    if (process.platform === "darwin" && process.arch === "arm64") {
      spcompCommand = "arch";
      compilerArgs.unshift("-x86_64", spcomp);
    }
    let command = spcompCommand;
    compilerArgs.forEach((e) => {
      command += e + " ";
      if (e.length > 10) {
        command += "\n";
      }
    });
    output.appendLine(`${command}\n`);
    execFile(spcompCommand, compilerArgs, async (error, stdout) => {
      if (error) {
        console.error(error);
      }
      ctx?.setSpcompStatus({ quiescent: true });
      output.append(stdout.toString().trim());
      if (
        workspace.getConfiguration("sourcepawn", workspaceFolder).get(
          "uploadAfterSuccessfulCompile"
        )
      ) {
        await uploadToServerCommand(fileToCompilePath);
      }
      if (
        workspace.getConfiguration("sourcepawn", workspaceFolder).get<string>(
          "runServerCommands"
        ) === "afterCompile"
      ) {
        await runServerCommands(fileToCompilePath);
      }
    });
  } catch (error) {
    console.error(error);
  }

  window.showTextDocument(lastActiveEditor.document, ViewColumn.Active);
}
