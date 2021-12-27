import {
  CompletionItem,
  CompletionItemKind,
  Position,
  SnippetString,
  Range,
  CompletionItemProvider,
  TextDocument,
  CancellationToken,
  commands,
  SignatureHelp,
} from "vscode";

const indentSize: number = 5;

class SpDocCompletionItem extends CompletionItem {
  constructor(position: Position, FunctionDesc: string[], indent: string) {
    super("/** */", CompletionItemKind.Text);
    let snippet = new SnippetString();
    let max = getMaxLength(FunctionDesc);
    snippet.appendText(`${indent}/**\n ${indent}* `);
    snippet.appendPlaceholder("Description");
    this.appendTextSnippet(snippet, "", indent);
    for (let arg of FunctionDesc) {
      this.appendTextSnippet(
        snippet,
        "@param " + arg + " ".repeat(getSpaceLength(arg, max)),
        indent
      );
      snippet.appendPlaceholder("Param description");
    }
    this.appendTextSnippet(
      snippet,
      "@return " + " ".repeat(getSpaceLengthReturn(max)),
      indent
    );
    snippet.appendPlaceholder("Return description");
    snippet.appendText(`\n${indent}`);
    this.insertText = snippet;
    let start: Position = new Position(position.line, 0);
    let end: Position = new Position(position.line, 0);
    this.range = new Range(start, end);
  }

  private appendTextSnippet(
    snippet: SnippetString,
    text: string,
    indent: string
  ): void {
    snippet.appendText(`\n${indent} * ${text}`);
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
    let { signature, indent } = await getFullParams(document, position);
    if (signature === undefined) {
      return undefined;
    }

    let functionDesc = signature.parameters.map((e) => e.label.toString());

    let DocCompletionItem = new SpDocCompletionItem(
      position,
      functionDesc,
      indent
    );
    return [DocCompletionItem];
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

async function getFullParams(document: TextDocument, position: Position) {
  const lines = document.getText().split("\n");
  let lineNB = position.line + 1;
  let line = lines[lineNB];
  let newSyntaxRe: RegExp = /^(\s)*(?:(?:stock|public|native|forward|static)\s+)*(?:(\w*)\s+)?(\w*)\s*\(/;
  let match: RegExpMatchArray = line.match(newSyntaxRe);
  if (!match) {
    match = line.match(
      /^(\s)*(?:(?:static|native|stock|public|forward)\s+)*(?:(\w+)\s*:)?\s*(\w*)\s*\(/
    );
    if (!match) {
      return;
    }
  }
  let newPos = new Position(lineNB, match[0].length);
  let res: SignatureHelp = await commands.executeCommand(
    "vscode.executeSignatureHelpProvider",
    document.uri,
    newPos
  );
  if (res.signatures.length === 0) {
    return {
      signature: undefined,
      indent: undefined,
    };
  }
  return {
    signature: res.signatures[0],
    indent: match[1] == undefined ? "" : match[1],
  };
}
