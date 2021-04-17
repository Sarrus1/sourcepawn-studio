import * as vscode from 'vscode';

const indentSize:number=5;

class SpDocCompletionItem extends vscode.CompletionItem {
	constructor(
		position: vscode.Position,
		FunctionDesc: string[]
	) {
		super('/** */', vscode.CompletionItemKind.Text);
		FunctionDesc.shift();
		let snippet = new vscode.SnippetString();
		let max = getMaxLength(FunctionDesc);
		snippet.appendText("/**\n * ");
		snippet.appendPlaceholder("Description");
		snippet.appendText("\n *");
		for(let arg of FunctionDesc)
		{
			snippet.appendText("\n * @param "+arg+" ".repeat(getSpaceLength(arg, max)));
			snippet.appendPlaceholder("Param description");
		}
		snippet.appendText("\n * @return "+" ".repeat(getSpaceLengthReturn(max)));
		snippet.appendPlaceholder("Return description");
		snippet.appendText("\n */");
		this.insertText = snippet;
		let start:vscode.Position = new vscode.Position(position.line, 0);
		let end:vscode.Position = new vscode.Position(position.line, 0);
		this.range = new vscode.Range(start, end);
	}
}

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
		let DocCompletionItem = new SpDocCompletionItem(position, FunctionDesc);
		return[DocCompletionItem];
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
	}
}

function getMaxLength(arr:string[]):number{
	let max:number=0;
	for (let str of arr){
		if(str.length>max) max=str.length;
	}
	return max;
}

function getSpaceLength(str:string, max:number):number{
	return (max+indentSize)-str.length;
}

function getSpaceLengthReturn(max):number{
	return (max+indentSize-1)
}