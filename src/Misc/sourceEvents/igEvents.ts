import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Iron Grip: Source Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const igEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	}
]