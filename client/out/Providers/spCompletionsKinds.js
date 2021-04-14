"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Include = exports.EnumMemberCompletion = exports.EnumCompletion = exports.VariableCompletion = exports.DefineCompletion = exports.MethodCompletion = exports.FunctionCompletion = void 0;
const vscode = require("vscode");
const spUtils_1 = require("../spUtils");
const path_1 = require("path");
class FunctionCompletion {
    constructor(name, detail, description, params, file, IsBuiltIn) {
        this.kind = vscode.CompletionItemKind.Function;
        this.description = description;
        this.name = name;
        this.params = params;
        this.detail = detail;
        this.file = file;
        this.IsBuiltIn = IsBuiltIn;
    }
    to_completion_item(file) {
        return {
            label: this.name,
            kind: this.kind,
            detail: path_1.basename(this.file),
        };
    }
    get_signature() {
        return {
            label: this.detail,
            documentation: spUtils_1.description_to_md(this.description),
            parameters: this.params,
        };
    }
    get_hover() {
        let filename = path_1.basename(this.file, ".inc");
        if (this.description == "") {
            return new vscode.Hover({ language: "sourcepawn", value: this.detail });
        }
        if (this.IsBuiltIn) {
            return new vscode.Hover([{ language: "sourcepawn", value: this.detail }, `[Online Documentation](https://sourcemod.dev/#/${filename}/function.${this.name})`, spUtils_1.description_to_md(this.description)]);
        }
        return new vscode.Hover([{ language: "sourcepawn", value: this.detail }, spUtils_1.description_to_md(this.description)]);
    }
}
exports.FunctionCompletion = FunctionCompletion;
class MethodCompletion {
    constructor(method_map, name, detail, description, params) {
        this.kind = vscode.CompletionItemKind.Method;
        this.method_map = method_map;
        this.name = name;
        this.detail = detail;
        this.description = description;
        this.params = params;
    }
    to_completion_item(file) {
        return {
            label: `${this.method_map}.${this.name}`,
            insertText: this.name,
            filterText: this.name,
            kind: this.kind,
            detail: this.description,
        };
    }
    get_signature() {
        return {
            label: this.detail,
            documentation: this.description,
            parameters: this.params,
        };
    }
    get_hover() {
        let description = "";
        if (!this.description) {
            return;
        }
        return new vscode.Hover([{ language: "sourcepawn", value: this.detail }, spUtils_1.description_to_md(this.description)]);
    }
}
exports.MethodCompletion = MethodCompletion;
class DefineCompletion {
    constructor(name, value, file) {
        this.kind = vscode.CompletionItemKind.Variable;
        this.name = name;
        this.value = value;
        this.file = path_1.basename(file);
    }
    to_completion_item(file) {
        return {
            label: this.name,
            kind: this.kind,
            detail: this.file
        };
    }
    get_signature() {
        return;
    }
    get_hover() {
        return new vscode.Hover({ language: "sourcepawn", value: `#define ${this.name} ${this.value}` });
    }
}
exports.DefineCompletion = DefineCompletion;
class VariableCompletion {
    constructor(name, file) {
        this.kind = vscode.CompletionItemKind.Variable;
        this.name = name;
        this.file = file;
    }
    to_completion_item(file) {
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
    get_signature() {
        return undefined;
    }
    get_hover() {
        return;
    }
}
exports.VariableCompletion = VariableCompletion;
class EnumCompletion {
    constructor(name, file) {
        this.kind = vscode.CompletionItemKind.Enum;
        this.name = name;
        this.file = file;
    }
    to_completion_item(file) {
        return {
            label: this.name,
            kind: this.kind,
            detail: path_1.basename(this.file)
        };
    }
    get_signature() {
        return undefined;
    }
    get_hover() {
        return;
    }
}
exports.EnumCompletion = EnumCompletion;
class EnumMemberCompletion {
    constructor(name, file, Enum) {
        this.kind = vscode.CompletionItemKind.EnumMember;
        this.name = name;
        this.file = file;
        this.enum = Enum;
    }
    to_completion_item(file) {
        return {
            label: this.name,
            kind: this.kind,
            detail: this.enum.name
        };
    }
    get_signature() {
        return undefined;
    }
    get_hover() {
        return;
    }
}
exports.EnumMemberCompletion = EnumMemberCompletion;
class Include {
    constructor(uri, IsBuiltIn) {
        this.uri = uri;
        this.IsBuiltIn = IsBuiltIn;
    }
    get_hover() {
        return;
    }
}
exports.Include = Include;
//# sourceMappingURL=spCompletionsKinds.js.map