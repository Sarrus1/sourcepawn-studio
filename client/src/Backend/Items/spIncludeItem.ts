import {
  CompletionItemKind,
  Range,
  CompletionItem,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  LocationLink,
} from "vscode";
import { URI } from "vscode-uri";
import { basename } from "path";

import { SPItem } from "./spItems";

export class IncludeItem implements SPItem {
  name: string;
  kind = CompletionItemKind.File;
  file: string;
  defRange: Range;

  constructor(uri: string, defRange: Range) {
    this.name = basename(URI.file(uri).fsPath);
    this.file = uri;
    this.defRange = defRange;
  }

  toCompletionItem(): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: "",
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      originSelectionRange: this.defRange,
      targetRange: new Range(0, 0, 0, 0),
      targetUri: URI.parse(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return;
  }

  toHover(): Hover {
    return new Hover(URI.parse(this.file).fsPath);
  }

  toDocumentSymbol(): DocumentSymbol {
    return;
  }
}
