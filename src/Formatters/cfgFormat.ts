import {
  DocumentFormattingEditProvider,
  TextDocument,
  FormattingOptions,
  CancellationToken,
  ProviderResult,
  TextEdit,
  workspace as Workspace,
  Position,
  Range,
  window,
} from "vscode";
import { parse } from "../Parser/cfgParser/cfgParser";
import {
  Comment,
  KeyValue,
  ParserOutput,
  Section,
  Value,
} from "../Parser/cfgParser/cfgParserInterfaces";

export class CFGDocumentFormattingEditProvider
  implements DocumentFormattingEditProvider {
  public provideDocumentFormattingEdits(
    document: TextDocument,
    options: FormattingOptions,
    token: CancellationToken
  ): ProviderResult<TextEdit[]> {
    const workspaceFolder = Workspace.getWorkspaceFolder(document.uri);

    // Get the user's settings.
    const insertSpaces: boolean =
      Workspace.getConfiguration("editor", workspaceFolder).get(
        "insertSpaces"
      ) || false;
    const tabSize: number =
      Workspace.getConfiguration("editor", workspaceFolder).get("tabSize") || 2;

    // Apply user settings
    const start = new Position(0, 0);
    const end = new Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    const range = new Range(start, end);
    const formatter = new CfgFormat(insertSpaces, tabSize);
    let text = "";
    try {
      text = formatter.format(document.getText());
    } catch (err) {
      console.error(err);
    }

    // If process failed,
    if (text === "") {
      window.showErrorMessage(
        "The formatter failed to run, check the console for more details."
      );
      return undefined;
    }
    return [new TextEdit(range, text)];
  }
}

class CfgFormat {
  indent: number;
  output: string;
  parsed: ParserOutput;
  rawText: string;
  indentString: string;

  constructor(insertSpaces: boolean, tabSize: number) {
    this.indent = 0;
    this.output = "";
    this.indentString = this.makeIndentString(insertSpaces, tabSize);
  }

  /**
   * Parse a keyvalue file content and format it.
   */
  public format(text: string): string {
    this.rawText = text;
    this.parsed = parse(this.rawText);
    // Parse the top comment.
    let out = this.parsed.doc.map((e) => this.writeComment(e)).join("\n");
    if (out !== "") {
      // Add a newline if needed.
      out += "\n";
    }
    // Map all top KeyValue to their formatted string.
    out += this.parsed.keyvalues
      .map((e) => this.writeKeyValue(e, 0, true))
      .join("\n");
    return out;
  }

  /**
   * Generate an indent string which is used as base for indentation.
   * @param  {boolean} insertSpaces   Use spaces or tabs for indentation.
   * @param  {number} tabSize         Number of tabs/spaces to use.
   * @returns string
   */
  private makeIndentString(insertSpaces: boolean, tabSize: number): string {
    const base = insertSpaces ? " " : "\t";
    return base.repeat(tabSize);
  }

  /**
   * Write a KeyValue pair.
   * @param  {KeyValue} keyvalue      The KeyValue object from the parser.
   * @param  {number} maxKeyLength    The max key length for this section.
   * @param  {boolean} last           Whether it is the last KeyValue of the section.
   * @returns string
   */
  private writeKeyValue(
    keyvalue: KeyValue,
    maxKeyLength: number,
    last: boolean
  ): string {
    let out = `"${keyvalue.key.txt}"`;
    const diff = maxKeyLength - keyvalue.key.txt.length;
    if (diff > 0) {
      out += " ".repeat(diff);
    }
    if (keyvalue.doc.length > 0) {
      // Write the middle comment if it exists.
      out += this.indentString;
      out += keyvalue.doc.map((e) => this.writeComment(e, false)).join("\n");
    }
    if (keyvalue.value.type === "section") {
      out += "\n" + this.writeSection(keyvalue.value);
      if (!last) {
        out += "\n";
      }
      if (keyvalue.trailDoc.length > 0) {
        // Write the trailing comment if it exists.
        out +=
          "\n" +
          keyvalue.trailDoc.map((e) => this.writeComment(e, true)).join("\n");
      }
    } else {
      out += this.indentString + this.writeValue(keyvalue.value);
      if (keyvalue.trailDoc.length > 0) {
        // Write the trailing comment if it exists.
        out += this.indentString;
        out += keyvalue.trailDoc
          .map((e) => this.writeComment(e, false))
          .join("\n");
      }
    }

    out += "\n";
    return this.indentLine(out);
  }

  /**
   * Write a section.
   * @param  {Section} section  The Section object from the parser.
   * @returns string
   */
  private writeSection(section: Section): string {
    let output = "";
    section.doc.forEach((e) => {
      output += this.writeComment(e) + "\n";
    });
    output += this.indentLine("{\n");
    this.indent++;
    const offset = this.getMaxKeyLength(section.keyvalues);
    section.keyvalues.forEach((e, i) => {
      output += this.writeKeyValue(
        e,
        offset,
        i === section.keyvalues.length - 1
      );
    });
    this.indent--;
    output += this.indentLine("}");
    return output;
  }

  /**
   * Get the max key length of the section.
   * @param  {KeyValue[]} keyvalues The KeyValues of the section.
   * @returns number
   */
  private getMaxKeyLength(keyvalues: KeyValue[]): number {
    let offset = 0;
    keyvalues.forEach((e) => {
      if (e.value.type === "section") {
        return;
      }
      if (e.key.txt.length > offset) {
        offset = e.key.txt.length;
      }
    });
    return offset;
  }

  /**
   * Write a value.
   * @param  {Value} value  The Value object from the parser.
   * @returns string
   */
  private writeValue(value: Value): string {
    let out = `"${value.txt}"`;
    return out;
  }

  /**
   * Write a comment.
   * @param  {Comment} comment The comment object from the parser.
   * @param  {boolean} indent  Whether to indent the comment.
   * @returns string
   */
  private writeComment(comment: Comment, indent = true): string {
    let out = "";
    switch (comment.type) {
      case "MultiLineComment":
      case "MultiLineCommentNoLineTerminator":
        out = "/*" + comment.value + "*/";
        break;
      default:
        out = "//" + comment.value;
    }
    if (indent) {
      return this.indentLine(out);
    }
    return out;
  }

  /**
   * Indent a line according to the current state.
   * @param line  Line to indent.
   * @returns
   */
  private indentLine(line: string): string {
    let out = this.indentString.repeat(this.indent) + line;
    return out;
  }
}
