import { CompletionItemKind, Diagnostic, DiagnosticSeverity } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname } from "path";
import { URI } from "vscode-uri";

import { ItemsRepository } from "../Backend/spItemsRepository";
import { FileItems } from "../Backend/spFilesRepository";
import { SPItem } from "../Backend/Items/spItems";
import { handleReferenceInParser } from "./handleReferencesInParser";
import {
  checkIfPluginInfo,
  getNextScope,
  parsedLocToRange,
  purgeCalls,
} from "./utils";
import { globalIdentifier } from "../Misc/spConstants";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { parserDiagnostics } from "../Providers/Linter/compilerDiagnostics";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { TypeDefItem } from "../Backend/Items/spTypedefItem";
import { TypeSetItem } from "../Backend/Items/spTypesetItem";
import { SemanticAnalyzer } from "./interfaces";
const spParser = require("./spParser2");

export function parseFile(
  file: string,
  items: FileItems,
  itemsRepository: ItemsRepository,
  searchTokens: boolean,
  IsBuiltIn: boolean
) {
  if (!existsSync(file)) {
    return;
  }
  let data = readFileSync(file, "utf-8");

  // Test for symbolic links
  let match = data.match(/^(?:\.\.\/)+(?:[\/\w\-])+\.\w+/);
  if (match !== null) {
    let folderpath = dirname(file);
    file = resolve(folderpath, match[0]);
    data = readFileSync(file, "utf-8");
  }
  parseText(data, file, items, itemsRepository, searchTokens, IsBuiltIn);
}

export interface spParserArgs {
  fileItems: FileItems;
  documents: Map<string, boolean>;
  filePath: string;
  IsBuiltIn: boolean;
  anonEnumCount: number;
  offset: number;
}

export function parseText(
  data: string,
  file: string,
  items: FileItems,
  itemsRepository: ItemsRepository,
  searchTokens: boolean,
  isBuiltIn: boolean,
  offset: number = 0
) {
  if (data === undefined) {
    return; // Asked to parse empty file
  }
  // Remove BOM if present
  if (data.charCodeAt(0) === 0xfeff) {
    data = data.substring(1);
  }
  if (!searchTokens) {
    const args: spParserArgs = {
      fileItems: items,
      documents: itemsRepository.documents,
      filePath: file,
      IsBuiltIn: isBuiltIn,
      anonEnumCount: 0,
      offset,
    };
    if (offset === 0) {
      // Only clear the diagnostics if there is no error.
      parserDiagnostics.set(URI.file(file), []);
    }
    try {
      spParser.args = args;
      const out: string = spParser.parse(data);
      //console.debug(out);
    } catch (err) {
      if (err.location !== undefined) {
        const range = parsedLocToRange(err.location, args);
        const diagnostic = new Diagnostic(
          range,
          err.message,
          DiagnosticSeverity.Error
        );
        const newDiagnostics = Array.from(
          parserDiagnostics.get(URI.file(file))
        );
        newDiagnostics.push(diagnostic);
        parserDiagnostics.set(URI.file(file), newDiagnostics);
        let { txt, newOffset } = getNextScope(
          data,
          err.location.start.line - 1
        );
        if (txt === undefined || offset === undefined) {
          return;
        }
        newOffset += offset;
        parseText(
          txt,
          file,
          items,
          itemsRepository,
          searchTokens,
          isBuiltIn,
          newOffset
        );
      }
    }
  } else {
    const lines = data.split("\n");
    const parser = new Parser(lines, file, items, itemsRepository);
    parser.parse();
  }
}

export class Parser {
  fileItems: FileItems;
  items: SPItem[];
  lines: string[];
  lineNb: number;
  filePath: string;
  methodAndProperties: Map<string, MethodItem | PropertyItem | VariableItem>;
  funcsAndMethodsInFile: (FunctionItem | MethodItem | PropertyItem)[];
  typeDefAndSetInFile: (TypeDefItem | TypeSetItem)[];
  MmEsInFile: (MethodMapItem | EnumStructItem)[];
  referencesMap: Map<string, SPItem>;

  constructor(
    lines: string[],
    filePath: string,
    completions: FileItems,
    itemsRepository: ItemsRepository
  ) {
    this.fileItems = completions;
    this.lineNb = 0;
    this.lines = lines;
    this.filePath = filePath;
    this.items = itemsRepository.getAllItems(URI.file(this.filePath));
    this.methodAndProperties = new Map();
    this.funcsAndMethodsInFile = [];
    this.MmEsInFile = [];
    this.referencesMap = new Map();
    this.typeDefAndSetInFile = [];
  }

