import {SemanticTokensLegend} from "vscode";

const tokenTypes = ['class', 'interface', 'enum', 'function', 'variable'];
const tokenModifiers = ['declaration', 'documentation'];
export const SP_LEGENDS = new SemanticTokensLegend(tokenTypes, tokenModifiers);