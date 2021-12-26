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

/**
 * Appends ".inc" to a an include file path if no extension is present.
 * @param  {string} file    The include file path to append the extension to.
 * @returns string
 */
export function getIncludeExtension(file: string): string {
  if (!/.sp\s*$/g.test(file) && !/.inc\s*$/g.test(file)) {
    file += ".inc";
  }
  return file;
}
