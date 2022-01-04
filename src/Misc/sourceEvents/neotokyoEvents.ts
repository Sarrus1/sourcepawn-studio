import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "NeoTokyo Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const neotokyoEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "game_round_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "game_round_start",
		kind: kind,
		detail: detail,
	}
]