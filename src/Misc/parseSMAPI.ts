import { window, commands, workspace as Workspace } from "vscode";
import * as glob from "glob";
import { FileItems } from "../Backend/spFilesRepository";
import { parseFile } from "../Parser/spParser";
import { join } from "path";
import { URI } from "vscode-uri";
import { ItemsRepository } from "../Backend/spItemsRepository";

/**
 * Parse all the files in the Sourcemod API folder defined by the SourcemodHome setting.
 * @param  {ItemsRepository} itemsRepo    The itemsRepository object constructed in the activation event.
 * @returns void
 */
export function parseSMApi(itemsRepo: ItemsRepository): void {
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
    return;
  }

  if (debug) console.log("Parsing SM API");

  const files: string[] = glob.sync(join(SMHome, "**/*.inc"));
  files.forEach((e) => itemsRepo.documents.add(URI.file(e).toString()));

  for (let file of files) {
    try {
      if (debug) console.log("SM API Reading", file);

      let items = new FileItems(URI.file(file).toString());
      parseFile(file, items, itemsRepo, false, true);

      if (debug) console.log("SM API Done parsing", file);

      let uri = URI.file(file).toString();
      itemsRepo.fileItems.set(uri, items);
      itemsRepo.documents.add(uri);

      if (debug) console.log("SM API Done dealing with", uri);
      parseFile(file, items, itemsRepo, true, true);
    } catch (e) {
      console.error(e);
    }
  }
  if (debug) console.log("Done parsing SM API");
}
