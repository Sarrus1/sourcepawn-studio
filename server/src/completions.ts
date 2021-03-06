import {
  CompletionItemKind,
  CompletionItem,
  TextDocumentPositionParams,
  SignatureHelp,
  SignatureInformation,
  TextDocuments,
  TextDocumentChangeEvent,
} from "vscode-languageserver/node";
import { parse_blob, parse_file } from "./parser";
import { TextDocument } from "vscode-languageserver-textdocument";

import * as glob from "glob";
import * as path from "path";
import { URI } from "vscode-uri";
import * as fs from "fs";

export interface Completion {
  name: string;
  kind: CompletionItemKind;
  description?: string;

  to_completion_item(file: string): CompletionItem;
  get_signature(): SignatureInformation;
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
  kind = CompletionItemKind.Function;

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

  to_completion_item(file: string): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
      data: this.description
    };
  }

  get_signature(): SignatureInformation {
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

  to_completion_item(file: string): CompletionItem {
    return {
      label: `${this.method_map}.${this.name}`,
      insertText: this.name,
      filterText: this.name,
      kind: this.kind,
      data: this.description,
    };
  }

  get_signature(): SignatureInformation {
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
  kind = CompletionItemKind.Variable;

  constructor(name: string) {
    this.name = name;
  }

  to_completion_item(file: string): CompletionItem {
    return {
      label: this.name,
      kind: this.kind,
    };
  }

  get_signature(): SignatureInformation {
    return undefined;
  }
}

export class VariableCompletion implements Completion {
  name: string;
  file: string;
  kind = CompletionItemKind.Variable;

  constructor(name: string, file: string) {
    this.name = name;
    this.file = file;
  }

  to_completion_item(file: string): CompletionItem {
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

  get_signature(): SignatureInformation {
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

  to_completion_resolve(item: CompletionItem): CompletionItem {
    item.label= item.label;
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
    // this is more beginner friendly
    if (!relative) {
      inc_file = path.join(base_directory, "include/", uri);
      if (fs.existsSync(inc_file)) {
        uri = URI.file(inc_file).toString();
        this.add_include(uri, IsBuiltIn);
      } else {
        uri = "file://__sourcemod_builtin/" + uri;
        this.add_include(uri, IsBuiltIn);
      }
    } else {
      inc_file = path.resolve(base_directory, uri);
      if (fs.existsSync(inc_file)) {
        uri = URI.file(inc_file).toString();
        this.add_include(uri, IsBuiltIn);
      } else {
        uri = URI.file(path.resolve(file + ".sp")).toString();
        this.add_include(uri, IsBuiltIn);
      }
    }
  }
}

export class CompletionRepository {
  completions: Map<string, FileCompletions>;
  documents: TextDocuments<TextDocument>;

  constructor(documents: TextDocuments<TextDocument>) {
    this.completions = new Map();
    this.documents = documents;
    documents.onDidOpen(this.handle_document_change.bind(this));
    documents.onDidChangeContent(this.handle_document_change.bind(this));
  }

  handle_document_change(event: TextDocumentChangeEvent<TextDocument>) {
    let completions = new FileCompletions(event.document.uri);
    parse_blob(event.document.getText(), completions, event.document.uri);
    this.read_unscanned_imports(completions);

    this.completions.set(event.document.uri, completions);
  }

  read_unscanned_imports(completions: FileCompletions) {
    for (let import_file of completions.includes) {
      let completion = this.completions.get(import_file.uri);
      if (!completion) {
        let file = URI.parse(import_file.uri).fsPath;
        let new_completions = new FileCompletions(import_file.uri);
        parse_file(file, new_completions, import_file.IsBuiltIn);

        this.read_unscanned_imports(new_completions);

        this.completions.set(import_file.uri, new_completions);
      }
    }
  }

  parse_sm_api(sourcemod_home: string) {
    glob(path.join(sourcemod_home, "**/*.inc"), (err, files) => {
      for (let file of files) {
        let completions = new FileCompletions(URI.file(file).toString());
        parse_file(file, completions, true);

        let uri =
          "file://__sourcemod_builtin/" + path.relative(sourcemod_home, file);
        this.completions.set(uri, completions);
      }
    });
  }

  get_completions(position: TextDocumentPositionParams): CompletionItem[] {
    let document = this.documents.get(position.textDocument.uri);
    let is_method = false;
    if (document) {
      let line = document.getText().split("\n")[position.position.line].trim();
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
    let all_completions = this.get_all_completions(
      position.textDocument.uri
    ).map((completion) =>
      completion.to_completion_item(position.textDocument.uri)
    );
    if (is_method) {
      return all_completions.filter(
        (completion) => completion.kind === CompletionItemKind.Method
      );
    } else {
      return all_completions.filter(
        (completion) => completion.kind !== CompletionItemKind.Method
      );
    }
  }

  get_all_completions(file: string): Completion[] {
    let completions = this.completions.get(file);

    let includes = new Set<string>();
    this.get_included_files(completions, includes);
    includes.add(file);
    return [...includes]
      .map((file) => {
        return this.get_file_completions(file);
      })
      .reduce(
        (completions, file_completions) => completions.concat(file_completions),
        []
      );
  }

  get_file_completions(file: string): Completion[] {
    let completions = this.completions.get(file);
    if (completions) {
      return completions.get_completions(this);
    }

    return [];
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

  get_signature(position: TextDocumentPositionParams): SignatureHelp {
    let document = this.documents.get(position.textDocument.uri);
    if (document) {
      let { method, parameter_count } = (() => {
        let line = document.getText().split("\n")[position.position.line];

        if (line[position.position.character - 1] === ")") {
          // We've finished this call
          return { method: undefined, parameter_count: 0 };
        }

        let method = "";
        let end_parameters = false;
        let parameter_count = 0;

        for (let i = position.position.character; i >= 0; i--) {
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
        position.textDocument.uri
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
