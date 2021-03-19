import vscode = require("vscode");
import * as fs from "fs";
import * as path from 'path';

export async function run(args: any) {
	vscode.workspace.workspaceFolders
	// TODO: Try to compile .sp file if found in scripting folder, otherwise, tell to use tasks.
}