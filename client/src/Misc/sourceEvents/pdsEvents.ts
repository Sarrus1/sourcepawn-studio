import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Perfect Dark: Source Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const pdsEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "koth_hill_activated",
		kind: kind,
		detail: detail,
	},
	{
		label: "koth_hill_deactivated",
		kind: kind,
		detail: detail,
	},
	{
		label: "koth_hill_taken",
		kind: kind,
		detail: detail,
	},
	{
		label: "koth_hill_close_to_capture",
		kind: kind,
		detail: detail,
	},
	{
		label: "koth_hill_captured",
		kind: kind,
		detail: detail,
	},
	{
		label: "koth_hill_lost",
		kind: kind,
		detail: detail,
	},
	{
		label: "popacap_close_to_point",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctb_spawn_activated",
		kind: kind,
		detail: detail,
	},
	{
		label: "ctb_spawn_deactivated",
		kind: kind,
		detail: detail,
	},
	{
		label: "weapon_set",
		kind: kind,
		detail: detail,
	}
]