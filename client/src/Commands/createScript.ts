import vscode = require("vscode");
import * as fs from "fs";
import * as path from "path";

export function run(rootpath: string = undefined) {
  let AuthorName: string = vscode.workspace
    .getConfiguration("sourcepawn")
    .get("AuthorName");
  if (!AuthorName) {
    vscode.window
      .showWarningMessage("You didn't specify an author name.", "Open Settings")
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand(
						"workbench.action.openSettings",
						"@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
  }

  let GithubName: string = vscode.workspace
    .getConfiguration("sourcepawn")
    .get("GithubName");

  // get workspace folder
  let workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders) {
    vscode.window.showErrorMessage("No workspace are opened.");
    return 1;
  }

  //Select the rootpath
	if(typeof rootpath === "undefined"){
		rootpath = workspaceFolders?.[0].uri.fsPath;
	}
  
  let rootname = path.basename(rootpath);

  // create a scripting folder if it doesn't exist
  let scriptingFolderPath = path.join(rootpath, "scripting");
  if (!fs.existsSync(scriptingFolderPath)) {
    fs.mkdirSync(scriptingFolderPath);
  }

  // Check if file already exists
  let scriptFileName: string = rootname + ".sp";
  let scriptFilePath = path.join(rootpath, "scripting", scriptFileName);
  if (fs.existsSync(scriptFilePath)) {
    vscode.window.showErrorMessage(
      scriptFileName + " already exists, aborting."
    );
    return 2;
  }
  let myExtDir: string = vscode.extensions.getExtension(
    "Sarrus.sourcepawn-vscode"
  ).extensionPath;
  let tasksTemplatesPath: string = path.join(
    myExtDir,
    "templates/plugin_template.sp"
  );
  fs.copyFileSync(tasksTemplatesPath, scriptFilePath);

  // Replace placeholders
  try {
    let data = fs.readFileSync(scriptFilePath, "utf8");
    let result = data.replace(/\${AuthorName}/gm, AuthorName);
    result = result.replace(/\${plugin_name}/gm, rootname);
    result = result.replace(/\${GithubName}/gm, GithubName);
    fs.writeFileSync(scriptFilePath, result, "utf8");
  } catch (err) {
    console.log(err);
    return 3;
  }
  return 0;
}
