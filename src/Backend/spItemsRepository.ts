import {
  Memento,
  Disposable,
  TextDocument,
  Position,
  CompletionList,
  FileCreateEvent,
  TextDocumentChangeEvent,
} from "vscode";
import { URI } from "vscode-uri";

import { SPItem } from "./Items/spItems";
import { events } from "../Misc/sourceEvents";
import { FileItem } from "./spFilesRepository";
import {
  handleAddedDocument,
  documentChangeCallback,
  isSPFile,
  newDocumentCallback,
} from "./spFileHandlers";
import { getAllItems, getItemFromPosition } from "./spItemsGetters";
import { refreshDiagnostics } from "../Providers/spLinter";
import { refreshCfgDiagnostics } from "../Providers/cfgLinter";
import { updateDecorations } from "../Providers/spDecorationsProvider";
import { performance } from "perf_hooks";

export class ItemsRepository implements Disposable {
  public fileItems: Map<string, FileItem>;
  public documents: Map<string, boolean>;
  private globalState: Memento;

  constructor(globalState: Memento) {
    this.fileItems = new Map<string, FileItem>();
    this.documents = new Map<string, boolean>();
    this.globalState = globalState;
  }

  public dispose() {}

  public handleAddedDocument(event: FileCreateEvent) {
    handleAddedDocument(this, event);
  }

  public handleDocumentChange(event: TextDocumentChangeEvent) {
    if (event.contentChanges.length === 0) {
      return;
    }
    if (isSPFile(event.document.uri.fsPath)) {
      refreshDiagnostics(event.document);
      documentChangeCallback(this, event);
      updateDecorations(this);
      return;
    }
    refreshCfgDiagnostics(event.document);
  }

  public handleNewDocument(document: TextDocument) {
    if (isSPFile(document.uri.fsPath)) {
      refreshDiagnostics(document);
      newDocumentCallback(this, document.uri);
      return;
    }
    refreshCfgDiagnostics(document);
  }

  public handleDocumentOpening(filePath: string) {
    newDocumentCallback(this, URI.file(filePath));
  }

  public getEventCompletions(): CompletionList {
    return new CompletionList(events);
  }

  public getAllItems(uri: URI): SPItem[] {
    return getAllItems(this, uri);
  }

  public getItemFromPosition(
    document: TextDocument,
    position: Position
  ): SPItem[] {
    return getItemFromPosition(this, document, position);
  }
}
