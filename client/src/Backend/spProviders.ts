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
} from "vscode";
import { ItemsRepository } from "./spItemsRepository";
import { JsDocCompletionProvider } from "../Providers/spDocCompletions";
import { definitionsProvider } from "../Providers/spDefinitionProvider";
import { signatureProvider } from "../Providers/spSignatureProvider";
import { hoverProvider } from "../Providers/spHoverProvider";
import { symbolProvider } from "../Providers/spSymbolProvider";
import { completionProvider } from "../Providers/spCompletionProvider";
import { semanticTokenProvider } from "../Providers/spSemanticTokenProvider";

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
}
