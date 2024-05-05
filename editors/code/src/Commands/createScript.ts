import {
  workspace as Workspace,
  window,
  extensions,
  workspace,
} from "vscode";
import {
  existsSync,
  readFileSync,
  copyFileSync,
  writeFileSync,
  mkdirSync,
} from "fs";
import { URI } from "vscode-uri";
import { join, basename } from "path";
import { editConfig, getConfig, Section } from "../configUtils";

export function run(rootpath?: string): void {
  const authorName: string = getConfig(Section.SourcePawn, "AuthorName");
  if (!authorName) {
    window
      .showWarningMessage("You didn't specify an author name.", "Open Settings")
      .then((choice) => {
        if (choice === "Open Settings") {
          editConfig(Section.SourcePawn, "AuthorName")
        }
      });
  }

  const githubName: string = getConfig(Section.SourcePawn, "GithubName");
  if (!githubName) {
    window
      .showWarningMessage("You didn't specify a GitHub name.", "Open Settings")
      .then((choice) => {
        if (choice === "Open Settings") {
          editConfig(Section.SourcePawn, "GithubName")
        }
      });
  }

  // Get workspace folder
  const workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    window.showErrorMessage("No workspaces are opened.");
    return;
  }

  // Select the rootpath
  if (!rootpath) {
    rootpath = workspaceFolders?.[0].uri.fsPath;
  }

  // Create a scripting folder if it doesn't exist
  const scriptingFolderPath = join(rootpath, "scripting");
  if (!existsSync(scriptingFolderPath)) {
    mkdirSync(scriptingFolderPath);
  }

  // Check if file already exists
  const scriptFileName: string = basename(rootpath) + ".sp";
  const scriptFilePath = join(rootpath, "scripting", scriptFileName);
  if (existsSync(scriptFilePath)) {
    window.showErrorMessage(scriptFileName + " already exists.");
    return;
  }
  const myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  const tasksTemplatesPath: string = join(
    myExtDir,
    "templates/plugin_template.sp"
  );
  copyFileSync(tasksTemplatesPath, scriptFilePath);

  // Replace placeholders
  try {
    const data = readFileSync(scriptFilePath, "utf8");
    let result = data.replace(/\${AuthorName}/gm, authorName);
    result = result.replace(/\${plugin_name}/gm, basename(rootpath));
    result = result.replace(/\${GithubName}/gm, githubName);
    writeFileSync(scriptFilePath, result, "utf8");
  } catch (err) {
    console.error(err);
    return;
  }
  workspace
    .openTextDocument(URI.file(scriptFilePath))
    .then((document) => window.showTextDocument(document));
}
