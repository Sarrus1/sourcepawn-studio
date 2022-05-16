import { FileItem } from "../Backend/spFilesRepository";

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
 * Declaration of an enum.
 */
export interface EnumDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "EnumDeclaration";

  /**
   * ID of the enum, if it exists.
   */
  id: ParsedID | null;

  /**
   * Location of the enum.
   */
  loc: ParserLocation;

  /**
   * Body of the enum.
   */
  body: EnumMemberDeclaration[];

  /**
   * Documentation of the enum.
   */
  doc: ParsedComment;
}

/**
 * Enum member declaration.
 */
export interface EnumMemberDeclaration {
  /**
   * ID of the enum member.
   */
  id: ParsedID;

  /**
   * The trailing comment  of the parsed enum member.
   */
  doc: ParsedComment;
}

/**
 * Enum struct member declaration.
 */
export type EnumstructMemberDeclaration =
  | MethodDeclaration
  | VariableDeclaration;

/**
 * Enum struct declaration.
 */
export interface EnumstructDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "EnumstructDeclaration";

  /**
   * ID of the enum struct.
   */
  id: ParsedID;

  /**
   * Location of the enum struct.
   */
  loc: ParserLocation;

  /**
   * Body of the enum struct.
   */
  body: EnumstructMemberDeclaration[] | null;

  /**
   * Documentation of the enum struct.
   */
  doc: ParsedComment;
}

/**
 * Parsed ID.
 */
export interface ParsedID {
  /**
   * Name of the identifier.
   */
  id: string;

  /**
   * Location of the identifier.
   */
  loc: ParserLocation;
}

/**
 * Object containing a processed doc comment and a deprecation notice.
 */
export interface DocString {
  /**
   * Processed doc comment, if it exists.
   */
  doc: string | undefined;

  /**
   * Processed deprecated warning, if it exists.
   */
  dep: string | undefined;
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
export interface FormalParameter {
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
 * Declaration of a typedef.
 */
export interface TypedefDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "TypedefDeclaration";

  /**
   * ID of the typedef.
   */
  id: ParsedID;

  /**
   * Location of the typedef.
   */
  loc: ParserLocation;

  /**
   * Body of the typedef.
   */
  body: TypedefBody;

  /**
   * Documentation of the typedef.
   */
  doc: ParsedComment;
}

/**
 * Body of a parsed typedef.
 */
export interface TypedefBody {
  /**
   * Return type of the parsed typedef.
   */
  returnType: ParsedID;

  /**
   * Params of the typedef declaration.
   */
  params: FormalParameter[];

  /**
   * Doc of the typedef when declared in a typeset. Will be empty for a regular typeset.
   */
  doc: ParsedComment;
}

/**
 * Declaration of a funcenum member.
 */
export interface FuncenumMemberDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "FuncenumMemberDeclaration";

  /**
   * ID of the funcenum member, if it exists.
   */
  id: ParsedID | null;

  /**
   * Access modifier of the funcenum member.
   */
  accessModifier: "public";

  /**
   * Params of the the funcenum member.
   */
  params: FormalParameter[];

  /**
   * Documentation of the funcenum member.
   */
  doc: ParsedComment;
}

/**
 * Declaration of a funcenum.
 */
export interface FuncenumDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "FuncenumDeclaration";

  /**
   * ID of the funcenum.
   */
  id: ParsedID;

  /**
   * Location of the the funcenum.
   */
  loc: ParserLocation;

  /**
   * Body of the the funcenum.
   */
  body: FuncenumMemberDeclaration[];

  /**
   * Documentation of the the funcenum.
   */
  doc: ParsedComment;
}

/**
 * Declaration of a typeset.
 */
export interface TypesetDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "TypesetDeclaration";

  /**
   * ID of the typeset.
   */
  id: ParsedID;

  /**
   * Location of the typeset.
   */
  loc: ParserLocation;

  /**
   * Body of the typeset.
   */
  body: TypedefBody[];

  /**
   * Documentation of the typeset.
   */
  doc: ParsedComment;
}

