import json
from typing import Dict, List

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
    note: str

    def __init__(self, name: str) -> None:
        self.name = name

class Scrapper:
    base_url = "https://wiki.alliedmods.net"

    def __init__(self) -> None:
        pass

    def get_games(self) -> Dict[str, Game]:
        out: Dict[str, Game] = {}
        res = requests.get(self.base_url+"/Game_Events_(Source)")
        soup = BeautifulSoup(res.text, 'html.parser')

        div = soup.find("ul")
        anchor_tags = div.find_all('a')
        for anchor in tqdm(anchor_tags):
            href = anchor.get('href', None)
            title = anchor.get('title', anchor.text.strip())
            game = Game(title, self.base_url + href)
            self.get_events(game)
            out[title] = game

        
        # dump as json
        with open('data.json', 'w') as f:
            json.dump(out, f, indent=4, default=vars)

    
    def get_events(self, game: Game) -> None:
        res = requests.get(game.url)
        soup = BeautifulSoup(res.text, 'html.parser')

        spans = soup.find_all("span", class_="toctext")
        for span in spans:
            game.events.append(Event(span.text.strip()))
            # structure = tds[1]
            # sub_tds = structure.find_all('td')
            # for i in range(0, len(sub_tds), 2):
            #     event_name = sub_tds[i].text.strip()
            #     event_note = sub_tds[i+1].text.strip()
            #     game.events.append(Event(event_name, event_note))
        

if __name__ == "__main__":
    Scrapper().get_games()