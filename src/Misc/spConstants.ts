import { DocumentFilter, SemanticTokensLegend } from "vscode";
import { ConstantItem } from "../Backend/Items/spConstantItem";

export const SP_MODE: DocumentFilter = {
  language: "sourcepawn",
  scheme: "file",
};

const tokenTypes = ["variable", "enumMember", "function", "class", "method"];
const tokenModifiers = ["readonly", "declaration", "deprecated"];

export const SP_LEGENDS = new SemanticTokensLegend(tokenTypes, tokenModifiers);

export const globalIdentifier = "$GLOBAL";
export const globalItem = new ConstantItem(globalIdentifier);
