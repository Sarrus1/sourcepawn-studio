import { workspace as Workspace, window, extensions } from "vscode";
import {
  existsSync,
  mkdirSync,
  copyFileSync,
  readFileSync,
  writeFileSync,
} from "fs";
import { basename, join } from "path";

export function run(rootpath?: string) {
  // get workspace folder
  let workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    let err: string = "No workspace are opened.";
    window.showErrorMessage(err);
    console.log(err);
    return 1;
  }

  //Select the rootpath
  if (rootpath === undefined) {
    rootpath = workspaceFolders?.[0].uri.fsPath;
  }

  let rootname = basename(rootpath);

  // create .github folder if it doesn't exist
  let masterFolderPath = join(rootpath, ".github");
  if (!existsSync(masterFolderPath)) {
    mkdirSync(masterFolderPath);
  }
  // create workflows folder if it doesn't exist
  masterFolderPath = join(rootpath, ".github", "workflows");
  if (!existsSync(masterFolderPath)) {
    mkdirSync(masterFolderPath);
  }

  // Check if main.yml already exists
  let masterFilePath = join(rootpath, ".github/workflows/main.yml");
  if (existsSync(masterFilePath)) {
    let err: string = "main.yml already exists, aborting.";
    window.showErrorMessage(err);
    console.log(err);
    return 2;
  }
  let myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  let tasksTemplatesPath: string = join(
    myExtDir,
    "templates/main_template.yml"
  );
  let result = readFileSync(tasksTemplatesPath, "utf-8");

  // Replace placeholders
  try {
    result = result.replace(/\${plugin_name}/gm, rootname);
    writeFileSync(masterFilePath, result, "utf8");
  } catch (err) {
    console.log(err);
    return 3;
  }

  // Check if test.yml already exists
  masterFilePath = join(rootpath, ".github/workflows/test.yml");
  if (existsSync(masterFilePath)) {
    let err: string = "test.yml already exists, aborting.";
    window.showErrorMessage(err);
    console.log(err);
    return 2;
  }
  tasksTemplatesPath = join(myExtDir, "templates/test_template.yml");
  result = readFileSync(tasksTemplatesPath, "utf-8");

  // Replace placeholders
  try {
    result = result.replace(/\${plugin_name}/gm, rootname);
    writeFileSync(masterFilePath, result, "utf8");
  } catch (err) {
    console.log(err);
    return 4;
  }
  return 0;
}
