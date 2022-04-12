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

import { FunctionItem } from "./spFunctionItem";
import { MethodItem } from "./spMethodItem";
import { FunctionParam } from "../../Parser/interfaces";
import { MethodMapItem } from "./spMethodmapItem";
import { EnumStructItem } from "./spEnumStructItem";

export interface SPItem {
  name: string;
  kind: CompletionItemKind;
  filePath?: string;
  type?: string;
  parent?: SPItem;
  description?: string;
  range?: Range;
  detail?: string;
  fullRange?: Range;
  references?: Location[];
  IsBuiltIn?: boolean;
  enumStructName?: string;
  params?: FunctionParam[];
  deprecated?: string;

  toCompletionItem(
    lastFunc?: MethodItem | FunctionItem | undefined,
    lastESOrMM?: MethodMapItem | EnumStructItem | undefined
  ): CompletionItem | undefined;
  toDefinitionItem(): LocationLink | undefined;
  toReferenceItem?(): Location[];
  toSignature(): SignatureInformation | undefined;
  toHover(): Hover | undefined;
  toDocumentSymbol?(): DocumentSymbol | undefined;
}

export class Include {
  uri: string;
  IsBuiltIn: boolean;

  constructor(uri: string, IsBuiltIn: boolean) {
    this.uri = uri;
    this.IsBuiltIn = IsBuiltIn;
  }
}
