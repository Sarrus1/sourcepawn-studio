import {
  CompletionItemKind,
  Range,
  CompletionItem,
  Location,
  SignatureInformation,
  Hover,
  DocumentSymbol,
  SymbolKind,
  LocationLink,
  workspace as Workspace,
} from "vscode";
import { descriptionToMD } from "../spUtils";
import { globalIdentifier } from "./spGlobalIdentifier";
import { basename } from "path";
import { URI } from "vscode-uri";

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

  toCompletionItem(file: string, lastFuncName?: string): CompletionItem;
  toDefinitionItem(): LocationLink;
  toSignature(): SignatureInformation;
  toHover(): Hover;
  toDocumentSymbol?(): DocumentSymbol;
}

export type FunctionParam = {
  label: string;
  documentation: string;
};

export class FunctionItem implements SPItem {
  name: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  file: string;
  range: Range;
  fullRange: Range;
  IsBuiltIn: boolean;
  kind = CompletionItemKind.Function;
  type: string;

  constructor(
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[],
    file: string,
    IsBuiltIn: boolean,
    range: Range,
    type: string,
    fullRange: Range
  ) {
    this.description = description;
    this.name = name;
    this.params = params;
    this.detail = detail;
    this.file = file;
    this.IsBuiltIn = IsBuiltIn;
    this.range = range;
    this.type = type;
    this.fullRange = fullRange;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return {
      label: this.detail,
      documentation: descriptionToMD(this.description),
      parameters: this.params,
    };
  }

  toHover(): Hover {
    let filename: string = basename(this.file, ".inc");
    if (this.description == "") {
      return new Hover({ language: "sourcepawn", value: this.detail });
    }
    if (this.IsBuiltIn) {
      return new Hover([
        { language: "sourcepawn", value: this.detail },
        `[Online Documentation](https://sourcemod.dev/#/${filename}/function.${this.name})`,
        descriptionToMD(this.description),
      ]);
    }
    return new Hover([
      { language: "sourcepawn", value: this.detail },
      descriptionToMD(this.description),
    ]);
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Function,
      this.fullRange,
      this.range
    );
  }
}

export class MacroItem extends FunctionItem {
  kind = CompletionItemKind.Interface;
}

export class MethodMapItem implements SPItem {
  name: string;
  parent: string;
  description: string;
  detail: string;
  kind = CompletionItemKind.Class;
  type: string;
  range: Range;
  IsBuiltIn: boolean;
  file: string;
  fullRange: Range;

  constructor(
    name: string,
    parent: string,
    detail: string,
    description: string,
    file: string,
    range: Range,
    IsBuiltIn: boolean = false
  ) {
    this.name = name;
    this.parent = parent;
    this.detail = detail;
    this.description = description;
    this.IsBuiltIn = IsBuiltIn;
    this.file = file;
    this.range = range;
    this.type = name;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file, ".inc"),
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    if (!this.description) {
      return new Hover([{ language: "sourcepawn", value: this.detail }]);
    }
    let filename: string = basename(this.file, ".inc");
    if (this.IsBuiltIn) {
      return new Hover([
        { language: "sourcepawn", value: this.detail },
        `[Online Documentation](https://sourcemod.dev/#/${filename}/methodmap.${this.name})`,
        descriptionToMD(this.description),
      ]);
    }
    return new Hover([
      { language: "sourcepawn", value: this.detail },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Class,
      this.fullRange,
      this.range
    );
  }
}

export class MethodItem implements SPItem {
  name: string;
  parent: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  kind: CompletionItemKind;
  fullRange: Range;
  type: string;
  range: Range;
  IsBuiltIn: boolean;
  file: string;

  constructor(
    parent: string,
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[],
    type: string,
    file: string,
    range: Range,
    IsBuiltIn: boolean = false,
    fullRange: Range = undefined
  ) {
    this.parent = parent;
    this.name = name;
    this.kind =
      this.name == this.parent
        ? CompletionItemKind.Constructor
        : CompletionItemKind.Method;
    this.detail = detail;
    this.description = description;
    this.params = params;
    this.type = type;
    this.IsBuiltIn = IsBuiltIn;
    this.file = file;
    this.range = range;
    this.fullRange = fullRange;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.parent,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return {
      label: this.detail,
      documentation: descriptionToMD(this.description),
      parameters: this.params,
    };
  }

