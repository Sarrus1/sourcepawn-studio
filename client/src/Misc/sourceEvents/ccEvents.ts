import { CompletionItem, CompletionItemKind } from "vscode";

const detail: string = "Codename Cure Event";
const kind: CompletionItemKind = CompletionItemKind.Keyword;

export const ccEvents: CompletionItem[] = [
	{
		label: "player_death",
		kind: kind,
		detail: detail,
	},
	{
		label: "zombie_killed",
		kind: kind,
		detail: detail,
	},
	{
		label: "game_win",
		kind: kind,
		detail: detail,
	},
	{
		label: "game_reset",
		kind: kind,
		detail: detail,
	},
	{
		label: "game_diff_change",
		kind: kind,
		detail: detail,
	},
	{
		label: "special_condition",
		kind: kind,
		detail: detail,
	},
	{
		label: "achievement_earned",
		kind: kind,
		detail: detail,
	},
	{
		label: "achievement_earned_local",
		kind: kind,
		detail: detail,
	}
]