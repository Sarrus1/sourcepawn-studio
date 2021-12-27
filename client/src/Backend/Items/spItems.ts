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
  file?: string;
  type?: string;
  parent?: string;
  description?: string;
  range?: Range;
  detail?: string;
  fullRange?: Range;
  calls?: Location[];
  IsBuiltIn?: boolean;
  enumStructName?: string;
  commitCharacters?: string[];

  toCompletionItem(lastFuncName?: string): CompletionItem;
  toDefinitionItem(): LocationLink;
  toSignature(): SignatureInformation;
  toHover(): Hover;
  toDocumentSymbol?(): DocumentSymbol;
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
