import {
  CompletionItemKind,
  Range,
  CompletionItem,
  Location,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  LocationLink,
  Position,
} from "vscode";

import { FunctionItem } from "./spFunctionItem";
import { MethodItem } from "./spMethodItem";
import { FunctionParam } from "../../Parser/interfaces";
import { MethodMapItem } from "./spMethodmapItem";
import { EnumStructItem } from "./spEnumStructItem";
import { VariableItem } from "./spVariableItem";

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
  params?: VariableItem[];
  deprecated?: string;

  toCompletionItem(
    lastFunc?: MethodItem | FunctionItem | undefined,
    lastESOrMM?: MethodMapItem | EnumStructItem | undefined,
    location?: Location,
    override?: boolean
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
  range: Range;

  constructor(uri: string, range: Range, IsBuiltIn: boolean) {
    this.uri = uri;
    this.range = range;
    this.IsBuiltIn = IsBuiltIn;
  }
}
