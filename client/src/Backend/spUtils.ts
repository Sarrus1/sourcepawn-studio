import { Position } from "vscode";

/**
 * Checks whether or not a string calls a method, a property or a namespace.
 * The following strings will return true:
 * "foo.bar();"
 * "foo.bar;"
 * "foo::bar;"
 * @param  {string} line
 * @param  {Position} position
 * @returns boolean
 */
export function isMethodCall(line: string, position: Position): boolean {
  return /(?:\.|\:\:)\w*$/.test(line.slice(0, position.character));
}
