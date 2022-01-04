import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Hidden: Source Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const hsEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_hurt",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_team",
		kind: kind,
		detail: detail,
	},
	{
		label: "alarm_trigger",
		kind: kind,
		detail: detail,
	},
	{
		label: "material_check",
		kind: kind,
		detail: detail,
	},
	{
		label: "extraction_start",
		kind: kind,
		detail: detail,
	},
	{
		label: "extraction_stop",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_location",
		kind: kind,
		detail: detail,
	},
	{
		label: "iris_radio",
		kind: kind,
		detail: detail,
	},
	{
		label: "game_round_restart",
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