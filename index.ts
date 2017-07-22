import * as path from 'path';

import { ExtensionContext, Disposable, workspace } from 'vscode';
import { LanguageClient, LanguageClientOptions, SettingMonitor, ServerOptions, TransportKind } from 'vscode-languageclient';

export function activate(context: ExtensionContext) {
    let serverModule = context.asAbsolutePath(path.join("out/server/server.js"));
    let debugOptions = { execArgv: ["--nolazy", "--debug=6009"] };

    let serverOptions: ServerOptions = {
        run: { module: serverModule, transport: TransportKind.ipc },
        debug: { module: serverModule, transport: TransportKind.ipc, options: debugOptions }
    };

    let clientOptions: LanguageClientOptions = {
        documentSelector: ['sourcepawn'],
        synchronize: {
            configurationSection: 'sourcepawnLanguageServer',
            fileEvents: workspace.createFileSystemWatcher('**/*.sp')
        }
    };

    let disposable = new LanguageClient('sourcepawnLanguageServer', serverOptions, clientOptions).start();

    context.subscriptions.push(disposable);
}