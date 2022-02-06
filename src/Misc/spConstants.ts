import { DocumentFilter, SemanticTokensLegend } from "vscode";

export const SP_MODE: DocumentFilter = {
  language: "sourcepawn",
  scheme: "file",
};

const tokenTypes = ["variable", "enumMember"];
const tokenModifiers = ["readonly", "declaration"];
export const SP_LEGENDS = new SemanticTokensLegend(tokenTypes, tokenModifiers);

export const globalIdentifier = "$GLOBAL";
