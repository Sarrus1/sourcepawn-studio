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
import { FileItems } from "./spFilesRepository";
import {
  handleAddedDocument,
  handleDocumentChange,
  newDocumentCallback,
} from "./spFileHandlers";
import { getAllItems, getItemFromPosition } from "./spItemsGetters";
import { refreshDiagnostics } from "../Providers/spLinter";
import { refreshCfgDiagnostics } from "../Providers/cfgLinter";
import { updateDecorations } from "../Providers/decorationsProvider";

export class ItemsRepository implements Disposable {
  public fileItems: Map<string, FileItems>;
  public documents: Set<string>;
  private globalState: Memento;

  constructor(globalState: Memento) {
    this.fileItems = new Map<string, FileItems>();
    this.documents = new Set<string>();
    this.globalState = globalState;
  }

  public dispose() {}

  public handleAddedDocument(event: FileCreateEvent) {
    handleAddedDocument(this, event);
  }

  public handleDocumentChange(event: TextDocumentChangeEvent) {
    refreshDiagnostics(event.document);
    refreshCfgDiagnostics(event.document);
    handleDocumentChange(this, event);
    updateDecorations(this);
  }

  public handleNewDocument(document: TextDocument) {
    refreshDiagnostics(document);
    refreshCfgDiagnostics(document);
    newDocumentCallback(this, document.uri);
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
