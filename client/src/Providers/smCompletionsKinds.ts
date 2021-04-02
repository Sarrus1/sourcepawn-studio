import * as vscode from "vscode";

export interface Completion {
  name: string;
  kind: vscode.CompletionItemKind;
  description?: string;

  to_completion_item(file: string): vscode.CompletionItem;
  get_signature(): vscode.SignatureInformation;
  get_hover(): vscode.Hover;
  documentation_to_md(string): vscode.MarkdownString;
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
      documentation: this.documentation_to_md(this.description),
      parameters: this.params,
    };
  }

  get_hover(): vscode.Hover {
    let description: string = "";
    if ((description = this.description) == "") {
      return;
    }
    return new vscode.Hover(this.documentation_to_md(description));
  }

  documentation_to_md(description: string): vscode.MarkdownString {
		// // Remove line breaks in the description
    // description = description.replace(/([^@])/g, function (doc) {
    //   return doc.replace(/+/gm, " ");
    // });
		// Make the @params nicer
    description = description.replace(
      /\s*(@param|@return)\s+([A-z0-9_]+)\s+/gm,
      "\n\n_$1_ `$2` - "
    );
		// Make other @ nicer
		description = description.replace(
      /\s*(@[A-z])\s+/gm,
      "\n\n_$1_ - "
    );
		// Format other functions which are referenced in the description
		description = description.replace(/([A-z0-9_]+\([A-z0-9_ \:]*\))/gm, "`$1`");
    return new vscode.MarkdownString(description);
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
    if ((description = this.description) == "") {
      return;
    }
    return new vscode.Hover(this.documentation_to_md(description));
  }

  documentation_to_md(description: string): vscode.MarkdownString {
    description = description.replace(/([^@])/g, function (doc) {
      return doc.replace(/\n+/gm, " ");
    });
    description = description.replace(
      /\s*(@[A-z0-9_]+)\s+([A-z0-9_]+)\s+/gm,
      "\n\n_$1_ `$2` - "
    );
    return new vscode.MarkdownString(description);
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

  get_hover(): vscode.Hover {
    return;
  }

  documentation_to_md(description: string): vscode.MarkdownString {
    return;
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

  documentation_to_md(description: string): vscode.MarkdownString {
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
    };
  }

  get_signature(): vscode.SignatureInformation {
    return undefined;
  }

  get_hover(): vscode.Hover {
    return;
  }

  documentation_to_md(description: string): vscode.MarkdownString {
    return;
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

  get_hover(): vscode.Hover {
    return;
  }

  documentation_to_md(description: string): vscode.MarkdownString {
    return;
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

  documentation_to_md(description: string): vscode.MarkdownString {
    return;
  }
}
