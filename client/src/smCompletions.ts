import * as vscode from "vscode";
import * as glob from "glob";
import * as path from "path";
import { URI } from "vscode-uri";
import * as fs from "fs";
import * as parser from "./smParser";
import * as smDefinitions from "./smDefinitions";

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

export class FileCompletions {
  completions: Map<string, Completion>;
  includes: Include[];
  uri: string;

  constructor(uri: string) {
    this.completions = new Map();
    this.includes = [];
    this.uri = uri;
  }

  add(id: string, completion: Completion) {
    this.completions.set(id, completion);
  }

  get(id: string): Completion {
    return this.completions.get(id);
  }

  get_completions(repo: CompletionRepository): Completion[] {
    let completions = [];
    for (let completion of this.completions.values()) {
      completions.push(completion);
    }
    return completions;
  }

  to_completion_resolve(item: vscode.CompletionItem): vscode.CompletionItem {
    item.label = item.label;
    item.documentation = item.documentation;
    return item;
  }

  add_include(include: string, IsBuiltIn: boolean) {
    this.includes.push(new Include(include, IsBuiltIn));
  }

  resolve_import(
    file: string,
    relative: boolean = false,
    IsBuiltIn: boolean = false
  ) {
    let uri = file + ".inc";
    let base_file = URI.parse(this.uri).fsPath;
    let base_directory = path.dirname(base_file);
    let inc_file = "";
    // If the include is not relative, check if the file exists in the include folder
    // this is more beginner friendly.
    if (!relative) {
      // First, check the include folder.
      inc_file = path.join(base_directory, "include/", uri);
      if (fs.existsSync(inc_file)) {
        uri = URI.file(inc_file).toString();
        this.add_include(uri, IsBuiltIn);
        return;
      }
      // Check the optional include folders
      let includes_dirs: string[] = vscode.workspace
        .getConfiguration("sourcepawnLanguageServer")
        .get("optionalIncludeDirsPaths");
      for (let includes_dir of includes_dirs) {
        inc_file = path.join(includes_dir, uri);
        if (fs.existsSync(inc_file)) {
          uri = URI.file(inc_file).toString();
          this.add_include(uri, IsBuiltIn);
          return;
        }
      }
      // Otherwise consider this a builtin
      uri = "file://__sourcemod_builtin/" + uri;
      this.add_include(uri, IsBuiltIn);
    } else {
      // First check if it's a .inc relative to the script file.
      inc_file = path.resolve(base_directory, uri);
      if (fs.existsSync(inc_file)) {
        uri = URI.file(inc_file).toString();
        this.add_include(uri, IsBuiltIn);
      } 
      // Otherwise consider it's a .sp relative to script file.
      else {
        uri = URI.file(path.resolve(file + ".sp")).toString();
        this.add_include(uri, IsBuiltIn);
      }
    }
  }
}

export class CompletionRepository
  implements vscode.CompletionItemProvider, vscode.Disposable {
  public completions: Map<string, FileCompletions>;
  documents: Set<vscode.Uri>;
  private globalState: vscode.Memento;

  constructor(globalState?: vscode.Memento) {
    this.completions = new Map();
    this.documents = new Set();
    this.globalState = globalState;
  }

  public provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): vscode.CompletionList {
    let completions: vscode.CompletionList = this.get_completions(
      document,
      position
    );
    return completions;
  }

  public dispose() {}

  get_completions(
    document: vscode.TextDocument,
    position: vscode.Position
  ): vscode.CompletionList {
    let is_method = false;
    if (document) {
      let line = document.getText().split("\n")[position.line].trim();
      for (let i = line.length - 2; i >= 0; i--) {
        if (line[i].match(/[a-zA-Z0-9_]/)) {
          continue;
        }

        if (line[i] === ".") {
          is_method = true;
          break;
        }
        break;
      }
    }
    let all_completions: Completion[] = this.get_all_completions(
      document.uri.toString()
    );
    let all_completions_list: vscode.CompletionList = new vscode.CompletionList();
    if (all_completions != []) {
      all_completions_list.items = all_completions.map((completion) => {
        if (completion) {
          if (completion.to_completion_item) {
            return completion.to_completion_item(document.uri.fsPath);
          }
        }
      });
    }
    //return all_completions_list;
    if (is_method) {
      all_completions_list.items.filter(
        (completion) => completion.kind === vscode.CompletionItemKind.Method
      );
      return all_completions_list;
    } else {
      all_completions_list.items.filter(
        (completion) => completion.kind !== vscode.CompletionItemKind.Method
      );
      return all_completions_list;
    }
  }

  get_all_completions(file: string): Completion[] {
    let completion = this.completions.get(file);
    let includes = new Set<string>();
    if (completion) {
      this.get_included_files(completion, includes);
    }
    includes.add(file);
    return [...includes]
      .map((file) => {
        return this.get_file_completions(file);
      })
      .reduce(
        (completion, file_completions) => completion.concat(file_completions),
        []
      );
  }

  get_file_completions(file: string): Completion[] {
    let file_completions: FileCompletions = this.completions.get(file);
    let completion_list: Completion[] = [];
    if (file_completions) {
      return file_completions.get_completions(this);
    }
    return completion_list;
  }

  get_included_files(completions: FileCompletions, files: Set<string>) {
    for (let include of completions.includes) {
      if (!files.has(include.uri)) {
        files.add(include.uri);
        let include_completions = this.completions.get(include.uri);
        if (include_completions) {
          this.get_included_files(include_completions, files);
        }
      }
    }
  }

  provideSignatureHelp(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): vscode.SignatureHelp {
    //let document = this.documents.get(URI.parse(position.textDocument.uri));
    if (document) {
      let { method, parameter_count } = (() => {
        let line = document.getText().split("\n")[position.line];

        if (line[position.character - 1] === ")") {
          // We've finished this call
          return { method: undefined, parameter_count: 0 };
        }

        let method = "";
        let end_parameters = false;
        let parameter_count = 0;

        for (let i = position.character; i >= 0; i--) {
          if (end_parameters) {
            if (line[i].match(/[A-Za-z0-9_]/)) {
              method = line[i] + method;
            } else {
              break;
            }
          } else {
            if (line[i] === "(") {
              end_parameters = true;
            } else if (line[i] === ",") {
              parameter_count++;
            }
          }
        }

        return { method, parameter_count };
      })();

      let completions = this.get_all_completions(
        document.uri.toString()
      ).filter((completion) => {
        return completion.name === method;
      });

      if (completions.length > 0) {
        return {
          signatures: [completions[0].get_signature()],
          activeParameter: parameter_count,
          activeSignature: 0,
        };
      }
    }

    return {
      signatures: [],
      activeSignature: 0,
      activeParameter: 0,
    };
  }
}
