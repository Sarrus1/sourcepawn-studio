import { PropertyDeclaration, spParserArgs } from "./interfaces";
import { PropertyItem } from "../Backend/Items/spPropertyItem";
import { parsedLocToRange } from "./utils";
import { MethodMapItem } from "../Backend/Items/spMethodmapItem";
import { processDocStringComment } from "./processComment";

/**
 * Process a methodmap's property.
 * @param  {spParserArgs} parserArgs  The parserArgs objects passed to the parser.
 * @param  {MethodMapItem} methodmapItem  The parent of the property.
 * @param  {PropertyDeclaration} res  Object containing the property declaration details.
 * @returns void
 */
export function readProperty(
  parserArgs: spParserArgs,
  methodmapItem: MethodMapItem,
  res: PropertyDeclaration
): void {
  const range = parsedLocToRange(res.id.loc, parserArgs);
  const fullRange = parsedLocToRange(res.loc, parserArgs);
  const { doc, dep } = processDocStringComment(res.doc);
  res.txt = res.txt.trim();
  const propertyItem = new PropertyItem(
    methodmapItem,
    res.id.id,
    parserArgs.filePath,
    res.txt,
    doc,
    range,
    fullRange,
    res.propertyType.id
  );
  parserArgs.fileItems.items.push(propertyItem);
  res.body.forEach((e) => {
    // readFunctionAndMethod(
    //   parserArgs,
    //   e.accessModifier,
    //   e.returnType,
    //   e.id,
    //   e.loc,
    //   e.doc,
    //   e.params,
    //   e.body,
    //   e.txt,
    //   propertyItem
    // );
  });
}
