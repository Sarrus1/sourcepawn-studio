/**
 * Range.
 */
export interface ParserRange {
  /**
   * Start position.
   */
  start: ParserPos;

  /**
   * End position.
   */
  end: ParserPos;
}

/**
 * Position.
 */
export interface ParserPos {
  /**
   * Offset.
   */
  offset: number;

  /**
   * Zero based line index.
   */
  line: number;

  /**
   * Zero based column index.
   */
  column: number;
}

/**
 * Output of the parser.
 */
export interface ParserOutput {
  /**
   * Initial comment.
   */
  doc: Comment[];

  /**
   * Key/Values combinations.
   */
  keyvalues: KeyValue[];
}

/**
 * KeyValue combination with their comments.
 */
export interface KeyValue {
  /**
   * Comment between the key and the value.
   */
  doc: Comment[];

  /**
   * Comment trailing the value.
   */
  trailDoc: Comment[];

  /**
   * Key
   */
  key: Key;

  /**
   * Value which can be a Section or a string.
   */
  value: Value | Section;
}

/**
 * Key.
 */
export interface Key {
  /**
   * Range of the key.
   */
  loc: ParserRange;

  /**
   * Key.
   */
  txt: string;

  /**
   * Type id.
   */
  type: "key";
}

/**
 * Value associated to a key.
 */
export interface Value {
  /**
   * Range of the value.
   */
  loc: ParserRange;

  /**
   * Value.
   */
  txt: string;

  /**
   * Type id.
   */
  type: "value";
}

/**
 * Section/Keyvalue pair.
 */
export interface Section {
  /**
   * Comment before the first key.
   */
  doc: Comment[];

  /**
   * Keyvalue pair of the section.
   */
  keyvalues: KeyValue[];

  /**
   * Type id.
   */
  type: "section";
}

/**
 * Comment.
 */
export interface Comment {
  /**
   * Type of the comment.
   */
  type:
    | "SingleLineComment"
    | "MultiLineComment"
    | "MultiLineCommentNoLineTerminator";

  /**
   * Range of the comment.
   */
  loc: ParserRange;

  /**
   * Text of the comment, without its delimiters (i.e // and /* *\/
   */
  value: string;
}
