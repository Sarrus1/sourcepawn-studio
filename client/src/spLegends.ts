import { SemanticTokensLegend } from "vscode";

const tokenTypes = ["variable"];
const tokenModifiers = ["readonly"];
export const SP_LEGENDS = new SemanticTokensLegend(tokenTypes, tokenModifiers);
