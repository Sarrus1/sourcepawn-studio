import * as lc from "vscode-languageclient/node";
import * as vscode from "vscode";
import * as sp from "../src/lsp_ext";
import { randomUUID } from "crypto";

export interface Env {
  [name: string]: string;
}

// Command URIs have a form of command:command-name?arguments, where
// arguments is a percent-encoded array of data we want to pass along to
// the command function. For "Show References" this is a list of all file
// URIs with locations of every reference, and it can get quite long.
//
// To work around it we use an intermediary linkToCommand command. When
// we render a command link, a reference to a command with all its arguments
// is stored in a map, and instead a linkToCommand link is rendered
// with the key to that map.
export const LINKED_COMMANDS = new Map<string, sp.CommandLink>();

// For now the map is cleaned up periodically (I've set it to every
// 10 minutes). In general case we'll probably need to introduce TTLs or
// flags to denote ephemeral links (like these in hover popups) and
// persistent links and clean those separately. But for now simply keeping
// the last few links in the map should be good enough. Likewise, we could
// add code to remove a target command from the map after the link is
// clicked, but assuming most links in hover sheets won't be clicked anyway
// this code won't change the overall memory use much.
setInterval(function cleanupOlderCommandLinks() {
  // keys are returned in insertion order, we'll keep a few
  // of recent keys available, and clean the rest
  const keys = [...LINKED_COMMANDS.keys()];
  const keysToRemove = keys.slice(0, keys.length - 10);
  for (const key of keysToRemove) {
    LINKED_COMMANDS.delete(key);
  }
}, 10 * 60 * 1000);

function renderCommand(cmd: sp.CommandLink): string {
  const commandId = randomUUID();
  LINKED_COMMANDS.set(commandId, cmd);
  return `[${
    cmd.title
  }](command:sourcepawn-vscode.linkToCommand?${encodeURIComponent(
    JSON.stringify([commandId])
  )} '${cmd.tooltip}')`;
}

function renderHoverActions(
  actions: sp.CommandLinkGroup[]
): vscode.MarkdownString {
  const text = actions
    .map(
      (group) =>
        (group.title ? group.title + " " : "") +
        group.commands.map(renderCommand).join(" | ")
    )
    .join("___");

  const result = new vscode.MarkdownString(text);
  result.isTrusted = true;
  return result;
}

export async function createClient(
  traceOutputChannel: vscode.OutputChannel,
  outputChannel: vscode.OutputChannel,
  //   initializationOptions: vscode.WorkspaceConfiguration,
  serverOptions: lc.ServerOptions,
  clientOptions: lc.LanguageClientOptions
): Promise<lc.LanguageClient> {
  //   clientOptions.initializationOptions = initializationOptions;
  clientOptions.diagnosticCollectionName = "sourcepawn";
  clientOptions.traceOutputChannel = traceOutputChannel;
  clientOptions.outputChannel = outputChannel;
  clientOptions.middleware = {
    workspace: {
      // HACK: This is a workaround, when the client has been disposed, VSCode
      // continues to emit events to the client and the default one for this event
      // attempt to restart the client for no reason
      async didChangeWatchedFile(event, next) {
        if (client.isRunning()) {
          await next(event);
        }
      },
    },
    async provideHover(
      document: vscode.TextDocument,
      position: vscode.Position,
      token: vscode.CancellationToken,
      _next: lc.ProvideHoverSignature
    ) {
      return client
        .sendRequest(
          sp.hover,
          {
            textDocument:
              client.code2ProtocolConverter.asTextDocumentIdentifier(document),
            position: client.code2ProtocolConverter.asPosition(position),
          },
          token
        )
        .then(
          (result) => {
            if (!result) return null;
            const hover = client.protocol2CodeConverter.asHover(result);
            if (!!result.actions) {
              hover.contents.push(renderHoverActions(result.actions));
            }
            console.log(hover);
            return hover;
          },
          (error) => {
            client.handleFailedRequest(
              lc.HoverRequest.type,
              token,
              error,
              null
            );
            return Promise.resolve(null);
          }
        );
    },
  };
  clientOptions.markdown = {
    supportHtml: true,
  };

  const client = new lc.LanguageClient(
    "sourcepawn-vscode",
    "SourcePawn Language Server",
    serverOptions,
    clientOptions
  );

  // To turn on all proposed features use: client.registerProposedFeatures();
  client.registerFeature(new ExperimentalFeatures());
  client.registerFeature(new OverrideFeatures());

  return client;
}

class ExperimentalFeatures implements lc.StaticFeature {
  getState(): lc.FeatureState {
    return { kind: "static" };
  }
  fillClientCapabilities(capabilities: lc.ClientCapabilities): void {
    capabilities.experimental = {
      snippetTextEdit: true,
      codeActionGroup: true,
      hoverActions: true,
      serverStatusNotification: true,
      colorDiagnosticOutput: true,
      openServerLogs: true,
      localDocs: true,
      commands: {
        commands: ["sourcepawn-vscode.gotoLocation"],
      },
      ...capabilities.experimental,
    };
  }
  initialize(
    _capabilities: lc.ServerCapabilities,
    _documentSelector: lc.DocumentSelector | undefined
  ): void {}
  dispose(): void {}
}

class OverrideFeatures implements lc.StaticFeature {
  getState(): lc.FeatureState {
    return { kind: "static" };
  }
  fillClientCapabilities(capabilities: lc.ClientCapabilities): void {
    // Force disable `augmentsSyntaxTokens`, VSCode's textmate grammar is somewhat incomplete
    // making the experience generally worse
    const caps = capabilities.textDocument?.semanticTokens;
    if (caps) {
      caps.augmentsSyntaxTokens = false;
    }
  }
  initialize(
    _capabilities: lc.ServerCapabilities,
    _documentSelector: lc.DocumentSelector | undefined
  ): void {}
  dispose(): void {}
}
