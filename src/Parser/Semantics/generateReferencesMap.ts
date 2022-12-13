import { CompletionItemKind } from "vscode";

import { purgeReferences } from "../utils";
import { globalIdentifier } from "../../Misc/spConstants";
import { FunctionItem } from "../../Backend/Items/spFunctionItem";
import { MethodItem } from "../../Backend/Items/spMethodItem";
import { PropertyItem } from "../../Backend/Items/spPropertyItem";
import { MethodMapItem } from "../../Backend/Items/spMethodmapItem";
import { EnumStructItem } from "../../Backend/Items/spEnumStructItem";
import { VariableItem } from "../../Backend/Items/spVariableItem";
import { TypedefItem } from "../../Backend/Items/spTypedefItem";
import { TypesetItem } from "../../Backend/Items/spTypesetItem";
import { Semantics } from "./spSemantics";

/**
 * Generate the references map and the methodsAndProperties map of the Semantics object.
 * Will run `purgeReferences` on all selected items.
 * @param  {Semantics} this  The Semantics object.
 * @returns void
 */
export function generateReferencesMap(this: Semantics): void {
  const MC = [CompletionItemKind.Method, CompletionItemKind.Constructor];
  const MPC = [CompletionItemKind.Property].concat(MC);
  const MmEs = [CompletionItemKind.Class, CompletionItemKind.Struct];

  this.allItems.forEach((item, i) => {
    if (item.kind === CompletionItemKind.Variable) {
      purgeReferences(item, this.filePath);
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
      purgeReferences(item, this.filePath);
      this.referencesMap.set(item.name, item);
    } else if (MmEs.includes(item.kind)) {
      if (item.filePath === this.filePath) {
        this.MmEsInFile.push(item as MethodMapItem | EnumStructItem);
      }
      purgeReferences(item, this.filePath);
      this.referencesMap.set(item.name, item);
    } else if (item.kind === CompletionItemKind.TypeParameter) {
      if (item.filePath === this.filePath) {
        this.typeDefAndSetInFile.push(item as TypedefItem | TypesetItem);
      }
      purgeReferences(item, this.filePath);
      this.referencesMap.set(item.name, item);
    } else if (!MPC.includes(item.kind) && item.references !== undefined) {
      purgeReferences(item, this.filePath);
      this.referencesMap.set(item.name, item);
    } else if (item.kind === CompletionItemKind.Property) {
      if (item.filePath === this.filePath) {
        this.funcsAndMethodsInFile.push(item as PropertyItem);
      }
      purgeReferences(item, this.filePath);
      this.methodAndProperties.set(
        `${item.name}-${item.parent.name}`,
        item as PropertyItem
      );
    } else if (MC.includes(item.kind)) {
      if (item.filePath === this.filePath) {
        this.funcsAndMethodsInFile.push(item as MethodItem);
      }
      purgeReferences(item, this.filePath);
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
  ).filter((e) => e.fullRange.start.line >= this.offset);

  this.funcsAndMethodsInFile = this.funcsAndMethodsInFile
    .sort((a, b) => a.fullRange.start.line - b.fullRange.start.line)
    .filter((e) => e.fullRange.start.line >= this.offset);

  this.typeDefAndSetInFile = this.typeDefAndSetInFile
    .sort((a, b) => a.fullRange.start.line - b.fullRange.start.line)
    .filter((e) => e.fullRange.start.line >= this.offset);
}
