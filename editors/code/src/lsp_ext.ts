/**
 * This file mirrors `src/lsp_ext.rs` declarations.
 */

import * as lc from "vscode-languageclient";

export const preprocessedDocument = new lc.RequestType<
  PreprocessedDocumentParams,
  string,
  void
>("sourcepawn-lsp/preprocessedDocument");

export type PreprocessedDocumentParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const syntaxTree = new lc.RequestType<SyntaxTreeParams, string, void>(
  "sourcepawn-lsp/syntaxTree"
);

export type SyntaxTreeParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const analyzerStatus = new lc.RequestType<
  AnalyzerStatusParams,
  string,
  void
>("sourcepawn-lsp/analyzerStatus");

export type AnalyzerStatusParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const itemTree = new lc.RequestType<ItemTreeParams, string, void>(
  "sourcepawn-lsp/itemTree"
);

export type ItemTreeParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const projectMainPath = new lc.RequestType<
  ProjectMainPathParams,
  lc.URI,
  void
>("sourcepawn-lsp/projectMainPath");

export type ProjectMainPathParams = {
  uri?: lc.URI;
};

export const projectsGraphviz = new lc.RequestType<
  ProjectsGraphvizParams,
  string,
  void
>("sourcepawn-lsp/projectsGraphviz");

export type ProjectsGraphvizParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const serverStatus = new lc.NotificationType<ServerStatusParams>(
  "sourcepawn-lsp/serverStatus"
);
export type ServerStatusParams = {
  health: "ok" | "warning" | "error";
  quiescent: boolean;
  message?: string;
};

export const spcompStatus = new lc.NotificationType<ServerStatusParams>(
  "sourcepawn-lsp/spcompStatus"
);
export type SpcompStatusParams = {
  quiescent: boolean;
};
