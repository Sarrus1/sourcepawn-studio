import * as vscode from 'vscode';

const defaultJsDoc = new vscode.SnippetString(`/**\n * $0\n */`);

// class JsDocCompletionItem extends vscode.CompletionItem {
// 	constructor(
// 		public readonly document: vscode.TextDocument,
// 		public readonly position: vscode.Position
// 	) {
		
// 		super('/** */', vscode.CompletionItemKind.Text);
// 		this.sortText = '\0';

// 		const line = document.lineAt(position.line).text;
// 		const prefix = line.slice(0, position.character).match(/\/\**\s*$/);
// 		const suffix = line.slice(position.character).match(/^\s*\**\//);
// 		const start = position.translate(0, prefix ? -prefix[0].length : 0);
// 		const range = new vscode.Range(start, position.translate(0, suffix ? suffix[0].length : 0));
// 		this.range = { inserting: range, replacing: range };
// 	}
// }

export class JsDocCompletionProvider implements vscode.CompletionItemProvider {

	public async provideCompletionItems(
		document: vscode.TextDocument,
		position: vscode.Position,
		token: vscode.CancellationToken
	): Promise<vscode.CompletionItem[] | undefined> {
		if (!document) {
			return undefined;
		}
		let FunctionDesc:string[] = this.getFunctionArgs(document, position);
		if (FunctionDesc==[])
		{
			return undefined;
		}
		FunctionDesc.shift();
		let snippet = new vscode.SnippetString();
		snippet.appendText("/**\n * ");
		snippet.appendPlaceholder("Description");
		snippet.appendText("\n *");
		for(let arg of FunctionDesc)
		{
			snippet.appendText("\n * @param "+arg+"\t\t\t\t\t");
			snippet.appendPlaceholder("Param description");
		}
		snippet.appendText("\n * @return \t\t\t\t\t\t\t\t");
		snippet.appendPlaceholder("Return description");
		snippet.appendText("\n */");
		let completionItem = new vscode.CompletionItem("/** */", vscode.CompletionItemKind.Text);
		completionItem.insertText = snippet;
		let start:vscode.Position = new vscode.Position(position.line, 0);
		let end:vscode.Position = new vscode.Position(position.line, 0);
		completionItem.range = new vscode.Range(start, end);
		return[completionItem];
	}

	private getFunctionArgs(
		document: vscode.TextDocument,
		position: vscode.Position
	): string[] {
		const lines = document.getText().split("\n");
		let old_style:boolean;
		let line = lines[position.line+1];
		let match = line.match(/(?:(?:static|native|stock|public|forward)+\s*)+\s+(?:[a-zA-Z\-_0-9]:)?([^\s]+)\s*([A-Za-z_]*)\(([^\)]*)(?:\)?)(?:\s*)(?:\{?)(?:\s*)(?:[^\;\s]*)$/);
		if(!match) return [];
		let match_buffer = "";
    let name_match = "";
    let params_match = [];
    // Separation for old and new style functions
    // New style
    if (match[2] != "") {
			old_style = false;
      name_match = match[2];
    }
    // Old style
    else {
			old_style = true;
      name_match = match[1];
    }
    match_buffer = match[0];
    // Check if function takes arguments
    let maxiter = 0;
    while (
      !match_buffer.match(/(\))(?:\s*)(?:;)?(?:\s*)(?:\{?)(?:\s*)$/) &&
      maxiter < 20
    ) {
			line = lines.shift();
      if (typeof line === "undefined") {
        break;
      }
      //partial_params_match += line;
      match_buffer += line;
      maxiter++;
    }
		params_match = match_buffer.match(/([A-z_0-9.]*)(?:\)|,)/gm);
    let params:string[] = [];
		for (let param of params_match) {
			params.push(param.replace(",","").replace(")", ""));
		}
		return([name_match].concat(params))

		// Only show the JSdoc completion when the everything before the cursor is whitespace
		// or could be the opening of a comment
		// const line = document.lineAt(position.line).text;
		// const prefix = line.slice(0, position.character);
		// if (!/^\s*$|\/\*\*\s*$|^\s*\/\*\*+\s*$/.test(prefix)) {
		// 	return false;
		// }

		// // And everything after is possibly a closing comment or more whitespace
		// const suffix = line.slice(position.character);
		// return /^\s*(\*+\/)?\s*$/.test(suffix);
	}
}

export function templateToSnippet(template: string): vscode.SnippetString {
	let snippetIndex = 1;
	template = template.replace(/\$/g, '\\$');
	template = template.replace(/^[ \t]*(?=(\/|[ ]\*))/gm, '');
	template = template.replace(/^(\/\*\*\s*\*[ ]*)$/m, (x) => x + `\$0`);
	template = template.replace(/\* @param([ ]\{\S+\})?\s+(\S+)[ \t]*$/gm, (_param, type, post) => {
		let out = '* @param ';
		if (type === ' {any}' || type === ' {*}') {
			out += `{\$\{${snippetIndex++}:*\}} `;
		} else if (type) {
			out += type + ' ';
		}
		out += post + ` \${${snippetIndex++}}`;
		return out;
	});

	template = template.replace(/\* @returns[ \t]*$/gm, `* @returns \${${snippetIndex++}}`);

	return new vscode.SnippetString(template);
}
/*
export function register(
	selector: DocumentSelector,
	modeId: string,
	client: ITypeScriptServiceClient,
	fileConfigurationManager: FileConfigurationManager,

): vscode.Disposable {
	return conditionalRegistration([
		requireConfiguration(modeId, 'suggest.completeJSDocs')
	], () => {
		return vscode.languages.registerCompletionItemProvider(selector.syntax,
			new JsDocCompletionProvider(client, fileConfigurationManager),
			'*');
	});
}
*/