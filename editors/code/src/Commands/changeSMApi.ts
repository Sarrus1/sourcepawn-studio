import {
  workspace as Workspace,
  window,
  QuickPickOptions,
  QuickPickItem,
} from "vscode";
import { Section, getConfig } from "../configUtils";

type AvailableAPIOptional = {
  name: string | undefined;
  includeDirectories: string[] | undefined;
  compilerPath: string | undefined;
  compilerArguments: string[] | undefined;
  linterArguments: string[] | undefined;
};

type AvailableAPI = {
  name: string;
  includeDirectories: string[];
  compilerPath: string;
  compilerArguments: string[];
  linterArguments: string[];
};

export async function run(args: any) {
  const availableAPIs: AvailableAPIOptional[] =
    getConfig(Section.SourcePawn, "availableAPIs")
      .map(buildAvailableAPIFromOptional);

  const quickPickItems: QuickPickItem[] = availableAPIs.map((availableAPI) => {
    return {
      label: availableAPI.name,
    };
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
    await getConfig(Section.SourcePawn).update(
      "compilerArguments",
      chosenAPI.compilerArguments
    );
    await getConfig(Section.LSP).update(
      "linterArguments",
      chosenAPI.linterArguments
    );
  });

  return 0;
}

function buildAvailableAPIFromOptional(
  optional: AvailableAPIOptional
): AvailableAPI {
  const name = "name" in optional ? optional.name : "";
  const compilerPath = "compilerPath" in optional ? optional.compilerPath : "";
  const includeDirectories =
    "includeDirectories" in optional ? optional.includeDirectories : [];
  const compilerArguments =
    "compilerArguments" in optional ? optional.compilerArguments : [];
  const linterArguments =
    "linterArguments" in optional ? optional.linterArguments : [];

  return {
    name,
    compilerPath,
    includeDirectories,
    compilerArguments,
    linterArguments,
  };
}
