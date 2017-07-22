import { ExtensionContext, Disposable, workspace, window, StatusBarAlignment } from 'vscode';
import { LanguageClient, LanguageClientOptions, SettingMonitor, ServerOptions, TransportKind } from 'vscode-languageclient';
import * as glob from 'glob';
import * as path from 'path';

export function activate(context: ExtensionContext) {
    let serverModule = context.asAbsolutePath("out/server/server.js");
    let debugOptions = { execArgv: ["--nolazy", "--debug=6009"] };
   
    glob(path.join(workspace.rootPath, "**/include/sourcemod.inc"), (err, files) => {
        if (files.length === 0) {
            if (!workspace.getConfiguration("sourcepawnLanguageServer").get("sourcemod_home")) {
                window.showWarningMessage("SourceMod API not found in the project. You may need to set SourceMod Home for autocompletion to work");
            }
        }
    });

    let serverOptions: ServerOptions = {
        run: { module: serverModule, transport: TransportKind.ipc },
        debug: { module: serverModule, transport: TransportKind.ipc, options: debugOptions }
    };

    let clientOptions: LanguageClientOptions = {
        documentSelector: ['sourcepawn'],
        synchronize: {
            configurationSection: 'sourcepawnLanguageServer',
            fileEvents: [workspace.createFileSystemWatcher('**/*.sp'), workspace.createFileSystemWatcher('**/*.inc')]
        }
    };

    let client = new LanguageClient('sourcepawnLanguageServer', serverOptions, clientOptions);
    let disposable = client.start();

    context.subscriptions.push(disposable);
}