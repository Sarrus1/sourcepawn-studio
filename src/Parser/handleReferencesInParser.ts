import { URI } from "vscode-uri";
import {
  CompletionItemKind,
  Diagnostic,
  DiagnosticSeverity,
  Location,
  Range,
} from "vscode";

import { globalIdentifier } from "../Misc/spConstants";
import { MethodItem } from "../Backend/Items/spMethodItem";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { checkIfConstructor } from "../spUtils";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { SemanticAnalyzer } from "./interfaces";

const globalScope = `-${globalIdentifier}-${globalIdentifier}`;

export function handleReferenceInParser(
  this: SemanticAnalyzer,
  name: string,
  range: Range
) {
  const start = range.start.character;

  if (name === "this") {
    let item = this.parser.items.find(
      (e) =>
        [CompletionItemKind.Struct, CompletionItemKind.Class].includes(
          e.kind
        ) &&
        this.parser.filePath == e.filePath &&
        e.fullRange.contains(range)
    );
    if (item !== undefined) {
      this.previousItems.push(item);
    }
    return;
  }
  let item =
    this.parser.referencesMap.get(name + this.scope) ||
    this.parser.referencesMap.get(name + this.outsideScope) ||
    this.parser.referencesMap.get(name + globalScope) ||
    this.parser.referencesMap.get(name);

  if (item === undefined && this.lastMMorES) {
    item = this.parser.methodAndProperties.get(
      name + "-" + this.lastMMorES.name
    );
  }

  // Handle positional arguments.
  if (item === undefined && !this.inTypeDef) {
    const lastFuncCall = this.previousItems
      .reverse()
      .find((e) => e.kind === CompletionItemKind.Function);
    if (lastFuncCall !== undefined) {
      item = this.parser.referencesMap.get(
        `${name}-${lastFuncCall.name}-${globalIdentifier}`
      );
    }
  }

  if (item !== undefined) {
    // Prevent double references.
    if (item.range.isEqual(range)) {
      return;
    }
    item = checkIfConstructor(
      item,
      range,
      this.parser.methodAndProperties,
      this.line
    );
    const location = new Location(URI.file(this.parser.filePath), range);
    item.references.push(location);
    this.previousItems.push(item);
  } else if (
    start > 0 &&
    this.previousItems.length > 0 &&
    [".", ":"].includes(this.line[start - 1]) &&
    !this.inTypeDef
  ) {
    let offset = 1;
    let item: MethodItem | PropertyItem | VariableItem;
    while (item === undefined && this.previousItems.length >= offset) {
      const parent = this.previousItems[this.previousItems.length - offset];
      offset++;
      if (parent.type === undefined) {
        continue;
      }
      item = this.parser.methodAndProperties.get(`${name}-${parent.type}`);
      if (item !== undefined) {
        break;
      }
      let inherit = this.allItems.find(
        (e) => e.kind === CompletionItemKind.Class && e.name === parent.type
      );
      if (inherit === undefined) {
        continue;
      }
      inherit = inherit.parent;
      while (
        inherit !== undefined &&
        inherit.kind !== CompletionItemKind.Constant
      ) {
        item = this.parser.methodAndProperties.get(`${name}-${inherit.name}`);
        if (item !== undefined) {
          break;
        }
        inherit = (inherit as MethodMapItem).parent;
      }
    }

    if (item !== undefined) {
      if (item.range.isEqual(range)) {
        return;
      }
      const location = new Location(URI.file(this.parser.filePath), range);
      item.references.push(location);
      this.previousItems.push(item);
      return;
    }
  }

  if (item === undefined && !this.inTypeDef) {
    this.diagnostics.push(
      new Diagnostic(
        range,
        `${name} is not defined`,
        DiagnosticSeverity.Warning
      )
    );
  }
}
