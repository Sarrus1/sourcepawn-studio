import { SyntaxNode } from "web-tree-sitter";
import { DocString } from "./interfaces";
import { TreeWalker } from "./spParser";

/**
 * Try to find a documentation comment from the comments history of the TreeWalker.
 * @param  {TreeWalker} walker  TreeWalker object used to find the documentation.
 * @param  {SyntaxNode} node    Node we are trying to find the documentation for.
 * @returns DocString           Extrapolated DocString.
 */
export function findDoc(walker: TreeWalker, node: SyntaxNode): DocString {
  const txt: string[] = [];
  let dep: string;
  let endIndex = node.startPosition.row;
  for (const deprec of walker.deprecated.reverse()) {
    if (endIndex === deprec.endPosition.row) {
      dep = deprec.childForFieldName("info")?.text;
      break;
    }
    if (endIndex > deprec.endPosition.row) {
      break;
    }
  }
  for (const comment of walker.comments.reverse()) {
    if (endIndex === comment.endPosition.row + (dep ? 2 : 1)) {
      txt.push(commentToDoc(comment.text));
      endIndex = comment.startPosition.row;
    } else {
      walker.comments = [];
    }
  }
  endIndex = node.startPosition.row;
  return { doc: txt.reverse().join("").trim(), dep };
}

/**
 * Convert a comment to a documentation string.
 * @param  {string} text  Comment to convert.
 * @returns string        Processed documentation string.
 */
export function commentToDoc(text: string): string {
  return (
    text
      // Remove leading /* and whitespace.
      .replace(/^\/\*\s*/, "")
      // Remove trailing */
      .replace(/\*\/$/, "")
      // Remove leading // and whitespace.
      .replace(/^\/\/\s*/, "")
  );
}
