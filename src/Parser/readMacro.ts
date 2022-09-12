import { SyntaxNode } from "web-tree-sitter";
import { MacroItem } from "../Backend/Items/spMacroItem";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { findDoc } from "./readDocumentation";

import { TreeWalker } from "./spParser";
import { pointsToRange } from "./utils";

/**
 * Process an enum declaration.
 * @param  {TreeWalker} walker  TreeWalker object.
 * @param  {SyntaxNode} node    Node to process.
 * @returns void
 */
export function readMacro(walker: TreeWalker, node: SyntaxNode): void {
  const nameNode = node.childForFieldName("name");
  const { doc, dep } = findDoc(walker, node);
  const macroItem = new MacroItem(
    nameNode.text,
    node.text,
    doc,
    walker.filePath,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    "any",
    pointsToRange(node.startPosition, node.endPosition),
    dep,
    []
  );
  walker.fileItem.items.push(macroItem);
  addParamsToMacro(macroItem, doc);
}

function addParamsToMacro(macroItem: MacroItem, doc: string) {
  let matchParams = macroItem.detail.match(/\((?:(%\d),?)+\)/);
  if (!matchParams) {
    return;
  }
  let nbParams = matchParams[0].match(/%/g).length;
  for (let i = 1; i <= nbParams; i++) {
    const match = doc.match(new RegExp(`@param\\s+(?:\\b${i}\\b)([^\\@]+)`));
    if (!match) {
      continue;
    }
    const documentation = match[1].replace(/\*/gm, "").trim();
    const variableItem = new VariableItem(
      `%${i}`,
      macroItem.filePath,
      macroItem,
      macroItem.range,
      "any",
      `param ${i}`,
      documentation,
      []
    );
    macroItem.params.push(variableItem);
  }
}
