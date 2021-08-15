import {
  CompletionItemKind,
  Range,
  CompletionItem,
  Location,
  SignatureInformation,
  Hover,
} from "vscode";
import { descriptionToMD } from "../spUtils";
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
  scope?: string;
  calls?: Location[];
  IsBuiltIn?: boolean;
  enumStructName?: string;

  toCompletionItem(file: string, lastFuncName?: string): CompletionItem;
  toDefinitionItem(): Location;
  toSignature(): SignatureInformation;
  toHover(): Hover;
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
    type: string
  ) {
    this.description = description;
    this.name = name;
    this.params = params;
    this.detail = detail;
    this.file = file;
    this.IsBuiltIn = IsBuiltIn;
    this.range = range;
    this.type = type;
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

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
  }
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

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
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
}

export class MethodItem implements SPItem {
  name: string;
  parent: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  kind = CompletionItemKind.Method;
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
    IsBuiltIn: boolean = false
  ) {
    this.parent = parent;
    this.name = name;
    this.detail = detail;
    this.description = description;
    this.params = params;
    this.type = type;
    this.IsBuiltIn = IsBuiltIn;
    this.file = file;
    this.range = range;
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

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
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
}

export class DefineItem implements SPItem {
  name: string;
  value: string;
  file: string;
  kind = CompletionItemKind.Constant;
  IsBuiltIn: boolean;
  range: Range;
  calls: Location[];

  constructor(
    name: string,
    value: string,
    file: string,
    range: Range,
    IsBuiltIn: boolean
  ) {
    this.name = name;
    this.value = value;
    this.file = file;
    this.range = range;
    this.calls = [];
    this.IsBuiltIn = IsBuiltIn;
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

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
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
}

export class VariableItem implements SPItem {
  name: string;
  file: string;
  kind = CompletionItemKind.Variable;
  scope: string;
  range: Range;
  type: string;
  enumStructName: string;

  constructor(
    name: string,
    file: string,
    scope: string,
    range: Range,
    type: string,
    enumStruct: string
  ) {
    this.name = name;
    this.file = file;
    this.scope = scope;
    this.range = range;
    this.type = type;
    this.enumStructName = enumStruct;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    if (typeof lastFuncName !== "undefined") {
      if (this.scope === lastFuncName) {
        return {
          label: this.name,
          kind: this.kind,
        };
      } else if (this.scope === "$GLOBAL") {
        return {
          label: this.name,
          kind: this.kind,
        };
      }
      return {
        label: "",
        kind: this.kind,
      };
    } else {
      return {
        label: this.name,
        kind: this.kind,
      };
    }
  }

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    return;
  }
}

export class EnumItem implements SPItem {
  name: string;
  file: string;
  kind = CompletionItemKind.Enum;
  description: string;
  range: Range;

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

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
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
}

export class EnumMemberItem implements SPItem {
  name: string;
  enum: EnumItem;
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
    this.enum = Enum;
    this.range = range;
    this.calls = [];
    this.IsBuiltIn = IsBuiltItn;
  }

  toCompletionItem(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.enum.name == "" ? basename(this.file) : this.enum.name,
    };
  }

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
  }

  toSignature(): SignatureInformation {
    return undefined;
  }

  toHover(): Hover {
    let enumName = this.enum.name;
    if (enumName == "") {
      return new Hover([
        { language: "sourcepawn", value: this.name },
        descriptionToMD(this.description),
      ]);
    } else {
      return new Hover([
        { language: "sourcepawn", value: `${this.enum.name} ${this.name};` },
        descriptionToMD(this.description),
      ]);
    }
  }
}

export class EnumStructItem implements SPItem {
  name: string;
  file: string;
  description: string;
  kind = CompletionItemKind.Struct;
  range: Range;

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

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
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

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
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
  kind = CompletionItemKind.Property;
  range: Range;

  constructor(
    parent: string,
    name: string,
    file: string,
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

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
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
}

export class Include {
  uri: string;
  IsBuiltIn: boolean;

  constructor(uri: string, IsBuiltIn: boolean) {
    this.uri = uri;
    this.IsBuiltIn = IsBuiltIn;
  }

  get_hover(): Hover {
    return;
  }
}
