import {
  window,
  QuickPickOptions,
  QuickPickItem,
} from "vscode";
import { Section, editConfig, getConfig } from "../configUtils";

type AvailableAPIOptional = {
  name: string | undefined;
  includeDirectories: string[] | undefined;
  compilerPath: string | undefined;
  outputDirectoryPath: string | undefined;
  compilerArguments: string[] | undefined;
};

type AvailableAPI = {
  name: string;
  includeDirectories: string[];
  compilerPath: string;
  outputDirectoryPath: string;
  compilerArguments: string[];
};

export async function run(args: any): Promise<void> {
  const availableAPIs: AvailableAPIOptional[] = getConfig(Section.SourcePawn, "availableAPIs")
    .map(buildAvailableAPIFromOptional);

  const validApis =
    availableAPIs.length !== 0 &&
    availableAPIs.filter(api => api.name && api.name !== "").length !== 0;

  if (!validApis) {
    window.showInformationMessage(
      "API list is empty or contains invalid entries! They must have a 'name' property.",
      "Open Settings"
    )
      .then((choice) => {
        if (choice === "Open Settings") {
          editConfig(Section.SourcePawn, "availableAPIs");
        }
      })
    return;
  }

  const apiNames = availableAPIs.map(api => api.name);
  const repeatedNames = apiNames.some((api, index) => apiNames.indexOf(api) !== index);

  if (repeatedNames) {
    window.showErrorMessage(
      "API list has elements with duplicate names!",
      "Open Settings"
    )
      .then(choice => {
        if (choice === "Open Settings") {
          editConfig(Section.SourcePawn, "availableAPIs")
        }
      });
    return;
  }

  const quickPickItems: QuickPickItem[] = availableAPIs.map(api => {
    return {
      label: api.name
    }
  });

  const quickPickOptions: QuickPickOptions = {
    canPickMany: false,
  };

  window.showQuickPick(quickPickItems, quickPickOptions).then(async (item) => {
    const chosenAPI = availableAPIs.find((e) => e.name === item.label);
    await getConfig(Section.LSP).update(
      "includeDirectories",
      chosenAPI.includeDirectories
    );
    await getConfig(Section.LSP).update(
      "compiler.path",
      chosenAPI.compilerPath
    );
    await getConfig(Section.LSP).update(
      "compiler.arguments",
      chosenAPI.compilerArguments
    );
    await getConfig(Section.SourcePawn).update(
      "outputDirectoryPath",
      chosenAPI.outputDirectoryPath
    );
    window.showInformationMessage(`Changed to API ${chosenAPI.name}!`)
  });

  return;
}

function buildAvailableAPIFromOptional(
  optional: AvailableAPIOptional
): AvailableAPI {
  const name = "name" in optional ? optional.name : "";
  const compilerPath = "compilerPath" in optional ? optional.compilerPath : "";
  const outputDirectoryPath =
    "outputDirectoryPath" in optional ? optional.outputDirectoryPath : ""
  const includeDirectories =
    "includeDirectories" in optional ? optional.includeDirectories : [];
  const compilerArguments =
    "compilerArguments" in optional ? optional.compilerArguments : [];

  return {
    name,
    compilerPath,
    outputDirectoryPath,
    includeDirectories,
    compilerArguments
  };
}
