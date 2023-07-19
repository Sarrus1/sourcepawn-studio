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
