import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Half-Life 2: Deathmatch Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const hl2DmEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "teamplay_round_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "spec_target_updated",
		kind: kind,
		detail: detail,
	},
	{
		label: "achievement_earned",
		kind: kind,
		detail: detail,
	}
]