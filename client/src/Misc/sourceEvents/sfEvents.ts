import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "SourceForts Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const sfEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "block_frozen",
		kind: kind,
		detail: detail,
	},
	{
		label: "block_unfrozen",
		kind: kind,
		detail: detail,
	},
	{
		label: "phase_switch",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_grab",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_drop",
		kind: kind,
		detail: detail,
	},
	{
		label: "player_score",
		kind: kind,
		detail: detail,
	},
	{
		label: "flag_return",
		kind: kind,
		detail: detail,
	}
]