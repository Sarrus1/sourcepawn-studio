import {
  workspace as Workspace,
  window,
  commands,
  OutputChannel,
} from "vscode";
import { URI } from "vscode-uri";
import { basename, extname, join, dirname, resolve } from "path";
import { existsSync, mkdirSync } from "fs";
import { execFile } from "child_process";

import { run as uploadToServerCommand } from "./uploadToServer";
import { run as refreshPluginsCommand } from "./refreshPlugins";
import { getCtxFromUri } from "../spIndex";
import { ProjectMainPathParams, projectMainPath } from "../lsp_ext";

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

  const alwaysCompileMainPath: boolean = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get<boolean>("MainPathCompilation");

  // Decide which file to compile here.
  let fileToCompilePath: string;
  if (alwaysCompileMainPath) {
    const params: ProjectMainPathParams = { uri: uri.toString() };
    const mainUri = await getCtxFromUri(uri)?.client.sendRequest(
      projectMainPath,
      params
    );
    if (mainUri === undefined) {
      fileToCompilePath = uri.fsPath;
    } else {
      fileToCompilePath = URI.parse(mainUri).fsPath;
    }
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
    Workspace.getConfiguration(
      "SourcePawnLanguageServer",
      workspaceFolder
    ).get<string>("spcompPath") || "";

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
      const workspaceFolder = Workspace.workspaceFolders[0];
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
  const compilerArguments: string[] = Workspace.getConfiguration(
    "sourcepawn",
    workspaceFolder
  ).get("compilerArguments");

  const includePaths: string[] = [
    join(scriptingFolderPath, "include"),
    scriptingFolderPath,
  ];

  Workspace.getConfiguration("SourcePawnLanguageServer", workspaceFolder)
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
    const ctx = getCtxFromUri(uri);
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
    console.error(error);
  }
}
