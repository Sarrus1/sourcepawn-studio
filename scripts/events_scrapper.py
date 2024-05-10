import json
from typing import Dict, List, Optional

import requests
from bs4 import BeautifulSoup
from tqdm import tqdm


def main():
    pass


class Game:
    name: str
    url: str
    events: List["Event"]

    def __init__(self, name: str, url: str) -> None:
        self.name = name
        self.url = url
        self.events = []


class Event:
    name: str
    note: Optional[str]
    attributes: List["Attribute"]

    def __init__(
        self,
        name: str,
        note: Optional[str] = None,
        attributes: Optional[List["Attribute"]] = None,
    ) -> None:
        self.name = name
        self.note = note
        self.attributes = attributes if attributes is not None else []


class Attribute:
    name: str
    type: str
    description: Optional[str]

    def __init__(self, name: str, type: str, description: str) -> None:
        self.name = name.strip()
        self.type = type.strip()
        description = description.strip()
        if description == "":
            description = None
        self.description = description


class Scrapper:
    base_url = "https://wiki.alliedmods.net"

    def __init__(self) -> None:
        pass

    def get_games(self) -> Dict[str, Game]:
        out: Dict[str, Game] = {}
        res = requests.get(self.base_url + "/Game_Events_(Source)")
        soup = BeautifulSoup(res.text, "html.parser")

        div = soup.find("ul")
        anchor_tags = div.find_all("a")
        for anchor in tqdm(anchor_tags):
            href = anchor.get("href", None)
            title = anchor.get("title", anchor.text.strip()).removesuffix(" Events")
            game = Game(title, self.base_url + href)
            self.get_events(game)
            out[title] = game

        # dump as json
        with open("data.json", "w") as f:
            json.dump(out, f, default=vars)

    def get_events(self, game: Game) -> None:
        res = requests.get(game.url)
        soup = BeautifulSoup(res.text, "html.parser")

        spans = soup.find_all("span", class_="mw-headline")
        for span in spans:
            name = span.text.strip()
            h3 = span.parent
            note_p = h3.find_next_sibling("p")
            if note_p is None:
                game.events.append(Event(name))
                continue
            note = note_p.text.strip().removeprefix("Note: ").replace("\u00a0", " ")
            table = note_p.find_next_sibling("table")
            if table is None:
                game.events.append(Event(name, note))
                continue
            table = table.find("table")
            if table is None:
                game.events.append(Event(name, note))
                continue
            tbody = table.find("tbody")
            if tbody is None:
                game.events.append(Event(name, note))
                continue
            attrs = []
            trs = tbody.find_all("tr")
            for tr in trs:
                tds = tr.find_all("td")
                if len(tds) != 3:
                    continue
                attrs.append(
                    Attribute(
                        tds[1].text.strip(), tds[0].text.strip(), tds[2].text.strip()
                    )
                )
            game.events.append(Event(name, note, attrs))


if __name__ == "__main__":
    Scrapper().get_games()
