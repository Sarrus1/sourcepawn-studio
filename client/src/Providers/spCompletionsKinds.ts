import * as vscode from "vscode";
import { description_to_md } from "../spUtils";
import {basename} from "path";

export interface Completion {
  name: string;
  kind: vscode.CompletionItemKind;
  description?: string;

  to_completion_item(file: string): vscode.CompletionItem;
  get_signature(): vscode.SignatureInformation;
  get_hover(): vscode.Hover;
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
  IsBuiltIn: boolean;
  kind = vscode.CompletionItemKind.Function;

  constructor(
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[],
    file: string,
    IsBuiltIn: boolean
  ) {
    this.description = description;
    this.name = name;
    this.params = params;
    this.detail = detail;
    this.file = file;
    this.IsBuiltIn = IsBuiltIn;
  }

  to_completion_item(file: string): vscode.CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file),
    };
  }

  get_signature(): vscode.SignatureInformation {
    return {
      label: this.detail,
      documentation: description_to_md(this.description),
      parameters: this.params,
    };
  }

  get_hover(): vscode.Hover {
    let filename : string = basename(this.file, ".inc");
    if (this.description == "") {
      return new vscode.Hover({language:"sourcepawn", value:this.detail});
    }
    if(this.IsBuiltIn)
    {
      return new vscode.Hover([{language:"sourcepawn", value:this.detail}, `[Online Documentation](https://sourcemod.dev/#/${filename}/function.${this.name})`, description_to_md(this.description)]);
    }
    return new vscode.Hover([{language:"sourcepawn", value:this.detail}, description_to_md(this.description)]);
  }
}

export class MethodCompletion implements Completion {
  name: string;
  method_map: string;
  description: string;
  detail: string;
  params: FunctionParam[];
  kind = vscode.CompletionItemKind.Method;

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

  to_completion_item(file: string): vscode.CompletionItem {
    return {
      label: `${this.method_map}.${this.name}`,
      insertText: this.name,
      filterText: this.name,
      kind: this.kind,
      detail: this.description,
    };
  }

  get_signature(): vscode.SignatureInformation {
    return {
      label: this.detail,
      documentation: this.description,
      parameters: this.params,
    };
  }

  get_hover(): vscode.Hover {
    let description: string = "";
    if (!this.description) {
      return;
    }
    return new vscode.Hover([{language:"sourcepawn", value: this.detail}, description_to_md(this.description)]);
  }
}

export class DefineCompletion implements Completion {
  name: string;
  value: string;
  file: string;
  kind = vscode.CompletionItemKind.Variable;

  constructor(name: string, value: string, file: string) {
    this.name = name;
    this.value = value;
    this.file = basename(file);
  }

  to_completion_item(file: string): vscode.CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.file
    };
  }

  get_signature(): vscode.SignatureInformation {
    return;
  }

  get_hover(): vscode.Hover {
      return new vscode.Hover({language:"sourcepawn", value: `#define ${this.name} ${this.value}`});
  }
}

export class VariableCompletion implements Completion {
  name: string;
  file: string;
  kind = vscode.CompletionItemKind.Variable;

  constructor(name: string, file: string) {
    this.name = name;
    this.file = file;
  }

  to_completion_item(file: string): vscode.CompletionItem {
    // Only return variables local to the document being edited
    if (file === this.file) {
      return {
        label: this.name,
        kind: this.kind,
      };
    }
    return {
      label: "",
      kind: this.kind,
    };
  }

  get_signature(): vscode.SignatureInformation {
    return undefined;
  }

  get_hover(): vscode.Hover {
    return;
  }
}

export class EnumCompletion implements Completion {
  name: string;
  file: string;
  kind = vscode.CompletionItemKind.Enum;

  constructor(name: string, file: string) {
    this.name = name;
    this.file = file;
  }

  to_completion_item(file: string): vscode.CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: basename(this.file)
    };
  }

  get_signature(): vscode.SignatureInformation {
    return undefined;
  }

  get_hover(): vscode.Hover {
    return;
  }
}

export class EnumMemberCompletion implements Completion {
  name: string;
  enum: EnumCompletion;
  file: string;
  description: string;
  kind = vscode.CompletionItemKind.EnumMember;

  constructor(name: string, file: string, description: string, Enum: EnumCompletion) {
    this.name = name;
    this.file = file;
    this.description = description;
    this.enum = Enum;
  }

  to_completion_item(file: string): vscode.CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      detail: this.enum.name == "" ? basename(this.file) : this.enum.name
    };
  }

  get_signature(): vscode.SignatureInformation {
    return undefined;
  }

  get_hover(): vscode.Hover {
    let enumName = this.enum.name;
    if(enumName=="")
    {
      return new vscode.Hover([{language:"sourcepawn", value:this.name}, description_to_md(this.description)]);
    }
    else
    {
      return new vscode.Hover([{language:"sourcepawn", value:this.enum.name+" "+this.name}, description_to_md(this.description)]);
    }
    
  }
}

export class Include {
  uri: string;
  IsBuiltIn: boolean;

  constructor(uri: string, IsBuiltIn: boolean) {
    this.uri = uri;
    this.IsBuiltIn = IsBuiltIn;
  }

  get_hover(): vscode.Hover {
    return;
  }
}
