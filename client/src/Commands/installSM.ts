import * as vscode from "vscode";
const wget = require("wget-improved");
import { window, ProgressLocation, CancellationToken, Progress } from "vscode";
import { join } from "path";
const decompress = require("decompress");
import { platform, arch } from "os";

const outputDir = join(
  vscode.extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath,
  "misc/"
);
const Platform = platform();

export async function run(args: any) {
  await window.withProgress(
    {
      location: ProgressLocation.Notification,
      title: "Sourcemod Download",
      cancellable: true,
    },
    async (progress, token) => {
      return downloadSM(progress, token);
    }
  );
  let spCompPath =
    vscode.workspace.getConfiguration("sourcepawn").get<string>("SpcompPath") ||
    "";
  let smHome =
    vscode.workspace
      .getConfiguration("sourcepawn")
      .get<string>("SourcemodHome") || "";
  let smDir = join(outputDir, "addons/sourcemod/scripting/include");
  let spComp: string;
  if (Platform === "win32") {
    spComp = join(outputDir, "addons/sourcemod/scripting/spcomp.exe");
  } else {
    spComp = join(outputDir, "addons/sourcemod/scripting/spcomp");
  }
  if (spCompPath != "" || smHome != "") {
    vscode.window
      .showInformationMessage(
        "The setting for SpcompPath or SourcemodHome is not empty, do you want to override them ?",
        "Yes",
        "No"
      )
      .then((choice) => {
        if (choice === "Yes") {
					updatePath(smDir, spComp);
        }
      });
    return 0;
  }
	updatePath(smDir, spComp);
  return 0;
}

function updatePath(smDir: string, spComp: string):void {
	vscode.workspace
	.getConfiguration("sourcepawn")
	.update("SourcemodHome", smDir, true);
	vscode.workspace
	.getConfiguration("sourcepawn")
	.update("SpcompPath", spComp, true);
}

async function downloadSM(
  progress: Progress<{ message?: string; increment?: number }>,
  token: CancellationToken
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    let oldStatus: number = 0;
    var src: string;
    if (Platform === "win32") {
      src = "http://sourcemod.net/latest.php?os=windows&version=1.10";
    } else if (Platform === "darwin") {
      src = "http://sourcemod.net/latest.php?os=mac&version=1.10";
    } else {
      src = "http://sourcemod.net/latest.php?os=linux&version=1.10";
    }
    const output = join(outputDir, "sm.gz");
    const options = {
      gunzip: false,
    };
    if (token.isCancellationRequested) {
      return;
    }
    let download = wget.download(src, output, options);
    download.on("error", function (err) {
      console.log(err);
      reject(err);
    });
    download.on("start", function (fileSize: number) {
      console.log("filesize: ", Math.ceil(fileSize / Math.pow(10, 6)), "Mo");
    });
    download.on("end", async function (endStatus) {
      console.log(endStatus);
      progress.report({ message: "Unzipping..." });
      await decompress(output, outputDir);
      resolve(endStatus);
    });
    download.on("progress", function (status) {
      if (typeof status === "number") {
        status = Math.floor(status * 100);
        let inc = status - oldStatus;
        oldStatus = status;
        progress.report({
          message: "Downloading Sourcemod",
          increment: inc,
        });
      }
    });
    token.onCancellationRequested(() => {
      console.log("Sourcemod download was cancelled by the user.");
      //TODO: Actually stop the download here. Might need a better NPM package...
    });
  });
}