  parse(): void {
    let line = this.lines[0];

    this.getReferencesMap();

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

    const thisArgs: SemanticAnalyzer = {
      parser: this,
      offset: 0,
      previousItems: [],
      line,
      lineNb: 0,
      scope: "",
      outsideScope: "",
      allItems: this.items,
      filePath: this.filePath,
      diagnostics: newDiagnostics,
      lastMMorES: undefined,
      inTypeDef: false,
    };

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
          thisArgs.inTypeDef = true;
        } else if (thisArgs.inTypeDef) {
          // Check for typesets that are back to back.
          if (
            this.typeDefAndSetInFile.length > typeIdx + 1 &&
            this.typeDefAndSetInFile[typeIdx + 1].fullRange.contains(e.range)
          ) {
            typeIdx++;
          } else {
            thisArgs.inTypeDef = false;
            typeIdx++;
          }
        }
      }

      if (checkIfPluginInfo(e.id, lastFunc, lastMMorES)) {
        return;
      }

      const lineNb = e.range.start.line;

      if (lineNb !== thisArgs.lineNb || i === 0) {
        thisArgs.lineNb = e.range.start.line;
        thisArgs.line = this.lines[thisArgs.lineNb];
        thisArgs.offset = 0;
        thisArgs.previousItems = [];

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
            thisArgs.scope = `-${lastFunc.name}-${
              this.funcsAndMethodsInFile[funcIdx - 2].name
            }-${lastMMorES.name}`;
          } else if (
            funcIdx > 2 &&
            this.funcsAndMethodsInFile[funcIdx - 3].kind ===
              CompletionItemKind.Property
          ) {
            thisArgs.scope = `-${lastFunc.name}-${
              this.funcsAndMethodsInFile[funcIdx - 3].name
            }-${lastMMorES.name}`;
          }
        } else {
          thisArgs.scope = `-${lastFunc ? lastFunc.name : globalIdentifier}-${
            lastMMorES ? lastMMorES.name : globalIdentifier
          }`;
        }
        thisArgs.outsideScope = `-${globalIdentifier}-${
          lastMMorES ? lastMMorES.name : globalIdentifier
        }`;
        thisArgs.lastMMorES = lastMMorES;
      }
      handleReferenceInParser.call(thisArgs, e.id, e.range);
    });
    parserDiagnostics.set(URI.file(this.filePath), newDiagnostics);
  }

  getReferencesMap(): void {
    const MC = [CompletionItemKind.Method, CompletionItemKind.Constructor];
    const MPC = [CompletionItemKind.Property].concat(MC);
    const MmEs = [CompletionItemKind.Class, CompletionItemKind.Struct];

    this.items.forEach((item, i) => {
      if (item.kind === CompletionItemKind.Variable) {
        purgeCalls(item, this.filePath);
        // Handle enum structs properties.
        if (item.parent.kind === CompletionItemKind.Struct) {
          this.referencesMap.set(
            `${item.name}-${globalIdentifier}-${item.parent.name}`,
            item
          );
          this.methodAndProperties.set(
            `${item.name}-${item.parent.name}`,
            item as VariableItem
          );
          return;
        }
        if (
          item.parent.kind === CompletionItemKind.Method &&
          item.parent.parent.kind === CompletionItemKind.Property
        ) {
          const key = `${item.name}-${item.parent.name}-${item.parent.parent.name}-${item.parent.parent.parent.name}`;
          this.referencesMap.set(key, item);
        }
        this.referencesMap.set(
          `${item.name}-${item.parent.name}-${
            item.parent.parent ? item.parent.parent.name : globalIdentifier
          }`,
          item
        );
      } else if (item.kind === CompletionItemKind.Function) {
        if (item.filePath === this.filePath) {
          this.funcsAndMethodsInFile.push(item as FunctionItem);
        }
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (MmEs.includes(item.kind)) {
        if (item.filePath === this.filePath) {
          this.MmEsInFile.push(item as MethodMapItem | EnumStructItem);
        }
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (item.kind === CompletionItemKind.TypeParameter) {
        if (item.filePath === this.filePath) {
          this.typeDefAndSetInFile.push(item as TypeDefItem | TypeSetItem);
        }
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (!MPC.includes(item.kind) && item.references !== undefined) {
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (item.kind === CompletionItemKind.Property) {
        if (item.filePath === this.filePath) {
          this.funcsAndMethodsInFile.push(item as PropertyItem);
        }
        purgeCalls(item, this.filePath);
        this.methodAndProperties.set(
          `${item.name}-${item.parent.name}`,
          item as PropertyItem
        );
      } else if (MC.includes(item.kind)) {
        if (item.filePath === this.filePath) {
          this.funcsAndMethodsInFile.push(item as MethodItem);
        }
        purgeCalls(item, this.filePath);
        if (item.parent.kind === CompletionItemKind.Property) {
          this.referencesMap.set(
            `${item.name}-${item.name}-${item.parent.name}-${item.parent.parent.name}`,
            item as MethodItem
          );
          return;
        }
        this.methodAndProperties.set(
          `${item.name}-${item.parent.name}`,
          item as MethodItem
        );
      }
    });

    this.MmEsInFile = this.MmEsInFile.sort(
      (a, b) => a.fullRange.start.line - b.fullRange.start.line
    );

    this.funcsAndMethodsInFile = this.funcsAndMethodsInFile.sort(
      (a, b) => a.fullRange.start.line - b.fullRange.start.line
    );

    this.typeDefAndSetInFile = this.typeDefAndSetInFile.sort(
      (a, b) => a.fullRange.start.line - b.fullRange.start.line
    );
  }
}
