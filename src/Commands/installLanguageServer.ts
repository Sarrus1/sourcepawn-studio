import {
  window,
  ProgressLocation,
  CancellationToken,
  Progress,
  extensions,
} from "vscode";
import { join } from "path";
import { arch, platform } from "os";
import {
  createReadStream,
  createWriteStream,
  existsSync,
  mkdirSync,
  rmSync,
} from "fs";
import axios from "axios";
import unzipper from "unzipper";
import { execSync } from "child_process";

import { ctx } from "../spIndex";

export async function run(args: any) {
  await ctx?.stop();

  const lspPath = join(
    extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath,
    "languageServer"
  );
  if (existsSync(lspPath)) {
    rmSync(lspPath, { recursive: true });
  }
  mkdirSync(lspPath);
  const version = await getLatestVersionName();
  if (version === undefined) {
    window.showErrorMessage(
      "Failed to download the latest version of the SourcePawn Language Server."
    );
    return 1;
  }

  await window.withProgress(
    {
      location: ProgressLocation.Notification,
      title: "SourcePawn LanguageServer",
      cancellable: true,
    },
    async (progress, token) => {
      return downloadLanguageServer(version, progress, token);
    }
  );

  await ctx?.start();

  return 0;
}

export async function getLatestVersionName() {
  const res = await axios.get(
    "https://api.github.com/repos/Sarrus1/sourcepawn-lsp/releases/latest"
  );
  if (res.status != 200) {
    return undefined;
  }
  return res.data["name"];
}

function makeLanguageServerURL(version: string) {
  const platform_ = platform();
  const arch_ = arch();
  let platform_id = "";
  let arch_id = "";
  if (platform_ === "win32") {
    platform_id = "windows";
    if (arch_ === "x64") {
      arch_id = "x86_64";
    } else if (arch_ === "x32") {
      arch_id = "i686";
    } else if (arch_ === "arm64") {
      arch_id = "aarch64";
    }
  } else if (platform_ === "darwin") {
    platform_id = "macos";
    if (arch_ === "x64") {
      arch_id = "x86_64";
    } else if (arch_ === "arm64") {
      arch_id = "aarch64";
    }
  } else {
    platform_id = "linux";
    if (arch_ === "x64") {
      arch_id = "x86_64";
    } else if (arch_ === "x32") {
      arch_id = "i686";
    } else if (arch_ === "arm64") {
      arch_id = "aarch64";
    } else {
      arch_id = "armv7hf";
    }
  }

  return `https://github.com/Sarrus1/sourcepawn-lsp/releases/download/${version}/sourcepawn_lsp-${arch_id}-${platform_id}.zip`;
}

async function downloadLanguageServer(
  version: string,
  progress: Progress<{ message?: string; increment?: number }>,
  token: CancellationToken
) {
  try {
    const { data, headers } = await axios({
      url: makeLanguageServerURL(version),
      method: "GET",
      responseType: "stream",
    });

    const lspPath = join(
      extensions.getExtension("Sarrus.sourcepawn-vscode").extensionPath,
      "languageServer"
    );
    const zipPath = join(lspPath, "sourcepawn_lsp.zip");

    return new Promise<void>((resolve, reject) => {
      const totalLength = headers["content-length"];

      const writer = createWriteStream(zipPath);

      // Update the progress bar.
      let oldStatus = 0;
      let newStatus = 0;
      let counter = 0;
      data.on("data", (chunk) => {
        counter += chunk.length;
        oldStatus = newStatus;
        newStatus = Math.floor((counter / totalLength) * 100);
        let inc = newStatus - oldStatus;
        progress.report({
          message: "Downloading...",
          increment: inc,
        });
      });
      data.pipe(writer);

      writer.on("finish", () => {
        // Unzip the file now that it has been downloaded.
        progress.report({
          message: "Unzipping...",
        });
        const artifactName =
          platform() == "win32" ? "sourcepawn_lsp.exe" : "sourcepawn_lsp";
        const outPath = join(lspPath, artifactName);
        createReadStream(zipPath)
          .pipe(unzipper.Parse())
          .on("entry", function (entry) {
            if (entry.path === artifactName) {
              entry.pipe(createWriteStream(outPath));
            } else {
              entry.autodrain();
            }
          })
          .on("close", () => {
            if (platform() != "win32") {
              execSync(`chmod +x ${outPath}`);
            }
            rmSync(zipPath);
            resolve();
          });
      });
    });
  } catch (error) {
    if (
      error["response"] !== undefined &&
      error["response"]["status"] === 404
    ) {
      window.showErrorMessage(
        "No release of sourcepawn_lsp were found for your. Either your OS is not supported or the release is currently being built. Please check the status [here](https://github.com/Sarrus1/sourcepawn-lsp/releases/latest)."
      );
    }
  }
}
