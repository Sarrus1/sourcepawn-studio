/**
 * This file mirrors `src/lsp_ext.rs` declarations.
 */

import * as lc from "vscode-languageclient";

export const preprocessedDocument = new lc.RequestType<
  PreprocessedDocumentParams,
  string,
  void
>("sourcepawn-studio/preprocessedDocument");

export type PreprocessedDocumentParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const syntaxTree = new lc.RequestType<SyntaxTreeParams, string, void>(
  "sourcepawn-studio/syntaxTree"
);

export type SyntaxTreeParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const analyzerStatus = new lc.RequestType<
  AnalyzerStatusParams,
  string,
  void
>("sourcepawn-studio/analyzerStatus");

export type AnalyzerStatusParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const itemTree = new lc.RequestType<ItemTreeParams, string, void>(
  "sourcepawn-studio/itemTree"
);

export type ItemTreeParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const projectMainPath = new lc.RequestType<
  ProjectMainPathParams,
  lc.URI,
  void
>("sourcepawn-studio/projectMainPath");

export type ProjectMainPathParams = {
  uri?: lc.URI;
};

export const projectsGraphviz = new lc.RequestType<
  ProjectsGraphvizParams,
  string,
  void
>("sourcepawn-studio/projectsGraphviz");

export type ProjectsGraphvizParams = {
  textDocument?: lc.TextDocumentIdentifier;
};

export const serverStatus = new lc.NotificationType<ServerStatusParams>(
  "sourcepawn-studio/serverStatus"
);
export type ServerStatusParams = {
  health: "ok" | "warning" | "error";
  quiescent: boolean;
  message?: string;
};

export const spcompStatus = new lc.NotificationType<ServerStatusParams>(
  "sourcepawn-studio/spcompStatus"
);
export type SpcompStatusParams = {
  quiescent: boolean;
};

export const hover = new lc.RequestType<
  lc.HoverParams,
  (lc.Hover & { actions: CommandLinkGroup[] }) | null,
  void
>(lc.HoverRequest.method);

export type CommandLink = {
  /**
   * A tooltip for the command, when represented in the UI.
   */
  tooltip?: string;
} & lc.Command;
export type CommandLinkGroup = {
  title?: string;
  commands: CommandLink[];
};
