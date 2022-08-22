import {
  workspace as Workspace,
  window,
  ProgressLocation,
  CancellationToken,
  Progress,
} from "vscode";
import { join } from "path";
import { platform, homedir } from "os";
import { existsSync, mkdirSync } from "fs";
const wget = require("wget-improved");
const decompress = require("decompress");

const outputDir = join(homedir(), "sourcemodAPI/");

const Platform = platform();

export async function run(args: any) {
  if (!existsSync(outputDir)) {
    mkdirSync(outputDir);
  }
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
  const spCompPath =
    Workspace.getConfiguration("sourcepawn").get<string>("SpcompPath") || "";
  const smHome =
    Workspace.getConfiguration("sourcepawn").get<string>("SourcemodHome") || "";
  const smDir = join(outputDir, "addons/sourcemod/scripting/include");
  let spComp: string;
  if (Platform === "win32") {
    spComp = join(outputDir, "addons/sourcemod/scripting/spcomp.exe");
  } else {
    spComp = join(outputDir, "addons/sourcemod/scripting/spcomp");
  }
  if (spCompPath != "" || smHome != "") {
    window
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

function updatePath(smDir: string, spComp: string): void {
  Workspace.getConfiguration("sourcepawn").update("SourcemodHome", smDir, true);
  Workspace.getConfiguration("sourcepawn").update("SpcompPath", spComp, true);
}

async function downloadSM(
  progress: Progress<{ message?: string; increment?: number }>,
  token: CancellationToken
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const options = {
      protocol: 'https',
      host: 'sm.alliedmods.net',
      path: '',
      method: 'GET'
    };

    if (Platform === "win32") {
      options.path = "/smdrop/1.11/sourcemod-latest-windows";
    } else if (Platform === "darwin") {
      options.path = "/smdrop/1.11/sourcemod-latest-mac";
    } else {
      options.path = "/smdrop/1.11/sourcemod-latest-linux";
    }

    let request = wget.request(options, function(response) {
      let oldStatus: number = 0;
      let content = '';
      if (response.statusCode === 200) {
        response.on('error', function(err) {
          console.log(err);
        });
        response.on('data', function(chunk) {
          content += chunk;
        });
        response.on('end', function() {
          progress.report({ message: "Downloading SourceMod: " + content });
          console.log(content);

          const output = join(outputDir, "sm.gz");

          const download = wget.download("https://sm.alliedmods.net/smdrop/1.11/" + content, output, options);
          download.on("error", function (err) {
            console.error(err);
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
              const inc = status - oldStatus;
              oldStatus = status;
              progress.report({
                message: "Downloading Sourcemod " + content,
                increment: inc,
              });
            }
          });
          token.onCancellationRequested(() => {
            console.log("SourceMod download was cancelled by the user.");
            //TODO: Actually stop the download here. Might need a better NPM package...
          });
        });
      } else {
        console.log('Response: ' + response.statusCode);
      }
    });

    request.end();
    request.on('error', function(err) {
      console.log(err);
    });
  });
}
