import {
  workspace as Workspace,
  window,
  QuickPickOptions,
  QuickPickItem,
  commands,
} from "vscode";

interface OptionalSMAPI {
  name: string;
  SMHomePath: string;
  compilerPath: string;
}

export async function run(args: any) {
  const optionalSMHomes: OptionalSMAPI[] = Workspace.getConfiguration(
    "sourcepawn"
  ).get("availableAPIs");
  const newSMHomeChoices: QuickPickItem[] = optionalSMHomes.map(
    (optionalHome) => {
      return {
        label: optionalHome.name,
        detail: optionalHome.SMHomePath,
      };
    }
  );

  const QuickPickOptions: QuickPickOptions = {
    canPickMany: false,
  };
  window
    .showQuickPick(newSMHomeChoices, QuickPickOptions)
    .then(async (newSMHome) => {
      if (newSMHome.detail == undefined) {
        return;
      }
      await Workspace.getConfiguration("sourcepawn").update(
        "SourcemodHome",
        newSMHome.detail
      );
      const spCompPath = optionalSMHomes.find((e) => e.name === newSMHome.label)
        .compilerPath;
      await Workspace.getConfiguration("sourcepawn").update(
        "SpcompPath",
        spCompPath
      );
      commands.executeCommand("workbench.action.reloadWindow");
    });
  return 0;
}
