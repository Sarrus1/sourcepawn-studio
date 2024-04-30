import path from "path";
import { run as runServerCommands } from "./runServerCommands";
import { getMainCompilationFile } from "../spUtils";
import { ProgressLocation, WorkspaceFolder, commands, window, workspace } from "vscode";
import { lastActiveEditor } from "../spIndex";
import { URI } from "vscode-uri";
import { Section, getConfig } from "../configUtils";
import sftp from 'ssh2-sftp-client'
import { glob } from "glob";
import { Client } from 'basic-ftp'
import * as fs from 'fs';

export interface UploadOptions {
  sftp: boolean;
  username: string;
  password: string;
  host: string;
  port: number;
  remoteRoot: string;
  exclude: string[];
}

export async function run(args?: string) {
  let workspaceFolder: WorkspaceFolder;
  let fileToUpload: string;

  // If we receive arguments, the file to upload has already been figured out for us,
  // else, we use the user's choice, main compilation file or current editor
  if (!args) {
    workspaceFolder = workspace.getWorkspaceFolder(lastActiveEditor.document.uri);
    const compileMainPath: boolean = getConfig(Section.SourcePawn, "MainPathCompilation", workspaceFolder);
    if (compileMainPath) {
      fileToUpload = await getMainCompilationFile();
    }
    else {
      fileToUpload = lastActiveEditor.document.uri.fsPath
    }
  }
  else {
    fileToUpload = args;
    workspaceFolder = workspace.getWorkspaceFolder(URI.file(fileToUpload));
  }

  // Return if upload settings are not defined
  const uploadOptions: UploadOptions = getConfig(Section.SourcePawn, "UploadOptions", workspaceFolder)
  if (uploadOptions === undefined) {
    window.showErrorMessage("Upload settings are empty.", "Open Settings")
      .then((choice) => {
        if (choice === "Open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return 1;
  }

  // Return if upload settings are not properly configured
  if (uploadOptions.username == "" || uploadOptions.host == "") {
    window.showErrorMessage("Cannot upload - user or host empty.", "Open Settings")
      .then((choice) => {
        if (choice === "Open Settings") {
          commands.executeCommand(
            "workbench.action.openSettings",
            "@ext:sarrus.sourcepawn-vscode"
          );
        }
      });
    return 2;
  }

  const workspaceRoot = workspaceFolder.uri.fsPath;

  // Define the filter function to exclude user-defined files and directories
  const filter = (itemPath: string): boolean => {
    const relativePath = path.relative(workspaceRoot, itemPath);
    for (const exclusion of uploadOptions.exclude) {
      const globPattern = exclusion.endsWith('/') ? `${exclusion}**` : exclusion;
      const matches = glob.sync(globPattern, { cwd: workspaceRoot, dot: true });
      if (matches.includes(relativePath)) {
        return false;
      }
    }

    return true;
  };

  // Begin progress notification
  await window.withProgress(
    {
      location: ProgressLocation.Notification,
      cancellable: true,
      title: "Uploading files..."
    },
    async (progress, token) => {
      const client: sftp = new sftp();

      // Handle cancellation
      token.onCancellationRequested(() => {
        window.showErrorMessage('The upload operation was cancelled.');
        client.end();
        return;
      });

      try {
        // If sftp, use ssh2-sftp
        if (uploadOptions.sftp) {

          // Connect
          await client.connect(uploadOptions);

          // Get currently uploading file name to display it
          client.on('upload', (info) => {
            const fileName = path.basename(info.source);
            progress.report({ message: ` ${fileName}` });
          });

          // Create promise
          await client.uploadDir(workspaceRoot, "/tf/dddd", { filter });

          // Show success message
          window.showInformationMessage('Files uploaded successfully!');
        }

        // else, use FTP-Deploy
        else {
          const ftp = new Client();

          // Access the server
          await ftp.access({
            host: uploadOptions.host,
            port: uploadOptions.port,
            user: uploadOptions.username,
            password: uploadOptions.password,
          });

          // We have to manually build the filters...
          const uploadFiles = async (dirPath: string) => {
            const files = await fs.promises.readdir(dirPath);
            for (const file of files) {
              const filePath = path.join(dirPath, file);
              const stat = await fs.promises.stat(filePath);

              if (stat.isDirectory()) {
                if (filter(filePath)) {
                  await ftp.ensureDir(path.join(uploadOptions.remoteRoot, path.relative(workspaceRoot, filePath)));
                  await uploadFiles(filePath);
                }
              }
              else if (stat.isFile()) {
                if (filter(filePath)) {
                  const remoteFilePath = path.join(uploadOptions.remoteRoot, path.relative(workspaceRoot, filePath));
                  progress.report({ message: ` ${file}` });
                  await ftp.uploadFrom(filePath, remoteFilePath);
                }
              }
            }
          };

          await uploadFiles(workspaceRoot);
          window.showInformationMessage('Files uploaded successfully!');
          ftp.close();

        }
      } catch (error) {
        if (token.isCancellationRequested) {
          return 0;
        }
        window.showErrorMessage('Failed to upload files! ' + error);
        client.end();
        return 1;
      } finally {
        client.end();
      }

      return 0;
    }
  );

  if (getConfig(Section.SourcePawn, "runServerCommands", workspaceFolder) === "afterUpload") {
    await runServerCommands(fileToUpload);
  }

  return 0;
}
