import {
  CompletionItem,
  CompletionItemKind,
  Position,
  SnippetString,
  Range,
  CompletionItemProvider,
  TextDocument,
  CancellationToken,
} from "vscode";

const indentSize: number = 5;

class SpDocCompletionItem extends CompletionItem {
  constructor(position: Position, FunctionDesc: string[]) {
    super("/** */", CompletionItemKind.Text);
    FunctionDesc.shift();
    let snippet = new SnippetString();
    let max = getMaxLength(FunctionDesc);
    snippet.appendText("/**\n * ");
    snippet.appendPlaceholder("Description");
    snippet.appendText("\n *");
    for (let arg of FunctionDesc) {
      snippet.appendText(
        "\n * @param " + arg + " ".repeat(getSpaceLength(arg, max))
      );
      snippet.appendPlaceholder("Param description");
    }
    snippet.appendText("\n * @return " + " ".repeat(getSpaceLengthReturn(max)));
    snippet.appendPlaceholder("Return description");
    snippet.appendText("\n */");
    this.insertText = snippet;
    let start: Position = new Position(position.line, 0);
    let end: Position = new Position(position.line, 0);
    this.range = new Range(start, end);
  }
}

export class JsDocCompletionProvider implements CompletionItemProvider {
  public async provideCompletionItems(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<CompletionItem[] | undefined> {
    if (!document) {
      return undefined;
    }
    let FunctionDesc: string[] = this.getFunctionArgs(document, position);
    if (FunctionDesc == []) {
      return undefined;
    }
    let DocCompletionItem = new SpDocCompletionItem(position, FunctionDesc);
    return [DocCompletionItem];
  }

  private getFunctionArgs(
    document: TextDocument,
    position: Position
  ): string[] {
    const lines = document.getText().split("\n");
    let old_style: boolean;
    let line = lines[position.line + 1];
    if (typeof line == "undefined") return [];
    let match = line.match(
      /(?:(?:static|native|stock|public|forward)+\s*)+\s+(?:\w:)?([^\s]+)\s*([A-Za-z_]*)\(([^\)]*)(?:\)?)(?:\s*)(?:\{?)(?:\s*)/
    );
    if (!match) return [];
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
    params_match = match_buffer.match(/([A-Za-z_0-9.]*)(?:\)|,)/gm);
    let params: string[] = [];
    for (let param of params_match) {
      params.push(param.replace(",", "").replace(")", ""));
    }
    return [name_match].concat(params);
  }
}

function getMaxLength(arr: string[]): number {
  let max: number = 0;
  for (let str of arr) {
    if (str.length > max) max = str.length;
  }
  return max;
}

function getSpaceLength(str: string, max: number): number {
  return max + indentSize - str.length;
}

function getSpaceLengthReturn(max): number {
  return max + indentSize - 1;
}
