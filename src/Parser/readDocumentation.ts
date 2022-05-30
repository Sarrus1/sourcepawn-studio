import { SyntaxNode } from "web-tree-sitter";
import { DocString } from "./interfaces";
import { TreeWalker } from "./spParser";

/**
 * Try to find a documentation comment from the comments history of the TreeWalker.
 * @param  {TreeWalker} walker  TreeWalker used to find the documentation.
 * @param  {SyntaxNode} node    Node we are trying to find the documentation for.
 * @returns DocString           Extrapolated DocString.
 */
export function findDoc(walker: TreeWalker, node: SyntaxNode): DocString {
  const txt: string[] = [];
  let dep: string;
  let endIndex = node.startPosition.row;
  for (let comment of walker.comments.reverse()) {
    if (endIndex === comment.endPosition.row + 1) {
      txt.push(commentToDoc(comment.text));
      endIndex = comment.startPosition.row;
    } else {
      walker.comments = [];
    }
  }
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
