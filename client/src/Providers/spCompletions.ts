import {
  CompletionItemKind,
  Range,
  CompletionItem,
  Location,
  SignatureInformation,
  Hover,
} from "vscode";
import { description_to_md } from "../spUtils";
import { basename } from "path";
import { URI } from "vscode-uri";

export interface Completion {
  name: string;
  kind: CompletionItemKind;
  description?: string;
  range?: Range;
  scope?: string;

  to_completion_item(file: string, lastFuncName: string): CompletionItem;
  toDefinitionItem(): Location;
  get_signature(): SignatureInformation;
  get_hover(): Hover;
}

export type FunctionParam = {
  label: string;
  documentation: string;
};

export class FunctionCompletion implements Completion {
  name: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  file: string;
  range: Range;
  IsBuiltIn: boolean;
  kind = CompletionItemKind.Function;

  constructor(
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[],
    file: string,
    IsBuiltIn: boolean,
    range: Range
  ) {
    this.description = description;
    this.name = name;
    this.params = params;
    this.detail = detail;
    this.file = file;
    this.IsBuiltIn = IsBuiltIn;
    this.range = range;
  }

  to_completion_item(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file),
    };
  }

  get_signature(): SignatureInformation {
    return {
      label: this.detail,
      documentation: description_to_md(this.description),
      parameters: this.params,
    };
  }

  get_hover(): Hover {
    let filename: string = basename(this.file, ".inc");
    if (this.description == "") {
      return new Hover({ language: "sourcepawn", value: this.detail });
    }
    if (this.IsBuiltIn) {
      return new Hover([
        { language: "sourcepawn", value: this.detail },
        `[Online Documentation](https://sourcemod.dev/#/${filename}/function.${this.name})`,
        description_to_md(this.description),
      ]);
    }
    return new Hover([
      { language: "sourcepawn", value: this.detail },
      description_to_md(this.description),
    ]);
  }

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
  }
}

export class MethodCompletion implements Completion {
  name: string;
  method_map: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  kind = CompletionItemKind.Method;

  constructor(
    method_map: string,
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[]
  ) {
    this.method_map = method_map;
    this.name = name;
    this.detail = detail;
    this.description = description;
    this.params = params;
  }

  to_completion_item(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.method_map,
    };
  }

  toDefinitionItem(): Location {
    return undefined;
  }

  get_signature(): SignatureInformation {
    return {
      label: this.detail,
      documentation: description_to_md(this.description),
      parameters: this.params,
    };
  }

  get_hover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.detail },
      description_to_md(this.description),
    ]);
  }
}

export class DefineCompletion implements Completion {
  name: string;
  value: string;
  file: string;
  kind = CompletionItemKind.Variable;
  range: Range;

  constructor(name: string, value: string, file: string, range: Range) {
    this.name = name;
    this.value = value;
    this.file = basename(file);
    this.range = range;
  }

  to_completion_item(
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

  get_signature(): SignatureInformation {
    return;
  }

  get_hover(): Hover {
    return new Hover({
      language: "sourcepawn",
      value: `#define ${this.name} ${this.value}`,
    });
  }
}

export class VariableCompletion implements Completion {
  name: string;
  file: string;
  kind = CompletionItemKind.Variable;
  scope: string;
  range: Range;

  constructor(name: string, file: string, scope: string, range: Range) {
    this.name = name;
    this.file = file;
    this.scope = scope;
    this.range = range;
  }

  to_completion_item(
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

  get_signature(): SignatureInformation {
    return undefined;
  }

  get_hover(): Hover {
    return;
  }
}

export class EnumCompletion implements Completion {
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

  to_completion_item(
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

  get_signature(): SignatureInformation {
    return undefined;
  }

  get_hover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.name },
      description_to_md(this.description),
    ]);
  }
}

export class EnumMemberCompletion implements Completion {
  name: string;
  enum: EnumCompletion;
  file: string;
  description: string;
  kind = CompletionItemKind.EnumMember;
  range: Range;

  constructor(
    name: string,
    file: string,
    description: string,
    Enum: EnumCompletion,
    range: Range
  ) {
    this.name = name;
    this.file = file;
    this.description = description;
    this.enum = Enum;
    this.range = range;
  }

  to_completion_item(
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

  get_signature(): SignatureInformation {
    return undefined;
  }

  get_hover(): Hover {
    let enumName = this.enum.name;
    if (enumName == "") {
      return new Hover([
        { language: "sourcepawn", value: this.name },
        description_to_md(this.description),
      ]);
    } else {
      return new Hover([
        { language: "sourcepawn", value: this.enum.name + " " + this.name },
        description_to_md(this.description),
      ]);
    }
  }
}

export class EnumStructCompletion implements Completion {
  name: string;
  file: string;
  description: string;
  kind = CompletionItemKind.Struct;
  range: Range;

  constructor(name: string, file: string, description: string, range: Range) {
    this.name = name;
    this.file = file;
    this.description = description;
  }

  to_completion_item(
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

  get_signature(): SignatureInformation {
    return undefined;
  }

  get_hover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.name },
      description_to_md(this.description),
    ]);
  }
}

export class EnumStructMemberCompletion implements Completion {
  name: string;
  enumStruct: EnumStructCompletion;
  file: string;
  description: string;
  kind = CompletionItemKind.Property;
  range: Range;

  constructor(
    name: string,
    file: string,
    description: string,
    EnumStruct: EnumStructCompletion,
    range: Range
  ) {
    this.name = name;
    this.file = file;
    this.description = description;
    this.enumStruct = EnumStruct;
    this.range = range;
  }

  to_completion_item(
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

  get_signature(): SignatureInformation {
    return undefined;
  }

  get_hover(): Hover {
    let enumName = this.enumStruct.name;
    if (enumName == "") {
      return new Hover([
        { language: "sourcepawn", value: this.name },
        description_to_md(this.description),
      ]);
    } else {
      return new Hover([
        {
          language: "sourcepawn",
          value: this.enumStruct.name + " " + this.name,
        },
        description_to_md(this.description),
      ]);
    }
  }
}

export class PropertyCompletion implements Completion {
  method_map: string;
  name: string;
  file: string;
  description: string;
  kind = CompletionItemKind.Property;
  range: Range;

  constructor(
    method_map: string,
    name: string,
    file: string,
    description: string,
    range: Range
  ) {
    this.method_map = method_map;
    this.name = name;
    this.file = file;
    this.description = description;
    this.range = range;
  }

  to_completion_item(
    file: string,
    lastFuncName: string = undefined
  ): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.method_map,
    };
  }

  toDefinitionItem(): Location {
    return new Location(URI.file(this.file), this.range);
  }

  get_signature(): SignatureInformation {
    return undefined;
  }

  get_hover(): Hover {
    if (!this.description) {
      return;
    }
    return new Hover([
      { language: "sourcepawn", value: this.name },
      description_to_md(this.description),
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
