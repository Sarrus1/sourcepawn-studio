import { window, commands, workspace as Workspace } from "vscode";
import * as glob from "glob";
import { join } from "path";
import { URI } from "vscode-uri";
import { ItemsRepository } from "../Backend/spItemsRepository";
import { newDocumentCallback } from "../Backend/spFileHandlers";

/**
 * Parse all the files in the Sourcemod API folder defined by the SourcemodHome setting.
 * @param  {ItemsRepository} itemsRepo    The itemsRepository object constructed in the activation event.
 * @returns void
 */
export function parseSMApi(itemsRepo: ItemsRepository): Promise<void> {
  return new Promise((resolve, reject) => {
    const SMHome: string =
      Workspace.getConfiguration("sourcepawn").get("SourcemodHome") || "";
    const debug = ["messages", "verbose"].includes(
      Workspace.getConfiguration("sourcepawn").get("trace.server")
    );

    if (SMHome === "") {
      window
        .showWarningMessage(
          "SourceMod API not found in the project. You should set SourceMod Home for tasks generation to work. Do you want to install it automatically?",
          "Yes",
          "No, open Settings"
        )
        .then((choice) => {
          if (choice == "Yes") {
            commands.executeCommand("sourcepawn-vscode.installSM");
          } else if (choice === "No, open Settings") {
            commands.executeCommand(
              "workbench.action.openSettings",
              "@ext:sarrus.sourcepawn-vscode"
            );
          }
        });
      reject();
      return;
    }

    if (debug) console.log("Parsing SM API");

    glob(join(SMHome, "**/*.inc"), async (err, files: string[]) => {
      files.forEach((e) =>
        itemsRepo.documents.set(URI.file(e).toString(), false)
      );
      // files.forEach((e) => newDocumentCallback(itemsRepo, URI.file(e)));
      resolve();
    });
  });
}
