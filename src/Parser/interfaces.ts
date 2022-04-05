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

  /**
   * The start location object returned by the Peggy.js generated parser.
   */
  start: ParserLocationDetails;

  /**
   * The end location object returned by the Peggy.js generated parser.
   */
  end: ParserLocationDetails;
}

/**
 * The details of the parsed location.
 */
export interface ParserLocationDetails {
  /**
   * The global offset of the location.
   */
  offset: number;

  /**
   * The line of the location.
   */
  line: number;

  /**
   * The column of the location.
   */
  column: number;
}

/**
 * A parsed enum member.
 */
export interface ParsedEnumMember {
  /**
   * The id of the parsed enum member.
   */
  id: string;

  /**
   * The location of the parsed enum member.
   */
  loc: ParserLocation;

  /**
   * The trailing comment (if it exists) of the parsed enum member.
   */
  doc: string | undefined;
}

/**
 * Parsed define ID.
 */
export interface ParsedID {
  /**
   * Name of the Identifier.
   */
  id: string;

  /**
   * Location of the Identifier.
   */
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
  /**
   * The return type of the parsed typedef.
   */
  returnType: ParsedID;

  /**
   * The params of the typedef declaration.
   */
  params?: (ParsedParam[] | null)[] | null;
}

/**
 * Parsed variable declaration.
 */
export interface VariableDeclaration {
  type: string;
  id: ParsedID;
  init?: null;
}

/**
 * Parsed type of a parsed parameter.
 */
export interface ParameterType {
  /**
   * Modifier of the parsed parameter (such as & or []).
   */
  modifier: string | null;

  /**
   * Name of the type.
   */
  name: ParsedID;
}

/**
 * Parsed parameter in a formal parameter declaration.
 */
export interface ParsedParam {
  /**
   * Type of the parsed statement.
   */
  type: string;

  /**
   * Type of the parameter declaration (const, static, etc).
   */
  declarationType?: string[] | string | null;

  /**
   * Type of the parameter if it exists (int, char, etc).
   */
  parameterType?: ParameterType;

  /**
   * Id of the parsed parameter.
   */
  id: ParsedID | "...";

  /**
   * Default value of the parameter if it exists.
   */
  init?: string[] | null;
}
