import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Insurgency: Source Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const insurgencyEvents: CompletionItem[] = [
	{
		label: "player_team",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_squad",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_spawn",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_hurt",
		kind: kind,
		detail: detail,
	},
	{
		label: "squad_order",
		kind: kind,
		detail: detail,
	},
	{
		label: "game_newmap",
		kind: kind,
		detail: detail,
	},
	{
		label: "game_squadupdate",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_end",
		kind: kind,
		detail: detail,
	},
	{
		label: "gravity_change",
		kind: kind,
		detail: detail,
	}
]