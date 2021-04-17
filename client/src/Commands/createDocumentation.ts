import * as vscode from "vscode";
import { URI } from "vscode-uri";

export function run(args: any) {
	// current editor
	const editor = vscode.window.activeTextEditor;

	// check if there is no selection
	if (!editor.selection.isEmpty) {
		const document:vscode.TextDocument=editor.document;
		// the Position object gives you the line and character where the cursor is
		const range:vscode.Range = new vscode.Range(editor.selection.start, editor.selection.end)
		const loc:vscode.Location = new vscode.Location(editor.document.uri, range);
		let lines:string[] = document.getText(range).split("\n");
		let match =  lines[0].match(/(?:(?:static|native|stock|public|forward)+\s*)+\s+(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\(([^\)]*)(?:\)?)(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*)$/);
		if(match){
			let match_buffer = "";
			let line = "";
			let name_match = "";
			let params_match = [];
			// Separation for old and new style functions
			// New style
			if (match[2] != "") {
				name_match = match[2];
			}
			// Old style
			else {
				name_match = match[1];
			}
			match_buffer = match[0];
			// Check if function takes arguments
			let maxiter = 0;
			while (
				!match_buffer.match(/(\))(?:\s*)(?:;)?(?:\s*)(?:\{?)(?:\s*)$/) &&
				maxiter < 20
			) {
				line = this.lines.shift();
				this.lineNb++;
				if (typeof line === "undefined") {
					return;
				}
				//partial_params_match += line;
				match_buffer += line;
				maxiter++;
			}
			let params = [];
			let current_param;
			if (params_match) {
				for (let param of params_match) {
					current_param = {
						label: param,
						documentation: param,
					};
					params.push(current_param);
				}
			}
			for(let param of current_param)
			{
				console.debug(param);
				
			}
		}
	}
}