  toHover(): Hover {
    if (!this.description) {
      return new Hover([{ language: "sourcepawn", value: this.detail }]);
    }
    let filename: string = basename(this.file, ".inc");
    if (this.IsBuiltIn) {
      return new Hover([
        { language: "sourcepawn", value: this.detail },
        `[Online Documentation](https://sourcemod.dev/#/${filename}/methodmap.${this.parent}/function.${this.name})`,
        descriptionToMD(this.description),
      ]);
    }
    return new Hover([
      { language: "sourcepawn", value: this.detail },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Method,
      this.fullRange,
      this.range
    );
  }
}

export class DefineItem implements SPItem {
  name: string;
  value: string;
  file: string;
  kind = CompletionItemKind.Constant;
  IsBuiltIn: boolean;
  range: Range;
  calls: Location[];
  fullRange: Range;

  constructor(
    name: string,
    value: string,
    file: string,
    range: Range,
    IsBuiltIn: boolean,
    fullRange: Range
  ) {
    this.name = name;
    this.value = value;
    this.file = file;
    this.range = range;
    this.calls = [];
    this.IsBuiltIn = IsBuiltIn;
    this.fullRange = fullRange;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.file,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return;
  }

  toHover(): Hover {
    return new Hover({
      language: "sourcepawn",
      value: `#define ${this.name} ${this.value}`,
    });
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      `#define ${this.name} ${this.value}`,
      SymbolKind.Constant,
      this.fullRange,
      this.range
    );
  }
}

export class VariableItem implements SPItem {
  name: string;
  file: string;
  kind = CompletionItemKind.Variable;
  parent: string;
  range: Range;
  type: string;
  enumStructName: string;

  constructor(
    name: string,
    file: string,
    parent: string,
    range: Range,
    type: string,
    enumStruct: string
  ) {
    this.name = name;
    this.file = file;
    this.parent = parent;
    this.range = range;
    this.type = type;
    this.enumStructName = enumStruct;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    if (lastFuncName !== undefined) {
      if (this.parent === lastFuncName) {
        return {
          label: this.name,
          kind: this.kind,
        };
      } else if (this.parent === globalIdentifier) {
        return {
          label: this.name,
          kind: this.kind,
        };
      }
      return undefined;
    } else {
      return {
        label: this.name,
        kind: this.kind,
      };
    }
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    return;
  }

  toDocumentSymbol(): DocumentSymbol {
    return new DocumentSymbol(
      this.name,
      this.type,
      SymbolKind.Variable,
      this.range,
      this.range
    );
  }
}

export class EnumItem implements SPItem {
  name: string;
  file: string;
  kind = CompletionItemKind.Enum;
  description: string;
  range: Range;
  fullRange: Range;

  constructor(name: string, file: string, description: string, range: Range) {
    this.name = name;
    this.file = file;
    this.description = description;
    this.range = range;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file),
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.name },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Enum,
      this.fullRange,
      this.range
    );
  }
}

export class EnumMemberItem implements SPItem {
  name: string;
  parent: string;
  file: string;
  description: string;
  kind = CompletionItemKind.EnumMember;
  range: Range;
  calls: Location[];
  IsBuiltIn: boolean;

  constructor(
    name: string,
    file: string,
    description: string,
    Enum: EnumItem,
    range: Range,
    IsBuiltItn: boolean
  ) {
    this.name = name;
    this.file = file;
    this.description = description;
    this.range = range;
    this.calls = [];
    this.IsBuiltIn = IsBuiltItn;
    this.parent = Enum.name;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.parent === "" ? basename(this.file) : this.parent,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    let enumName = this.parent;
    if (enumName == "") {
      return new Hover([
        { language: "sourcepawn", value: this.name },
        descriptionToMD(this.description),
      ]);
    } else {
      return new Hover([
        { language: "sourcepawn", value: `${this.parent} ${this.name};` },
        descriptionToMD(this.description),
      ]);
    }
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.name === "") {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Enum,
      this.range,
      this.range
    );
  }
}

export class EnumStructItem implements SPItem {
  name: string;
  file: string;
  description: string;
  kind = CompletionItemKind.Struct;
  range: Range;
  fullRange: Range;

  constructor(name: string, file: string, description: string, range: Range) {
    this.name = name;
    this.file = file;
    this.description = description;
    this.range = range;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file),
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.name },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Struct,
      this.fullRange,
      this.range
    );
  }
}

export class EnumStructMemberItem implements SPItem {
  name: string;
  enumStruct: EnumStructItem;
  file: string;
  description: string;
  type: string;
  kind = CompletionItemKind.Property;
  parent: string;
  range: Range;

  constructor(
    name: string,
    file: string,
    description: string,
    EnumStruct: EnumStructItem,
    range: Range,
    type: string
  ) {
    this.name = name;
    this.file = file;
    this.description = description;
    this.enumStruct = EnumStruct;
    this.range = range;
    this.parent = EnumStruct.name;
    this.type = type;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.enumStruct.name,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    let enumName = this.enumStruct.name;
    if (enumName == "") {
      return new Hover([
        { language: "sourcepawn", value: this.name },
        descriptionToMD(this.description),
      ]);
    } else {
      return new Hover([
        {
          language: "sourcepawn",
          value: this.enumStruct.name + " " + this.name,
        },
        descriptionToMD(this.description),
      ]);
    }
  }
}

