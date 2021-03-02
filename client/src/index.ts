import { ExtensionContext, workspace as Workspace, window, commands } from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';
import * as glob from 'glob';
import * as path from 'path';


export function activate(context: ExtensionContext) {
    let serverModule = context.asAbsolutePath(
        path.join('server', 'out', 'server.js')
    );
    let debugOptions = { execArgv: ["--nolazy", "--inspect=6009"] };
    glob(path.join(Workspace.workspaceFolders[0].name || "", "scripting/include/sourcemod.inc"), (err, files) => {
        if (files.length === 0) {
            if (!Workspace.getConfiguration("sourcepawnLanguageServer").get("sourcemod_home")) {
                window.showWarningMessage("SourceMod API not found in the project. You may need to set SourceMod Home for autocompletion to work", "Open Settings").then((choice) => {
                    if (choice === 'Open Settings') {
                        commands.executeCommand("workbench.action.openWorkspaceSettings");
                    }
                });
            }
        } else {
            if (!Workspace.getConfiguration("sourcepawnLanguageServer").get("sourcemod_home")) {
                Workspace.getConfiguration("sourcepawnLanguageServer").update("sourcemod_home", path.dirname(files[0]));
            }
        }
    });
    let serverOptions: ServerOptions = {
        run: { module: serverModule, transport: TransportKind.ipc },
        debug: { module: serverModule, transport: TransportKind.ipc, options: debugOptions }
    };

    let clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'sourcepawn' }],
        synchronize: {
            //configurationSection: 'sourcepawnLanguageServer',
            fileEvents: [Workspace.createFileSystemWatcher('**/*.sp'), Workspace.createFileSystemWatcher('**/*.inc')]
        }
    };

    let client = new LanguageClient('sourcepawnLanguageServer', serverOptions, clientOptions);
    let disposable = client.start();

    context.subscriptions.push(disposable);
}