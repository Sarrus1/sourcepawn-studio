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
    super("Generate docstring", CompletionItemKind.Text);
    const snippet = new SnippetString();
    const max = getMaxLength(FunctionDesc);
    snippet.appendText(`${indent}/**\n ${indent}* `);
    snippet.appendPlaceholder("Description");
    this.appendTextSnippet(snippet, "", indent);
    for (const arg of FunctionDesc) {
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
    snippet.appendText(`\n${indent} */`);
    this.insertText = snippet;
    this.filterText = "/*";
    const start: Position = new Position(position.line, 0);
    const end: Position = new Position(position.line, 0);
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
    const { signature, indent } = await getFullParams(document, position);
    if (signature === undefined) {
      return undefined;
    }

    const functionDesc = signature.parameters.map((e) => e.label.toString());

    const DocCompletionItem = new SpDocCompletionItem(
      position,
      functionDesc,
      indent
    );
    return [DocCompletionItem];
  }
}

function getMaxLength(arr: string[]): number {
  let max: number = 0;
  for (const str of arr) {
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
  const lineNB = position.line + 1;
  const line = lines[lineNB];
  const newSyntaxRe = /^(\s)*(?:(?:stock|public|native|forward|static)\s+)*(?:(\w*)\s+)?(\w*)\s*\(/;
  let match = line.match(newSyntaxRe);
  if (!match) {
    match = line.match(
      /^(\s)*(?:(?:static|native|stock|public|forward)\s+)*(?:(\w+)\s*:)?\s*(\w*)\s*\(/
    );
    if (!match) {
      return {
        signature: undefined,
        indent: undefined,
      };
    }
  }
  const newPos = new Position(lineNB, match[0].length);
  const res: SignatureHelp = await commands.executeCommand(
    "vscode.executeSignatureHelpProvider",
    document.uri,
    newPos
  );
  if (res === undefined || res.signatures.length === 0) {
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
