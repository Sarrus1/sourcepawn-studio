import { CompletionItemKind, workspace as Workspace, Position } from "vscode";
import { existsSync, readFileSync } from "fs";
import { resolve, dirname, basename } from "path";
import { URI } from "vscode-uri";

import { ItemsRepository } from "../Backend/spItemsRepository";
import { FileItems } from "../Backend/spFilesRepository";
import { SPItem } from "../Backend/Items/spItems";
import { searchForReferencesInString } from "./searchForReferencesInString";
import { handleReferenceInParser } from "./handleReferencesInParser";
import { purgeCalls } from "./utils";
import { globalIdentifier, globalItem } from "../Misc/spConstants";
import { ParseState } from "./interfaces";
import { FunctionItem } from "../Backend/Items/spFunctionItem";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../Backend/Items/spEnumStructItem";
import { ConstantItem } from "../Backend/Items/spConstantItem";
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
  documents: Set<string>;
  filePath: string;
  IsBuiltIn: boolean;
  anonEnumCount: number;
}

export function parseText(
  data: string,
  file: string,
  items: FileItems,
  itemsRepository: ItemsRepository,
  searchTokens,
  isBuiltIn: boolean
) {
  if (data === undefined) {
    return; // Asked to parse empty file
  }
  // Remove BOM if present
  if (data.charCodeAt(0) === 0xfeff) {
    data = data.substring(1);
  }
  if (!searchTokens)
    try {
      const args: spParserArgs = {
        fileItems: items,
        documents: itemsRepository.documents,
        filePath: file,
        IsBuiltIn: isBuiltIn,
        anonEnumCount: 0,
      };
      spParser.args = args;
      const out: string = spParser.parse(data);
      //console.debug(out);
    } catch (e) {
      if (e.location !== undefined) {
        console.error(basename(file), e.message, e.location.start);
      }
      console.error(e);
    }
  else {
    const lines = data.split("\n");
    const parser = new Parser(lines, file, isBuiltIn, items, itemsRepository);
    parser.parse();
  }
}

export class Parser {
  fileItems: FileItems;
  items: SPItem[];
  lines: string[];
  lineNb: number;
  filePath: string;
  IsBuiltIn: boolean;
  documents: Set<string>;
  lastFuncLine: number;
  lastFunc: FunctionItem | ConstantItem;
  methodsAndProperties: (MethodItem | PropertyItem)[];
  funcsAndMethodsInFile: (FunctionItem | MethodItem)[];
  MmEsInFile: (MethodMapItem | EnumStructItem)[];
  referencesMap: Map<string, SPItem>;
  itemsRepository: ItemsRepository;
  debugging: boolean;
  anonymousEnumCount: number;
  deprecated: string | undefined;

  constructor(
    lines: string[],
    filePath: string,
    IsBuiltIn: boolean,
    completions: FileItems,
    itemsRepository: ItemsRepository
  ) {
    this.fileItems = completions;
    this.lineNb = 0;
    this.lines = lines;
    this.filePath = filePath;
    this.IsBuiltIn = IsBuiltIn;
    this.documents = itemsRepository.documents;
    this.lastFuncLine = -1;
    this.lastFunc = globalItem;
    // Get all the items from the itemsRepository for this file
    this.items = itemsRepository.getAllItems(URI.file(this.filePath));
    this.itemsRepository = itemsRepository;
    let debugSetting = Workspace.getConfiguration("sourcepawn").get(
      "trace.server"
    );
    this.debugging = debugSetting == "messages" || debugSetting == "verbose";
    this.anonymousEnumCount = 0;
    this.methodsAndProperties = [];
    this.funcsAndMethodsInFile = [];
    this.MmEsInFile = [];
    this.referencesMap = new Map<string, SPItem>();
  }

  parse(): void {
    let line = this.lines.shift();

    this.getReferencesMap();

    let lastFunc: FunctionItem | MethodItem | undefined;
    let lastMMorES: MethodMapItem | EnumStructItem | undefined;

    while (line !== undefined) {
      const pos = new Position(this.lineNb, 0);

      if (!lastFunc || !lastFunc.fullRange.contains(pos)) {
        if (
          this.funcsAndMethodsInFile.length > 0 &&
          this.funcsAndMethodsInFile[0].fullRange.contains(pos)
        ) {
          lastFunc = this.funcsAndMethodsInFile.shift();
        } else {
          lastFunc = undefined;
        }
      }

      if (!lastMMorES || !lastMMorES.fullRange.contains(pos)) {
        if (
          this.MmEsInFile.length > 0 &&
          this.MmEsInFile[0].fullRange.contains(pos)
        ) {
          lastMMorES = this.MmEsInFile.shift();
        } else {
          lastMMorES = undefined;
        }
      }

      const parseState: ParseState = {
        bComment: false,
        lComment: false,
        sString: false,
        dString: false,
      };

      searchForReferencesInString(line, handleReferenceInParser, {
        parser: this,
        parseState: parseState,
        scope: `-${lastFunc ? lastFunc.name : globalIdentifier}-${
          lastMMorES ? lastMMorES.name : globalIdentifier
        }`,
        offset: 0,
      });
      line = this.lines.shift();
      this.lineNb++;
    }
  }

  getReferencesMap(): void {
    const MP = [CompletionItemKind.Method, CompletionItemKind.Property];
    const MmEs = [CompletionItemKind.Class, CompletionItemKind.Struct];

    this.items.forEach((item) => {
      if (item.kind === CompletionItemKind.Variable) {
        purgeCalls(item, this.filePath);
        this.referencesMap.set(
          `${item.name}-${item.parent.name}-${
            item.parent.parent ? item.parent.parent.name : globalIdentifier
          }`,
          item
        );
      } else if (
        [CompletionItemKind.Function, CompletionItemKind.Method].includes(
          item.kind
        )
      ) {
        if (item.filePath === this.filePath) {
          this.funcsAndMethodsInFile.push(item as FunctionItem | MethodItem);
        }
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (MmEs.includes(item.kind)) {
        if (item.filePath === this.filePath) {
          this.MmEsInFile.push(item as MethodMapItem | EnumStructItem);
        }
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (!MP.includes(item.kind) && item.references !== undefined) {
        purgeCalls(item, this.filePath);
        this.referencesMap.set(item.name, item);
      } else if (MP.includes(item.kind) && item.references !== undefined) {
        this.methodsAndProperties.push(item as MethodItem | PropertyItem);
      }
    });

    this.MmEsInFile = this.MmEsInFile.sort(
      (a, b) => a.fullRange.start.line - b.fullRange.start.line
    );

    this.funcsAndMethodsInFile = this.funcsAndMethodsInFile.sort(
      (a, b) => a.fullRange.start.line - b.fullRange.start.line
    );
  }

  getAllMacros(items: SPItem[]): string[] {
    if (items == undefined) {
      return [];
    }
    let arr: string[] = [];
    for (let e of items) {
      if (e.kind === CompletionItemKind.Interface) {
        arr.push(e.name);
      }
    }
    return arr;
  }
}
