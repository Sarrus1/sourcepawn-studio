import * as TreeSitter from "web-tree-sitter";

import { globalItem } from "../Misc/spConstants";
import { VariableItem } from "../Backend/Items/spVariableItem";
import { FileItem } from "../Backend/spFilesRepository";
import { pointsToRange } from "./utils";

/**
 * Process a global variable declaration.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {MethodmapDeclaration} res  The object containing the variable declaration details.
 * @returns void
 */
export function readVariable(
  fileItem: FileItem,
  node: TreeSitter.SyntaxNode,
  filePath: string
): void {
  const variableType = node.childForFieldName("type").text;
  let storageClass = [];
  for (let child of node.children) {
    // FIXME: More efficient way to do this ?
    if (child.type === "variable_storage_class") {
      // FIXME: Only works for 0 or 1 storage class.
      storageClass.push(child.text);
      continue;
    }
    if (child.type !== "variable_declaration") {
      continue;
    }
    const declaration = child.childForFieldName("name");
    const variableItem = new VariableItem(
      declaration.text,
      filePath,
      globalItem,
      pointsToRange(declaration.startPosition, declaration.endPosition),
      variableType,
      "detail",
      "doc",
      storageClass
    );
    fileItem.items.push(variableItem);
  }
}
