import {
  workspace as Workspace,
  window,
  QuickPickOptions,
  QuickPickItem,
} from "vscode";

type AvailableAPIOptional = {
  name: string | undefined;
  includeDirectories: string[] | undefined;
  spcompPath: string | undefined;
  compilerArguments: string[] | undefined;
  linterArguments: string[] | undefined;
};

type AvailableAPI = {
  name: string;
  includeDirectories: string[];
  spcompPath: string;
  compilerArguments: string[];
  linterArguments: string[];
};

export async function run(args: any) {
  const availableAPIs = Workspace.getConfiguration("sourcepawn")
    .get<AvailableAPIOptional[]>("availableAPIs")
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
    await Workspace.getConfiguration("SourcePawnLanguageServer").update(
      "includeDirectories",
      chosenAPI.includeDirectories
    );
    await Workspace.getConfiguration("SourcePawnLanguageServer").update(
      "spcompPath",
      chosenAPI.spcompPath
    );
    await Workspace.getConfiguration("sourcepawn").update(
      "compilerArguments",
      chosenAPI.compilerArguments
    );
    await Workspace.getConfiguration("SourcePawnLanguageServer").update(
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
  const spcompPath = "spcompPath" in optional ? optional.spcompPath : "";
  const includeDirectories =
    "includeDirectories" in optional ? optional.includeDirectories : [];
  const compilerArguments =
    "compilerArguments" in optional ? optional.compilerArguments : [];
  const linterArguments =
    "linterArguments" in optional ? optional.linterArguments : [];

  return {
    name,
    spcompPath,
    includeDirectories,
    compilerArguments,
    linterArguments,
  };
}
