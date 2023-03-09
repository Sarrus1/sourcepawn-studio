import {
  workspace as Workspace,
  window,
  ProgressLocation,
  CancellationToken,
  Progress,
  QuickPickItem,
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
      return getSourceModVersion(progress, token);
    }
  );
  const spCompPath =
    Workspace.getConfiguration("SourcePawnLanguageServer").get<string>(
      "spcompPath"
    ) || "";
  const smDir = join(outputDir, "addons/sourcemod/scripting/include");
  let spComp: string;
  if (Platform === "win32") {
    spComp = join(outputDir, "addons/sourcemod/scripting/spcomp.exe");
  } else {
    spComp = join(outputDir, "addons/sourcemod/scripting/spcomp");
  }
  if (spCompPath != "") {
    window
      .showInformationMessage(
        "The setting for spcompPath is not empty, do you want to override it?",
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
  const includeDirectories = Workspace.getConfiguration(
    "SourcePawnLanguageServer"
  ).get<string[]>("includesDirectories");
  includeDirectories.push(smDir);
  Workspace.getConfiguration("SourcePawnLanguageServer").update(
    "includesDirectories",
    Array.from(new Set(includeDirectories)), // avoid duplicates
    true
  );
  Workspace.getConfiguration("SourcePawnLanguageServer").update(
    "spcompPath",
    spComp,
    true
  );
}

async function getSourceModVersion(
  progress: Progress<{ message?: string; increment?: number }>,
  token: CancellationToken
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    window
      .showQuickPick(buildQuickPickSMVersion(), {
        title: "Pick a version of Sourcemod to install",
      })
      .then((value) => {
        resolve(downloadSM(value.label, progress, token));
      });
  });
}

function buildQuickPickSMVersion(): QuickPickItem[] {
  return [
    { label: "1.7", description: "Legacy" },
    { label: "1.8", description: "Legacy" },
    { label: "1.9", description: "Legacy" },
    { label: "1.10", description: "Legacy" },
    { label: "1.11", description: "Stable", picked: true },
    { label: "1.12", description: "Dev" },
  ];
}

async function downloadSM(
  smVersion: string,
  progress: Progress<{ message?: string; increment?: number }>,
  token: CancellationToken
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const options = {
      protocol: "https",
      host: "sm.alliedmods.net",
      path: "",
      method: "GET",
    };

    if (Platform === "win32") {
      options.path = "/smdrop/" + smVersion + "/sourcemod-latest-windows";
    } else if (Platform === "darwin") {
      options.path = "/smdrop/" + smVersion + "/sourcemod-latest-mac";
    } else {
      options.path = "/smdrop/" + smVersion + "/sourcemod-latest-linux";
    }

    let request = wget.request(options, function (response) {
      let oldStatus: number = 0;
      let content = "";
      if (response.statusCode === 200) {
        response.on("error", function (err) {
          console.log(err);
        });
        response.on("data", function (chunk) {
          content += chunk;
        });
        response.on("end", function () {
          progress.report({ message: "Downloading Sourcemod: " + content });
          console.log(content);

          const output = join(outputDir, "sm.gz");

          const download = wget.download(
            "https://sm.alliedmods.net/smdrop/" + smVersion + "/" + content,
            output,
            options
          );
          download.on("error", function (err) {
            console.error(err);
            reject(err);
          });
          download.on("start", function (fileSize: number) {
            console.log(
              "filesize: ",
              Math.ceil(fileSize / Math.pow(10, 6)),
              "Mo"
            );
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
            console.log("Sourcemod download was cancelled by the user.");
            //TODO: Actually stop the download here. Might need a better NPM package...
          });
        });
      } else {
        console.log("Response: " + response.statusCode);
      }
    });

    request.end();
    request.on("error", function (err) {
      console.log(err);
    });
  });
}
