"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.DefinitionRepository = exports.DefLocation = exports.DefinitionKind = void 0;
const vscode = require("vscode");
var DefinitionKind;
(function (DefinitionKind) {
    DefinitionKind[DefinitionKind["Variable"] = 0] = "Variable";
    DefinitionKind[DefinitionKind["Function"] = 1] = "Function";
    DefinitionKind[DefinitionKind["Define"] = 2] = "Define";
    DefinitionKind[DefinitionKind["Enum"] = 3] = "Enum";
    DefinitionKind[DefinitionKind["EnumMember"] = 4] = "EnumMember";
})(DefinitionKind = exports.DefinitionKind || (exports.DefinitionKind = {}));
class DefLocation extends vscode.Location {
    constructor(uri, range, type) {
        super(uri, range);
        this.type = type;
    }
}
exports.DefLocation = DefLocation;
class DefinitionRepository {
    constructor(globalState) {
        this.definitions = new Map();
        this.globalState = globalState;
    }
    provideDefinition(document, position, token) {
        let word = document.getText(document.getWordRangeAtPosition(position));
        let definition = this.definitions.get(word);
        if (typeof definition != "undefined" && this.isLocalFileVariable(document, definition)) {
            return new vscode.Location(definition.uri, definition.range);
        }
    }
    ;
    dispose() { }
    isLocalFileVariable(document, definition) {
        if (definition.type === DefinitionKind.Variable) {
            return document.uri.fsPath == definition.uri.fsPath;
        }
        return true;
    }
}
exports.DefinitionRepository = DefinitionRepository;
//# sourceMappingURL=spDefinitions.js.map