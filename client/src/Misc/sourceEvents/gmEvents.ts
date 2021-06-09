import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Garry's Mod Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const gmEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	}
]