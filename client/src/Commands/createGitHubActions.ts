import { workspace as Workspace, window, extensions } from "vscode";
import {
  existsSync,
  mkdirSync,
  copyFileSync,
  readFileSync,
  writeFileSync,
} from "fs";
import { basename, join } from "path";

export function run(rootpath: string = undefined) {
  // get workspace folder
  let workspaceFolders = Workspace.workspaceFolders;
  if (!workspaceFolders) {
    let err: string = "No workspace are opened.";
    window.showErrorMessage(err);
    console.log(err);
    return 1;
  }

  //Select the rootpath
  if (typeof rootpath === "undefined") {
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

  // Check if master.yml already exists
  let masterFilePath = join(rootpath, ".github/workflows/master.yml");
  if (existsSync(masterFilePath)) {
    let err: string = "master.yml already exists, aborting.";
    window.showErrorMessage(err);
    console.log(err);
    return 2;
  }
  let myExtDir: string = extensions.getExtension("Sarrus.sourcepawn-vscode")
    .extensionPath;
  let tasksTemplatesPath: string = join(
    myExtDir,
    "templates/master_template.yml"
  );
  copyFileSync(tasksTemplatesPath, masterFilePath);

  // Replace placeholders
  try {
    let result = readFileSync(masterFilePath, "utf8");
    result = result.replace(/\${plugin_name}/gm, rootname);
    writeFileSync(masterFilePath, result, "utf8");
  } catch (err) {
    console.log(err);
    return 3;
  }
  return 0;
}
