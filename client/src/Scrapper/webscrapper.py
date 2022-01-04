from selenium import webdriver
from selenium.webdriver.common.by import By
import json
import os

BASE_URL = "https://totalcsgo.com"


def main():
    docs: list = []
    for id in range(1, 32):
        if id == 1:
            driver.get(f"{BASE_URL}/commands")
        else:
            driver.get(f"{BASE_URL}/commands/{id}")
        tables = driver.find_elements(By.CSS_SELECTOR, 'table.table-hover')

        for table in tables:
            for row in table.find_elements(By.CSS_SELECTOR, 'tr'):
                i = 0
                for cell in row.find_elements(By.TAG_NAME, 'td'):
                    i += 1
                    if i == 1:
                        name: str = cell.text
                        print(name)
                        try:
                            tag = cell.find_element(By.TAG_NAME, 'a')
                            url: str = tag.get_attribute('href')
                        except:
                            url: str = None
                    elif i == 2:
                        syntax: str = cell.text
                    else:
                        description: str = cell.text
                try:
                    docs.append({"name": name, "url": url.removeprefix(
                        BASE_URL), "syntax": syntax, "description": description, "arguments": []})
                except:
                    continue

    with open('data.json', 'w') as outfile:
        json.dump(docs, outfile)

    with open('data.json', 'r') as infile:
        docs = json.load(infile)

    for doc in docs:
        if doc["url"] == "":
            continue
        try:
            syntax = getSyntax(f'{BASE_URL}{doc["url"]}')
        except:
            continue
        doc["arguments"] = syntax

    with open('data.json', 'w') as outfile:
        json.dump(docs, outfile)


def getSyntax(url: str):
    driver.get(url)
    table = driver.find_element(By.CSS_SELECTOR, 'table')
    arguments: list = []
    for row in table.find_elements(By.CSS_SELECTOR, 'tr'):
        i = 0
        for cell in row.find_elements(By.TAG_NAME, 'td'):
            i += 1
            if i == 1:
                label = cell.find_element(By.TAG_NAME, 'strong').text
            else:
                description = cell.text
            try:
                arguments.append({"label": label, "description": description})
            except:
                continue
    return arguments


if __name__ == "__main__":
    CHROMEDRIVER_PATH = './chromedriver'
    if os.name == 'nt':
        CHROMEDRIVER_PATH += ".exe"
    with webdriver.Chrome(CHROMEDRIVER_PATH) as driver:
        main()
