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
import { createWriteStream, existsSync, mkdirSync, rmSync } from "fs";
import axios from "axios";
import decompress from "decompress";
import { getConfig, Section } from "../configUtils";

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
  const spCompPath: string = getConfig(Section.LSP, "spcompPath") || "";
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
  const includeDirectories: string[] = getConfig(Section.LSP, "includeDirectories");
  includeDirectories.push(smDir);
  getConfig(Section.LSP).update(
    "includeDirectories",
    Array.from(new Set(includeDirectories)), // avoid duplicates
    true
  );
  getConfig(Section.LSP).update(
    "spcompPath",
    spComp,
    true
  );
}

async function getSourceModVersion(
  progress: Progress<{ message?: string; increment?: number }>,
  token: CancellationToken
): Promise<void> {
  let oldStatus = 0;
  const value = await window.showQuickPick(buildQuickPickSMVersion(), {
    title: "Pick a version of Sourcemod to install",
  });
  await downloadAndDecompressFile(
    await getSourcemodUrl(value.label),
    join(outputDir, "sm.gz"),
    (newStatus: number) => {
      if (newStatus === 100) {
        progress.report({ message: "Unzipping..." });
        return;
      }
      let inc = newStatus - oldStatus;
      oldStatus = newStatus;
      progress.report({
        message: "Downloading...",
        increment: inc,
      });
    }
  );
  return;
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

async function getSourcemodUrl(smVersion: string) {
  let url = `https://sm.alliedmods.net/smdrop/${smVersion}/sourcemod-latest-`;
  switch (platform()) {
    case "win32":
      url += "windows";
      break;
    case "darwin":
      url += "mac";
      break;
    default:
      url += "linux";
      break;
  }
  const res = await axios.get(url);
  return `https://sm.alliedmods.net/smdrop/${smVersion}/${res.data}`;
}

async function downloadAndDecompressFile(
  url: string,
  outputFilePath: string,
  progressCallback: (progress: number) => void
) {
  try {
    const { data, headers } = await axios({
      url,
      method: "GET",
      responseType: "stream",
    });

    return new Promise<void>((resolve, reject) => {
      const writer = createWriteStream(outputFilePath);

      if (progressCallback) {
        // Get the content length from the response headers for progress reporting
        const totalBytes = parseInt(headers["content-length"], 10);
        let downloadedBytes = 0;

        // Register the download progress event
        data.on("data", (chunk) => {
          downloadedBytes += chunk.length;
          const progress = (downloadedBytes / totalBytes) * 100;
          progressCallback(progress);
        });
      }
      data.pipe(writer);

      writer.on("finish", () => {
        if (progressCallback) {
          // Ensure the progress reaches 100% after download completion
          progressCallback(100.0);
          console.log("File downloaded.");
        }
        decompress(outputFilePath, outputDir).then(() => {
          console.log("File decompressed.");
          rmSync(outputFilePath, { force: true, recursive: true });
          console.log("Temporary files deleted.");
          resolve();
        });
      });
    });
  } catch (error) {
    console.error("Error during download and decompression:", error.message);
  }
}
