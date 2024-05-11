import { WorkspaceConfiguration, WorkspaceFolder, commands, workspace as Workspace } from 'vscode'

/**
 * Get a value from the user's extension settings.
 * @param section The section setting to access
 * @param key The setting key
 * @param workspaceFolder The workplace folder to seek the setting value
 * @param def A default value in case the value returns as undefined
 * @returns A value from the user's extension settings
 */
export function getConfig(section: Section, key?: string, workspaceFolder?: WorkspaceFolder, def?: any): any {
    let config: WorkspaceConfiguration;
    if (!key) {
        return Workspace.getConfiguration(section.toString());
    }
    if (workspaceFolder) {
        config = Workspace.getConfiguration(section.toString(), workspaceFolder);
    }
    else {
        config = Workspace.getConfiguration(section.toString());
    }
    return config.get(key, def);
}

/**
 * Opens the settings for the user to modify the provided configuration
 * @param section The section setting to access
 * @param key The setting key
 */
export function editConfig(section: Section, key: string): void {
    commands.executeCommand(
        "workbench.action.openSettings",
        `@id:${section}.${key}`
    );
}

export enum Section {
    SourcePawn = "sourcepawn",
    LSP = "SourcePawnLanguageServer",
    Editor = "editor",
}
