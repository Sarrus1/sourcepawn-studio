import {
  CompletionItemKind,
  Range,
  CompletionItem,
  Location,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  LocationLink,
} from "vscode";

export interface SPItem {
  name: string;
  kind: CompletionItemKind;
  filePath?: string;
  type?: string;
  parent?: string;
  description?: string;
  range?: Range;
  detail?: string;
  fullRange?: Range;
  calls?: Location[];
  IsBuiltIn?: boolean;
  enumStructName?: string;

  toCompletionItem(lastFuncName?: string): CompletionItem | undefined;
  toDefinitionItem(): LocationLink | undefined;
  toSignature(): SignatureInformation | undefined;
  toHover(): Hover | undefined;
  toDocumentSymbol?(): DocumentSymbol | undefined;
}

export type FunctionParam = {
  label: string;
  documentation: string;
};

export class Include {
  uri: string;
  IsBuiltIn: boolean;

  constructor(uri: string, IsBuiltIn: boolean) {
    this.uri = uri;
    this.IsBuiltIn = IsBuiltIn;
  }
}
