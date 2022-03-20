/**
 * An object which handles the state of a parser, by keeping track of
 * whether the parser is in a comment or a string.
 */
export interface ParseState {
  /**
   * Whether the parser is in a block comment or not.
   */
  bComment: boolean;

  /**
   * Whether the parser is in a line comment or not.
   */
  lComment: boolean;

  /**
   * Whether the parser is in a string delimited by single quotes (') or not.
   */
  sString: boolean;

  /**
   * Whether the parser is in a string delimited by double quotes (") or not.
   */
  dString: boolean;
}

/**
 * The location object returned by the Peggy.js generated parser.
 */
export interface ParserLocation {
  source: any;
  start: Start;
  end: End;
}

/**
 * The start location object returned by the Peggy.js generated parser.
 */
export interface Start {
  offset: number;
  line: number;
  column: number;
}

/**
 * The end location object returned by the Peggy.js generated parser.
 */
export interface End {
  offset: number;
  line: number;
  column: number;
}

/**
 * A parsed enum member.
 */
export interface ParsedEnumMember {
  id: string;
  loc: ParserLocation;
  doc: string | undefined;
}

/**
 * Parsed define ID.
 */
export interface ParsedID {
  id: string;
  loc: ParserLocation;
}

/**
 * An object which contains a parsed doc comment and a deprecation notice.
 */
export interface DocString {
  doc: string | undefined;
  dep: string | undefined;
}

/**
 * Body of a parsed TypeDef.
 */
export interface TypeDefBody {
  returnType: ParsedID;
  params?: (ParamsEntity[] | null)[] | null;
}

/**
 * Params of a TypeDef.
 */
export interface ParamsEntity {
  type: string;
  declarationType?: null;
  parameterType: ParsedID;
  init?: null;
  id: ParsedID;
}

export interface VariableDeclarations {
  type: string;
  id: ParsedID;
  init?: null;
}

export interface ParsedFunctionParam {
  type: string;
  declarationType?: string[] | string | null;
  parameterType?: ParsedID;
  init?: string[] | null;
  id: ParsedID;
}
