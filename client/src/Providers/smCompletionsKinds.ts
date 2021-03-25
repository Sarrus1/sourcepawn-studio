import * as vscode from "vscode";

export interface Completion {
  name: string;
  kind: vscode.CompletionItemKind;
  description?: string;

  to_completion_item(file: string): vscode.CompletionItem;
  get_signature(): vscode.SignatureInformation;
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
  kind = vscode.CompletionItemKind.Function;

  constructor(
    name: string,
    detail: string,
    description: string,
    params: FunctionParam[]
  ) {
    this.description = description;
    this.name = name;
    this.params = params;
    this.detail = detail;
  }

  to_completion_item(file: string): vscode.CompletionItem {
    return {
      label: this.name,
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
}

export class DefineCompletion implements Completion {
  name: string;
  type: string;
  kind = vscode.CompletionItemKind.Variable;

  constructor(name: string) {
    this.name = name;
  }

  to_completion_item(file: string): vscode.CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
    };
  }

  get_signature(): vscode.SignatureInformation {
    return undefined;
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
    };
  }

  get_signature(): vscode.SignatureInformation {
    return undefined;
  }
}

export class EnumMemberCompletion implements Completion {
  name: string;
  enum: EnumCompletion;
  file: string;
  kind = vscode.CompletionItemKind.EnumMember;

  constructor(name: string, file: string, Enum: EnumCompletion) {
    this.name = name;
    this.file = file;
    this.enum = Enum;
  }

  to_completion_item(file: string): vscode.CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
    };
  }

  get_signature(): vscode.SignatureInformation {
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