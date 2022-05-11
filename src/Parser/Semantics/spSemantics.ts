import { CompletionItemKind, Diagnostic, Range } from "vscode";
import { URI } from "vscode-uri";

import { ItemsRepository } from "../../Backend/spItemsRepository";
import { FileItem } from "../../Backend/spFilesRepository";
import { SPItem } from "../../Backend/Items/spItems";
import { handleToken } from "./handleReferencesInParser";
import { checkIfPluginInfo } from "../utils";
import { globalIdentifier } from "../../Misc/spConstants";
import { FunctionItem } from "../../Backend/Items/spFunctionItem";
import { MethodItem } from "../../Backend/Items/spMethodItem";
import { PropertyItem } from "../../Backend/Items/spPropertyItem";
import { MethodMapItem } from "../../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../../Backend/Items/spEnumStructItem";
import { parserDiagnostics } from "../../Providers/Linter/compilerDiagnostics";
import { VariableItem } from "../../Backend/Items/spVariableItem";
import { TypeDefItem } from "../../Backend/Items/spTypedefItem";
import { TypeSetItem } from "../../Backend/Items/spTypesetItem";
import { generateReferencesMap } from "./generateReferencesMap";

export class Semantics {
  fileItems: FileItem;
  lines: string[];
  lineNb: number;
  filePath: string;
  methodAndProperties: Map<string, MethodItem | PropertyItem | VariableItem>;
  funcsAndMethodsInFile: (FunctionItem | MethodItem | PropertyItem)[];
  typeDefAndSetInFile: (TypeDefItem | TypeSetItem)[];
  MmEsInFile: (MethodMapItem | EnumStructItem)[];
  referencesMap: Map<string, SPItem>;
  previousItems: SPItem[];
  offset: number;
  line: string;
  scope: string;
  outsideScope: string;
  allItems: SPItem[];
  diagnostics: Diagnostic[];
  lastFunc: FunctionItem | MethodItem | PropertyItem | undefined;
  lastMMorES: MethodMapItem | EnumStructItem | undefined;
  inTypeDef: boolean;
  range: Range;

  constructor(
    lines: string[],
    filePath: string,
    completions: FileItem,
    itemsRepository: ItemsRepository,
    offset: number,
    range?: Range
  ) {
    this.fileItems = completions;
    this.lineNb = 0;
    this.lines = lines;
    this.filePath = filePath;
    this.allItems = itemsRepository.getAllItems(URI.file(this.filePath));
    this.methodAndProperties = new Map();
    this.funcsAndMethodsInFile = [];
    this.MmEsInFile = [];
    this.referencesMap = new Map();
    this.typeDefAndSetInFile = [];
    this.offset = offset;
    this.range = range;
    generateReferencesMap.call(this);
  }
  /**
   * Perform a semantic analyzis on the tokens of a file in order to link each token
   * to its item in order to generate the references array of each item.
   * @returns void
   */
  analyze(): void {
    let line = this.lines[0];

    let lastFunc: FunctionItem | MethodItem | PropertyItem | undefined;
    let lastMMorES: MethodMapItem | EnumStructItem | undefined;

    this.fileItems.tokens.sort((a, b) => {
      if (a.range.start.line === b.range.start.line) {
        return a.range.start.character - b.range.start.character;
      }
      return a.range.start.line - b.range.start.line;
    });

    const newDiagnostics = Array.from(
      parserDiagnostics.get(URI.file(this.filePath))
    );
    this.previousItems = [];
    this.line = line;
    this.lineNb = 0;
    this.scope = "";
    this.outsideScope = "";
    this.diagnostics = newDiagnostics;
    this.lastMMorES = undefined;
    this.inTypeDef = false;

    let funcIdx = 0;
    let mmIdx = 0;
    let typeIdx = 0;

    this.fileItems.tokens.forEach((e, i) => {
      if (
        !lastFunc ||
        ((lastFunc.kind === CompletionItemKind.Property ||
          (funcIdx > 0 &&
            this.funcsAndMethodsInFile[funcIdx - 1].kind ===
              CompletionItemKind.Property)) &&
          ["get", "set"].includes(e.id)) ||
        !lastFunc.fullRange.contains(e.range)
      ) {
        if (
          this.funcsAndMethodsInFile.length > funcIdx &&
          this.funcsAndMethodsInFile[funcIdx].fullRange.contains(e.range)
        ) {
          lastFunc = this.funcsAndMethodsInFile[funcIdx];
          funcIdx++;
        } else {
          lastFunc = undefined;
        }
      }

      if (!lastMMorES || !lastMMorES.fullRange.contains(e.range)) {
        if (
          this.MmEsInFile.length > mmIdx &&
          this.MmEsInFile[mmIdx].fullRange.contains(e.range)
        ) {
          lastMMorES = this.MmEsInFile[mmIdx];
          mmIdx++;
        } else {
          lastMMorES = undefined;
        }
      }

      if (
        this.typeDefAndSetInFile.length > 0 &&
        this.typeDefAndSetInFile.length > typeIdx
      ) {
        if (this.typeDefAndSetInFile[typeIdx].fullRange.contains(e.range)) {
          this.inTypeDef = true;
        } else if (this.inTypeDef) {
          // Check for typesets that are back to back.
          if (
            this.typeDefAndSetInFile.length > typeIdx + 1 &&
            this.typeDefAndSetInFile[typeIdx + 1].fullRange.contains(e.range)
          ) {
            typeIdx++;
          } else {
            this.inTypeDef = false;
            typeIdx++;
          }
        }
      }

      if (checkIfPluginInfo(e.id, lastFunc, lastMMorES)) {
        return;
      }

      const lineNb = e.range.start.line;

      if (lineNb - this.offset !== this.lineNb || i === 0) {
        this.lineNb = e.range.start.line - this.offset;

        this.line = this.lines[this.lineNb];
        this.previousItems = [];

        // Handle property getters and setters.
        if (
          lastMMorES &&
          lastMMorES.kind === CompletionItemKind.Class &&
          lastFunc &&
          lastFunc.kind === CompletionItemKind.Method &&
          ["get", "set"].includes(lastFunc.name)
        ) {
          if (
            funcIdx > 1 &&
            this.funcsAndMethodsInFile[funcIdx - 2].kind ===
              CompletionItemKind.Property
          ) {
            this.scope = `-${lastFunc.name}-${
              this.funcsAndMethodsInFile[funcIdx - 2].name
            }-${lastMMorES.name}`;
          } else if (
            funcIdx > 2 &&
            this.funcsAndMethodsInFile[funcIdx - 3].kind ===
              CompletionItemKind.Property
          ) {
            this.scope = `-${lastFunc.name}-${
              this.funcsAndMethodsInFile[funcIdx - 3].name
            }-${lastMMorES.name}`;
          }
        } else {
          this.scope = `-${lastFunc ? lastFunc.name : globalIdentifier}-${
            lastMMorES ? lastMMorES.name : globalIdentifier
          }`;
        }
        this.outsideScope = `-${globalIdentifier}-${
          lastMMorES ? lastMMorES.name : globalIdentifier
        }`;
        this.lastMMorES = lastMMorES;
      }
      handleToken.call(this, e.id, e.range);
    });
    // parserDiagnostics.set(URI.file(this.filePath), newDiagnostics);
  }
}
