import { FormatterConfig, formatKeyvalue } from "valve_kv_tools";
import {
  DocumentFormattingEditProvider,
  TextDocument,
  FormattingOptions,
  CancellationToken,
  ProviderResult,
  TextEdit,
  Position,
  Range,
  window,
} from "vscode";

export class KVDocumentFormattingEditProvider
  implements DocumentFormattingEditProvider
{
  public provideDocumentFormattingEdits(
    document: TextDocument,
    options: FormattingOptions,
    token: CancellationToken
  ): ProviderResult<TextEdit[]> {
    const start = new Position(0, 0);
    const end = new Position(
      document.lineCount - 1,
      document.lineAt(document.lineCount - 1).text.length
    );
    const config = new FormatterConfig(
      !options.insertSpaces,
      options.tabSize,
      1
    );
    const range = new Range(start, end);
    try {
      const text = formatKeyvalue(document.getText(), config);
      return [new TextEdit(range, text)];
    } catch (err) {
      console.error(err);
      window.showErrorMessage("Your syntax is invalid.");
      return undefined;
    }
  }
}
