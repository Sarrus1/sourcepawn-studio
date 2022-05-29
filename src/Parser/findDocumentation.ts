import { SyntaxNode } from "web-tree-sitter";
import { DocString } from "./interfaces";
import { TreeWalker } from "./spParser";

/**
 * Process a parsed comment and try to extrapolate a doc comment from it.
 * This will handle `#pragma deprecated`.
 * @param  {ParsedComment} docstring  The parsed comment to analyse.
 * @returns {DocString}
 */
export function findDocumentation(
  walker: TreeWalker,
  node: SyntaxNode,
  trailing: boolean
): DocString {
  if (trailing) {
    // TODO: Handle trailing comments.
    return { doc: undefined, dep: undefined };
  }

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
