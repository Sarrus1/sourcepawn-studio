import { workspace as Workspace, window, extensions } from "vscode";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { basename, join } from "path";

export function run(rootpath?: string): void {
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

  // Check if main.yml already exists
  let masterFilePath = join(rootpath, ".github/workflows/main.yml");
  if (existsSync(masterFilePath)) {
    window.showErrorMessage("main.yml file already exists.");
    return;
  }

  // Create .github folder if it doesn't exist
  let masterFolderPath = join(rootpath, ".github");
  if (!existsSync(masterFolderPath)) {
    mkdirSync(masterFolderPath);
  }

  // Create workflows folder if it doesn't exist
  masterFolderPath = join(rootpath, ".github", "workflows");
  if (!existsSync(masterFolderPath)) {
    mkdirSync(masterFolderPath);
  }

  // Read template
  const myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  let tasksTemplatesPath: string = join(
    myExtDir,
    "templates/main_template.yml"
  );
  let result = readFileSync(tasksTemplatesPath, "utf-8");

  // Replace placeholders
  try {
    result = result.replace(/\${plugin_name}/gm, basename(rootpath));
    writeFileSync(masterFilePath, result, "utf8");
  } catch (err) {
    window.showErrorMessage("Failed to write to main.yml! " + err);
    return;
  }

  // Check if test.yml already exists
  masterFilePath = join(rootpath, ".github/workflows/test.yml");
  if (existsSync(masterFilePath)) {
    window.showErrorMessage("test.yml file already exists.");
    return;
  }

  tasksTemplatesPath = join(myExtDir, "templates/test_template.yml");
  result = readFileSync(tasksTemplatesPath, "utf-8");

  // Replace placeholders
  try {
    result = result.replace(/\${plugin_name}/gm, basename(rootpath));
    writeFileSync(masterFilePath, result, "utf8");
  } catch (err) {
    window.showErrorMessage("Failed to write to test.yml! " + err);
    return;
  }

  window.showInformationMessage("Github Actions files created successfully!");
}
