import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Dystopia Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const dystopiaEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "cyber_frag",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_class",
		kind: kind,
		detail: detail,
	},
	{
		label: "objective",
		kind: kind,
		detail: detail,
	},
	{
		label: "round_restart",
		kind: kind,
		detail: detail,
	},
	{
		label: "dys_changemap",
		kind: kind,
		detail: detail,
	},
	{
		label: "dys_points",
		kind: kind,
		detail: detail,
	},
	{
		label: "dys_weapon_stats",
		kind: kind,
		detail: detail,
	},
	{
		label: "dys_implant_stats",
		kind: kind,
		detail: detail,
	},
	{
		label: "dys_scoring_stats",
		kind: kind,
		detail: detail,
	}
]