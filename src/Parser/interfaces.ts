import { FileItem } from "../Backend/spFilesRepository";

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
 * The `args` object passed to the peggy.js parser.
 */
export interface spParserArgs {
  /**
   * The FileItem object of the file being parsed.
   */
  fileItems: FileItem;

  /**
   * The documents object of the FileItems object.
   */
  documents: Map<string, boolean>;

  /**
   * The path of the file being parsed.
   */
  filePath: string;

  /**
   * Is the file being parsed a Sourcemod builtin ?
   */
  IsBuiltIn: boolean;

  /**
   * The counter for anonymous enums.
   */
  anonEnumCount: number;

  /**
   * The start line offset of the file being parsed when the parser has recovered from an error.
   */
  offset: number;

  /**
   * The variable declarations for the current scope of the parser.
   * Used in the readFunctionsAndMethods callback, to parse the variables faster than using recursion.
   */
  variableDecl: ScoppedVariablesDeclaration;
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
  doc: ParsedComment;
}

/**
 * A parsed enum struct member.
 */
export type ParsedEnumStructMember = MethodDeclaration | VariableDeclaration;

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
  returnType: ParsedID | undefined;

  /**
   * The params of the typedef declaration.
   */
  params?: (ParsedParam[] | null)[] | null;
}

/**
 * Parsed type of a parsed parameter.
 */
export interface VariableType {
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
  parameterType?: VariableType | null;

  /**
   * Id of the parsed parameter.
   */
  id: ParsedID;

  /**
   * Default value of the parameter if it exists.
   */
  init?: string[] | null;
}

export interface FunctionParam {
  label: string;
  documentation: string;
}

/**
 * Parsed variable declaration (list or single variable).
 */
export interface VariableDeclaration {
  type: "VariableDeclaration";
  variableDeclarationType: string[] | string | null;
  variableType: VariableType | null;
  declarations: VariableDeclarator[];
  doc: ParsedComment;
}

export interface LocalVariableDeclaration {
  type: "LocalVariableDeclaration";
  content: VariableDeclaration;
}

export interface ForLoopVariableDeclaration {
  type: "ForLoopVariableDeclaration";
  content: ForLoopVariableDeclaration;
}

export type ScoppedVariablesDeclaration = (
  | LocalVariableDeclaration
  | ForLoopVariableDeclaration
)[];

export interface PropertyDeclaration {
  type: "PropertyDeclaration";
  propertyType: ParsedID;
  id: ParsedID;
  doc: ParsedComment;
  loc: ParserLocation;
  body;
  txt: string;
}

export interface MethodDeclaration {
  type: "MethodDeclaration";
  accessModifier: string[];
  returnType: ParsedID;
  loc: ParserLocation;
  id: ParsedID;
  params: ParsedParam[];
  doc: ParsedComment;
  body;
  txt: string;
}

export interface MethodmapNativeForwardDeclaration {
  type: "MethodmapNativeForwardDeclaration";
  accessModifier: string[];
  returnType: ParsedID;
  loc: ParserLocation;
  id: ParsedID;
  params: ParsedParam[];
  doc: ParsedComment;
  body: undefined;
  txt: string;
}

/**
 * Single variable of a list of variable.
 */
export interface VariableDeclarator {
  type: "VariableDeclarator";
  arrayInitialer: string | null;
  id: ParsedID;
  init: any;
}

export interface FunctionDeclaration {
  type: "FunctionDeclaration";
  accessModifier: string[];
  returnType: ParsedID | undefined;
  id: ParsedID;
  loc: ParserLocation;
  params: ParsedParam[];
  body: FunctionBody;
  txt: string;
}

export interface FunctionBody {
  type: "BlockStatement";
  body: any[] | null;
}

/**
 * Parsed preprocessor statement
 */
export interface PreprocessorStatement {
  /**
   * The type of the preprocessor statement ("PragmaValue" for exemple).
   */
  type:
    | "PragmaValue"
    | "PreprocessorStatement"
    | "DefineStatement"
    | "MacroDeclaration";

  /**
   * The ID of the preprocessor statement. Only for "DefineStatement" and "MacroStatement".
   */
  id?: ParsedID;

  /**
   * The value of the preprocessor statement.
   */
  value?: string;
}

/**
 * Parsed Include statement.
 */
export interface IncludeStatement {
  /**
   * The type of the preprocessor statement.
   */
  type: "IncludeStatement";

  /**
   * The path of the include, between the <> or "".
   */
  path: string;

  /**
   * The location of the <path> or "path".
   */
  loc: ParserLocation;
}

export type RawComment =
  | MultiLineComment
  | MultiLineCommentNoLineTerminator
  | SingleLineComment;

export interface SingleLineComment {
  /**
   * The type of the comment.
   */
  type: "SingleLineComment";

  /**
   * The content of the comment.
   */
  text: string;
}

export interface MultiLineComment {
  /**
   * The type of the comment.
   */
  type: "MultiLineComment";

  /**
   * The content of the comment.
   */
  text: string;
}
export interface MultiLineCommentNoLineTerminator {
  /**
   * The type of the comment.
   */
  type: "MultiLineCommentNoLineTerminator";

  /**
   * The content of the comment.
   */
  text: string;
}

export type ParsedComment =
  | (RawComment | LineTerminatorSequence)[]
  | RawComment
  | undefined;

export interface LineTerminatorSequence {
  type: "LineTerminatorSequence";
  content: LineTerminatorSequenceContent;
}

export type LineTerminatorSequenceContent = (
  | string
  | PreprocessorStatement
  | IncludeStatement
  | null
)[];
