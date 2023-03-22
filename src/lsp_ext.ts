/**
 * This file mirrors `srcs/lsp_ext.rs` declarations.
 */

import * as lc from "vscode-languageclient";

export const serverStatus = new lc.NotificationType<ServerStatusParams>(
  "experimental/serverStatus"
);
export type ServerStatusParams = {
  health: "ok" | "warning" | "error";
  quiescent: boolean;
  message?: string;
};

export const spcompStatus = new lc.NotificationType<ServerStatusParams>(
  "experimental/spcompStatus"
);
export type SpcompStatusParams = {
  quiescent: boolean;
};