export class PropertyItem implements SPItem {
  parent: string;
  name: string;
  file: string;
  description: string;
  type: string;
  detail: string;
  kind = CompletionItemKind.Property;
  range: Range;
  fullRange: Range;
  constructor(
    parent: string,
    name: string,
    file: string,
    detail: string,
    description: string,
    range: Range,
    type: string
  ) {
    this.parent = parent;
    this.name = name;
    this.file = file;
    this.description = description;
    this.range = range;
    this.type = type;
    this.detail = detail;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.parent,
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.name },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.Property,
      this.fullRange,
      this.range
    );
  }
}

export class ConstantItem implements SPItem {
  name: string;
  kind = CompletionItemKind.Constant;
  calls: Location[];
  constructor(name: string) {
    this.name = name;
    this.calls = [];
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: "",
    };
  }

  toDefinitionItem(): LocationLink {
    return undefined;
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    return undefined;
  }

  toDocumentSymbol(): DocumentSymbol {
    return undefined;
  }
}

export class IncludeItem implements SPItem {
  name: string;
  kind = CompletionItemKind.File;
  file: string;
  defRange: Range;

  constructor(uri: string, defRange: Range) {
    let workspaceFolder = Workspace.getWorkspaceFolder(URI.parse(uri));
    this.name = basename(URI.file(uri).fsPath);
    let smHome: string =
      Workspace.getConfiguration("sourcepawn", workspaceFolder).get(
        "SourcemodHome"
      ) || "";
    uri = this.file = uri.replace(
      "file://__sourcemod_builtin",
      URI.file(smHome).toString()
    );
    this.defRange = defRange;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
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
    return undefined;
  }

  toHover(): Hover {
    return new Hover(URI.parse(this.file).fsPath);
  }

  toDocumentSymbol(): DocumentSymbol {
    return undefined;
  }
}

export class KeywordItem implements SPItem {
  name: string;
  kind = CompletionItemKind.Keyword;
  constructor(name: string) {
    this.name = name;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: "",
    };
  }

  toDefinitionItem(): LocationLink {
    return undefined;
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    return undefined;
  }

  toDocumentSymbol(): DocumentSymbol {
    return undefined;
  }
}

export class TypeDefItem implements SPItem {
  name: string;
  details: string;
  type: string;
  file: string;
  description: string;
  kind = CompletionItemKind.TypeParameter;
  range: Range;
  fullRange: Range;

  constructor(
    name: string,
    details: string,
    file: string,
    description: string,
    type: string,
    range: Range,
    fullRange: Range
  ) {
    this.name = name;
    this.details = details;
    this.file = file;
    this.type = type;
    this.description = description;
    this.range = range;
    this.fullRange = fullRange;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file),
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.details },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.TypeParameter,
      this.fullRange,
      this.range
    );
  }
}

export class TypeSetItem implements SPItem {
  name: string;
  details: string;
  file: string;
  description: string;
  kind = CompletionItemKind.TypeParameter;
  range: Range;
  fullRange: Range;

  constructor(
    name: string,
    details: string,
    file: string,
    description: string,
    range: Range,
    fullRange: Range
  ) {
    this.name = name;
    this.details = details;
    this.file = file;
    this.description = description;
    this.range = range;
    this.fullRange = fullRange;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file),
    };
  }

  toDefinitionItem(): LocationLink {
    return {
      targetRange: this.range,
      targetUri: URI.file(this.file),
    };
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: `typedef ${this.name}` },
      descriptionToMD(this.description),
    ]);
  }

  toDocumentSymbol(): DocumentSymbol {
    if (this.fullRange === undefined) {
      return undefined;
    }
    return new DocumentSymbol(
      this.name,
      this.description,
      SymbolKind.TypeParameter,
      this.fullRange,
      this.range
    );
  }
}

export class CommentItem implements SPItem {
  name: string;
  file: string;
  kind = CompletionItemKind.User;
  range: Range;

  constructor(file: string, range: Range) {
    this.file = file;
    this.range = range;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return undefined;
  }

  toDefinitionItem(): LocationLink {
    return undefined;
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    return undefined;
  }

  toDocumentSymbol(): DocumentSymbol {
    return undefined;
  }
}

export class Include {
  uri: string;
  IsBuiltIn: boolean;

  constructor(uri: string, IsBuiltIn: boolean) {
    this.uri = uri;
    this.IsBuiltIn = IsBuiltIn;
  }
}
