import {
  workspace as Workspace,
  window,
  QuickPickOptions,
  QuickPickItem,
  commands,
} from "vscode";

interface OptionalSMHome {
  name: string;
  path: string;
}

export async function run(args: any) {
  const optionalSMHomes: OptionalSMHome[] = Workspace.getConfiguration(
    "sourcepawn"
  ).get("optionalSMHomes");
  let newSMHomeChoices: QuickPickItem[] = [];
  for (let optionalHome of optionalSMHomes) {
    newSMHomeChoices.push({
      label: optionalHome.name,
      detail: optionalHome.path,
    });
  }

  const QuickPickOptions: QuickPickOptions = {
    canPickMany: false,
  };
  window.showQuickPick(newSMHomeChoices, QuickPickOptions).then((newSMHome) => {
    if (newSMHome.detail == undefined) {
      return;
    }
    Workspace.getConfiguration("sourcepawn").update(
      "SourcemodHome",
      newSMHome.detail
    );
    commands.executeCommand("workbench.action.reloadWindow");
  });
  return 0;
}
