import { SyntaxNode } from "web-tree-sitter";

import { TypesetItem } from "../Backend/Items/spTypesetItem";
import { TypedefItem } from "../Backend/Items/spTypedefItem";
import { TreeWalker } from "./spParser";
import { findDoc } from "./readDocumentation";
import { pointsToRange } from "./utils";

/**
 * Process a typeset declaration.
 * @param  {TreeWalker} walker            TreeWalker object.
 * @param  {TreeSitter.SyntaxNode} node   Node to process.
 * @returns void
 */
export function readTypeset(walker: TreeWalker, node: SyntaxNode): void {
  const nameNode = node.childForFieldName("name");
  const { doc, dep } = findDoc(walker, node);

  const childs = [];
  let counter = -1;
  node.children.forEach((e) => {
    if (e.type === "comment") {
      walker.pushComment(e);
      return;
    }
    if (e.type !== "typedef_expression") {
      return;
    }
    counter++;
    const { doc: child_doc, dep: child_dep } = findDoc(walker, e);
    const typeNode = e.childForFieldName("returnType");
    childs.push(
      new TypedefItem(
        `${nameNode.text}$${counter}`,
        e.text,
        walker.filePath,
        child_doc,
        typeNode.text,
        undefined,
        pointsToRange(e.startPosition, e.endPosition),
        e.children
          .find((e) => e.type === "typedef_args")
          .children.filter((e) => e.type === "typedef_arg")
      )
    );
  });

  const typeDefItem = new TypesetItem(
    nameNode.text,
    `typeset ${nameNode.text} (${childs.length} members)`,
    walker.filePath,
    doc,
    pointsToRange(nameNode.startPosition, nameNode.endPosition),
    pointsToRange(node.startPosition, node.endPosition),
    childs
  );
  walker.fileItem.items.push(typeDefItem);
}
