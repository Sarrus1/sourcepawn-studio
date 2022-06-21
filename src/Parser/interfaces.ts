export interface FunctionParam {
  label: string;
  documentation: string;
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
