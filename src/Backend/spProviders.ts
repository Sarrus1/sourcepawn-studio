import {
  Memento,
  TextDocument,
  Position,
  CancellationToken,
  CompletionList,
  Hover,
  SignatureHelp,
  SemanticTokens,
  DocumentSymbol,
  Definition,
  LocationLink,
  Location,
  ReferenceContext,
  WorkspaceEdit,
  Range,
  CallHierarchyItem,
  CallHierarchyIncomingCall,
  CallHierarchyOutgoingCall,
} from "vscode";
import { ItemsRepository } from "./spItemsRepository";
import { JsDocCompletionProvider } from "../Providers/spDocCompletions";
import { definitionsProvider } from "../Providers/spDefinitionProvider";
import { signatureProvider } from "../Providers/spSignatureProvider";
import { hoverProvider } from "../Providers/spHoverProvider";
import { symbolProvider } from "../Providers/spSymbolProvider";
import { completionProvider } from "../Providers/spCompletionProvider";
import { semanticTokenProvider } from "../Providers/spSemanticTokenProvider";
import { referencesProvider } from "../Providers/spReferencesProvider";
import { renameProvider } from "../Providers/spRenameProvider";
import { getItemFromPosition } from "./spItemsGetters";
import {
  prepareCallHierarchy,
  provideIncomingCalls,
  provideOutgoingCalls,
} from "../Providers/spCallHierarchy";

export class Providers {
  documentationProvider: JsDocCompletionProvider;
  itemsRepository: ItemsRepository;

  constructor(globalState?: Memento) {
    this.documentationProvider = new JsDocCompletionProvider();
    this.itemsRepository = new ItemsRepository(globalState);
  }

  public async provideCompletionItems(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<CompletionList> {
    return completionProvider(this.itemsRepository, document, position, token);
  }

  public async provideHover(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<Hover> {
    return hoverProvider(this.itemsRepository, document, position, token);
  }

  public async provideSignatureHelp(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<SignatureHelp> {
    return signatureProvider(this.itemsRepository, document, position, token);
  }

  public async provideDefinition(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<Definition | LocationLink[]> {
    return definitionsProvider(this.itemsRepository, document, position, token);
  }

  public async provideDocumentSemanticTokens(
    document: TextDocument
  ): Promise<SemanticTokens> {
    return semanticTokenProvider(this.itemsRepository, document);
  }

  public async provideDocumentSymbols(
    document: TextDocument,
    token: CancellationToken
  ): Promise<DocumentSymbol[]> {
    return symbolProvider(this.itemsRepository, document, token);
  }

  public async provideReferences(
    document: TextDocument,
    position: Position,
    context: ReferenceContext,
    token: CancellationToken
  ): Promise<Location[]> {
    return referencesProvider(
      this.itemsRepository,
      position,
      document,
      context,
      token
    );
  }

  public async prepareRename(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<Range> {
    let items = getItemFromPosition(this.itemsRepository, document, position);
    if (items.length > 0) {
      return items[0].range;
    }
    throw "This symbol cannot be renamed.";
  }

  public async provideRenameEdits(
    document: TextDocument,
    position: Position,
    newName: string,
    token: CancellationToken
  ): Promise<WorkspaceEdit> {
    return renameProvider(
      this.itemsRepository,
      position,
      document,
      newName,
      token
    );
  }

  public async provideCallHierarchyIncomingCalls(
    item: CallHierarchyItem,
    token: CancellationToken
  ): Promise<CallHierarchyIncomingCall[]> {
    return provideIncomingCalls(item, token, this.itemsRepository);
  }

  public async provideCallHierarchyOutgoingCalls(
    item: CallHierarchyItem,
    token: CancellationToken
  ): Promise<CallHierarchyOutgoingCall[]> {
    return provideOutgoingCalls(item, token, this.itemsRepository);
  }

  public async prepareCallHierarchy(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): Promise<CallHierarchyItem | CallHierarchyItem[]> {
    return prepareCallHierarchy(document, position, token);
  }
}
