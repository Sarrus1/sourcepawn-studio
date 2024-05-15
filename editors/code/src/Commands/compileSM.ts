import {
  workspace as Workspace,
  window,
  OutputChannel,
} from "vscode";
import { URI } from "vscode-uri";
import { basename, join, dirname, resolve } from "path";
import { existsSync, mkdirSync } from "fs";
import { execFile } from "child_process";

import { run as uploadToServerCommand } from "./uploadToServer";
import { run as runServerCommands } from "./runServerCommands";
import { getCtxFromUri, lastActiveEditor } from "../spIndex";
import { getMainCompilationFile, isSPFile } from "../spUtils";
import { Section as Section, editConfig, getConfig } from "../configUtils";

// Create an OutputChannel variable here but do not initialize yet.
let output: OutputChannel;

/**
 * Callback for the Compile file command.
 * @param  {URI} args URI of the document to be compiled. This will be overrided if MainPathCompilation is set to true.
 * @returns Promise
 */
export async function run(args: URI): Promise<number> {
  let fileToCompilePath: string;

  // If we always compile the main path, we always ignore the path of the current editor
  const workspaceFolder = Workspace.getWorkspaceFolder(lastActiveEditor.document.uri);
  const compileMainPath: boolean = getConfig(Section.SourcePawn, "MainPathCompilation", workspaceFolder);
  if (compileMainPath) {
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
    window.showErrorMessage("The current file is not a SourcePawn script.");
    return 1;
  }

  // Return if compiler not found
  const spcomp: string = getConfig(Section.LSP, "compiler.path", workspaceFolder);
  if (!spcomp) {
    window
      .showErrorMessage(
        "Sourcemod compiler not found in the project. You need to set the 'compiler.path' setting to be able to compile a plugin.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          editConfig(Section.LSP, "compiler.path")
        }
      });
    return 1;
  }

  // Decide where to output the compiled file.
  const scriptingFolderPath = dirname(fileToCompilePath);
  const pluginsFolderPath = join(scriptingFolderPath, "../", "plugins/");
  let outputDir: string = getConfig(Section.SourcePawn, "outputDirectoryPath", workspaceFolder) || pluginsFolderPath;
  if (outputDir === pluginsFolderPath && !existsSync(outputDir)) {
    mkdirSync(outputDir);
  } else {
    // If the outputDirectoryPath setting is not empty, make sure it exists before trying to write to it.
    if (!existsSync(outputDir)) {
      const workspaceFolder = Workspace.workspaceFolders[0];
      outputDir = join(workspaceFolder.uri.fsPath, outputDir);
      if (!existsSync(outputDir)) {
        window.showErrorMessage(
          "The output directory does not exist.",
          "Open Settings"
        )
          .then((choice) => {
            if (choice === "Open Settings") {
              editConfig(Section.SourcePawn, "outputDirectoryPath");
            }
          });
        return 1;
      }
    }
  }
  outputDir += basename(fileToCompilePath, ".sp") + ".smx";

  // Add the compiler options from the settings.
  const compilerArguments: string[] = getConfig(Section.LSP, "compiler.arguments", workspaceFolder);
  const includePaths: string[] = [
    join(scriptingFolderPath, "include"),
    scriptingFolderPath,
  ];

  const includeDirs: string[] = getConfig(Section.LSP, "includeDirectories", workspaceFolder);
  includeDirs.map((e) =>
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
    // Set spcomp status
    const ctx = getCtxFromUri(URI.file(fileToCompilePath));
    ctx?.setSpcompStatus({ quiescent: false });

    // Set up compiler command and args
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

    // Execute
    execFile(spcompCommand, compilerArgs, async (error, stdout) => {
      // Update spcomp status
      ctx?.setSpcompStatus({ quiescent: true });
      output.appendLine(stdout.toString().trim() + '\n');

      // Restore last active editor's focus
      window.showTextDocument(lastActiveEditor.document);

      // Return if compilation failed
      if (error) {
        window.showErrorMessage("Compilation failed.")
        return 1;
      }

      // Little success message in console
      output.appendLine("Compilation successful.");

      let uploadSuccessful: boolean = false;

      // Run upload command if chosen
      if (getConfig(Section.SourcePawn, "uploadAfterSuccessfulCompile", workspaceFolder)) {
        uploadSuccessful = await uploadToServerCommand(fileToCompilePath);
      }

      // Run server commands if chosen
      const commandsOption: string = getConfig(Section.SourcePawn, "runServerCommands", workspaceFolder);
      if (uploadSuccessful && commandsOption === "afterCompile") {
        await runServerCommands(fileToCompilePath);
      }

      return 0;
    });
  } catch (error) {
    console.error(error);
    return 1;
  }

  return 0;
}
