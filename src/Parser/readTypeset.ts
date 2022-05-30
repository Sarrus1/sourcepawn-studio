import {
  FuncenumDeclaration,
  spParserArgs,
  TypesetDeclaration,
} from "./interfaces";
import { TypesetItem } from "../Backend/Items/spTypesetItem";
import { parsedLocToRange } from "./utils";
import { processDocStringComment } from "./processComment";
import { TypedefItem } from "../Backend/Items/spTypedefItem";

/**
 * Process a typeset declaration.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {TypesetDeclaration|FuncenumDeclaration} res  Object containing the typeset/funcenum declaration details.
 * @returns void
 */
export function readTypeset(
  parserArgs: spParserArgs,
  res: TypesetDeclaration | FuncenumDeclaration
): void {
  const range = parsedLocToRange(res.id.loc, parserArgs);
  const fullRange = parsedLocToRange(res.loc, parserArgs);
  const { doc, dep } = processDocStringComment(res.doc);

  let childs: TypedefItem[];
  if (res.type === "TypesetDeclaration") {
    childs = res.body.map((e, i) => {
      const { doc: child_doc, dep: child_dep } = processDocStringComment(e.doc);
      const name = `${res.id.id}\$${i}`;
      return new TypedefItem(
        name,
        `typedef ${name} = function ${e.returnType.id} ();`,
        parserArgs.filePath,
        child_doc,
        e.returnType.id,
        undefined,
        undefined,
        []
      );
    });
  } else {
    childs = res.body.map((e, i) => {
      const { doc: child_doc, dep: child_dep } = processDocStringComment(e.doc);
      const name = `${res.id.id}\$${i}`;
      return new TypedefItem(
        name,
        `typedef ${name} = function ();`,
        parserArgs.filePath,
        child_doc,
        "",
        undefined,
        undefined,
        []
      );
    });
  }

  const typeDefItem = new TypesetItem(
    res.id.id,
    `typeset ${res.id.id} (${childs.length} members)`,
    parserArgs.filePath,
    doc,
    range,
    fullRange,
    childs
  );
  parserArgs.fileItems.items.push(typeDefItem);
}