/**
 * Declaration of a functag.
 */
export interface FunctagDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "FunctagDeclaration";

  /**
   * ID of the functag.
   */
  id: ParsedID;

  /**
   * Location of the functag.
   */
  loc: ParserLocation;

  /**
   * Body of the functag.
   */
  body: FunctagBody;

  /**
   * Documentation of the functag.
   */
  doc: ParsedComment;
}

/**
 * Body of a parsed Functag.
 */
export interface FunctagBody {
  /**
   * Return type of the parsed typedef.
   */
  returnType: ParsedID | undefined;

  /**
   * Params of the typedef declaration.
   */
  params: FormalParameter[] | null;
}

/**
 * Declaration of a methodmap.
 */
export interface MethodmapDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "MethodmapDeclaration";

  /**
   * ID of the methodmap.
   */
  id: ParsedID;

  /**
   * Location of the methodmap.
   */
  loc: ParserLocation;

  /**
   * Inherit of the methodmap, if it exists.
   */
  inherit: ParsedID | "__nullable__" | null;

  /**
   * Body of the methodmap.
   */
  body: MethodmapBody;

  /**
   * Documentation of the methodmap.
   */
  doc: ParsedComment;
}

/**
 * Body of a methodmap.
 */
export type MethodmapBody = (
  | PropertyDeclaration
  | MethodDeclaration
  | MethodmapNativeForwardDeclaration
)[];

/**
 * Parsed variable declaration (list or single variable).
 */
export interface VariableDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "VariableDeclaration";

  /**
   * Access modifiers of the variable.
   */
  accessModifiers: VariableAcessModifiers[] | null;

  /**
   * Type of the variable.
   */
  variableType: VariableType | null;

  /**
   * Variable declarations.
   */
  declarations: VariableDeclarator[];

  /**
   * Documentation of the variable.
   */
  doc: ParsedComment;
}

/**
 * Variable access modifiers.
 */
export type VariableAcessModifiers = "public" | "stock" | "const" | "static";

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

/**
 * Declaration of a methodmap's property.
 */
export interface PropertyDeclaration {
  /**
   * Generic type of the declaration.
   */
  type: "PropertyDeclaration";

  /**
   * ID of the type of the property.
   */
  propertyType: ParsedID;

  /**
   * ID of the property.
   */
  id: ParsedID;

  /**
   * Documentation of the property.
   */
  doc: ParsedComment;

  /**
   * Location of the property.
   */
  loc: ParserLocation;

  /**
   * Body of the property.
   */
  body: (MethodDeclaration | MethodmapNativeForwardDeclaration)[];

  /**
   * Raw text of the property declaration.
   */
  txt: string;
}

export interface MethodDeclaration {
  type: "MethodDeclaration";
  accessModifier: string[];
  returnType: ParsedID;
  loc: ParserLocation;
  id: ParsedID;
  params: FormalParameter[];
  doc: ParsedComment;
  body: FunctionBody;
  txt: string;
}

export interface MethodmapNativeForwardDeclaration {
  type: "MethodmapNativeForwardDeclaration";
  accessModifier: string[];
  returnType: ParsedID;
  loc: ParserLocation;
  id: ParsedID;
  params: FormalParameter[];
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
  params: FormalParameter[];
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
  type: "PragmaValue" | "PreprocessorStatement" | "MacroDeclaration";

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
 * Define statement
 */
export interface DefineStatement {
  /**
   * Generic type of the declaration.
   */
  type: "DefineStatement";

  /**
   * ID of the define.
   */
  id: ParsedID;

  /**
   * Location of the define.
   */
  loc: ParserLocation;

  /**
   * Value of the define.
   */
  value: string | null;

  /**
   * Documentation of the define.
   */
  doc: ParsedComment;
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
  | null;

export interface LineTerminatorSequence {
  type: "LineTerminatorSequence";
  content: LineTerminatorSequenceContent;
}

export type LineTerminatorSequenceContent = (
  | string
  | PreprocessorStatement
  | IncludeStatement
  | DefineStatement
  | null
)[];
