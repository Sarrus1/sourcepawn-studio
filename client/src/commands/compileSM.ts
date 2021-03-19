import * as vscode from "vscode";
import * as path from 'path';
import * as fs from "fs";
import { execFileSync } from "child_process";

export async function run(args: any) {
	let activeDocumentPath = vscode.window.activeTextEditor.document.uri.fsPath;
	let activeDocumentName = path.basename(activeDocumentPath);
	activeDocumentName = activeDocumentName.replace(".sp", ".smx");
	let activeDocumentExt = path.extname(activeDocumentPath);

	// Don't compile if it's not a .sp file.
	if (activeDocumentExt != ".sp")
	{
		vscode.window.showErrorMessage("Not a .sp file, aborting");
		return;
	}
	const spcomp =
    vscode.workspace.getConfiguration("sourcepawnLanguageServer").get<string>(
      "spcomp_path"
    ) || "";
	
	if(!spcomp)
	{
    vscode.window
      .showErrorMessage(
        "SourceMod compiler not found in the project. You need to set the spcomp path for the Linter to work.",
        "Open Settings"
      )
      .then((choice) => {
        if (choice === "Open Settings") {
          vscode.commands.executeCommand("workbench.action.openWorkspaceSettings");
        }
      });
		return;
	}

	let terminal = vscode.window.createTerminal("SourcePawn compile");
	terminal.show();
	let workspaceFolderPath = vscode.workspace.workspaceFolders?.[0].uri.fsPath || "";
	// Create plugins folder if it doesn't exist.
	let pluginsFolderPath = path.join(workspaceFolderPath, "plugins/");
	if (!fs.existsSync(pluginsFolderPath)){
		fs.mkdirSync(pluginsFolderPath);
	}
	let command = "".concat(spcomp, " ", activeDocumentPath, " ", "-E", // Treat warnings as errors
	" -O2", // Optimization level (0=none, 2=full)
	" -v2", // "erbosity level; 0=quiet, 1=normal, 2=verbose
	// Set the path for sm_home
	" -i" +
		vscode.workspace.getConfiguration("sourcepawnLanguageServer").get(
			"sourcemod_home"
		) || "",
	" -o "+pluginsFolderPath+activeDocumentName // Output path for the smx file
	);
	try{
		terminal.sendText(command);
		// execFileSync(spcomp, [
		// 	activeDocumentPath,
		// 	"-E", // Treat warnings as errors
		// 	"-O2", // Optimization level (0=none, 2=full)
		// 	"-v2", // "erbosity level; 0=quiet, 1=normal, 2=verbose
		// 	// Set the path for sm_home
		// 	"-iD" +
		// 		vscode.workspace.getConfiguration("sourcepawnLanguageServer").get(
		// 			"sourcemod_home"
		// 		) || "",
		// 	"-o"+pluginsFolderPath+activeDocumentName // Output path for the smx file
		// ]);
	}
	catch (error) {
		console.debug(error);
	}

}