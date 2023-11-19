# https://github.com/fffaraz/awesome-cpp#awesome-cpp
# https://raw.githubusercontent.com/fffaraz/awesome-cpp/master/README.md
#
# Download the README.md, then go through each line.
# IF the line starts with *, begin parsing it.
# If the next non-whitespace character is [ then its likely we are looking at a project.
#
# Its like this in the readme:
#   * [name](url) - description
#
# This means we can very easily scrape this stuff and generate rust code from it.
#

import requests

README_URL = "https://raw.githubusercontent.com/fffaraz/awesome-cpp/master/README.md"

page: str | None = None

try:
    with open("scrape_cache.txt", "r") as f:
        page = f.read()
    print("[pakgen] using cached README.md")
except:
    req = requests.get(README_URL)
    if req.status_code != 200:
        print("[pakgen] failed to download README.md!")
        exit(0)
    page = req.content.decode()
    print("[pakgen] downloaded live README.md")
# The content is all that is needed.

with open("scrape_cache.txt", "w+") as f:
    if len(f.readlines()) == 0:
        print("[pakgen] cached scraped projects")
        f.write(page)

# Extracted lines that we think matter, so only ones that begin with * and the next non-whitespace
# character is a [
lines_that_matter = []
lines = page.split("\\n")

skipped = 0

for line in lines:
    if not line.startswith("*"):
        skipped += 1
        continue
    if len(line) < 3:
        skipped += 1
        continue
    if not line[1:].lstrip().startswith("["):
        skipped += 1
        continue
    if "\\" in line:
        skipped += 1
        # skip lines with weird escape sequences.
        continue
    lines_that_matter.append(line)

print(f"[pakgen] skipped {skipped} initial lines due to invalid syntax.")

# actually parsing


def try_parse_info(line):
    name, link, desc, lang = None, None, None, None
    pos = 0

    while line[pos] != "[":
        pos += 1

    # make up for the one iteration we skip
    pos += 1

    # we have the name
    start_pos = pos
    while line[pos] != "]":
        pos += 1
    name = line[start_pos:pos]
    # skip "]" and the "("
    pos += 2

    start_pos = pos
    while line[pos] != ")":
        pos += 1
    link = line[start_pos:pos]
    # Skip ") - "
    pos += 4
    desc = line[pos:]

    name = name.replace('"', "'")
    desc = desc.replace('"', "'")

    if (
        "cpp" in name.lower()
        or "c++" in name.lower()
        or "cpp" in desc.lower()
        or "c++" in desc.lower()
    ):
        lang = "CXX"
    else:
        lang = "C"

    name = name.replace(" ", "-")
    name = name.lower()

    return {"name": name, "link": link, "description": desc, "language": lang}


parsed_info = []
# NOTE: It should be "book" too but it blocks all facebook repositorys...
DISALLOWED_IDENTIFIERS = ["podcast", "videos", "talks", "article", "blog", "cppcon"]
skipped = 0

for line in lines_that_matter:
    data = try_parse_info(line)
    if any(x in data["description"].lower() for x in DISALLOWED_IDENTIFIERS):
        skipped += 1
        continue
    if "https://github.com/" not in data["link"]:
        skipped += 1
        continue
    parsed_info.append(data)

print(
    f"[pakgen] skipped {skipped} packages due to them not being from github or not being projects."
)
json_object = {}

for package in parsed_info:
    json_object[package["name"]] = {
        "url": package["link"],
        "description": package["description"],
        "language": package["language"],
    }

import json

with open("src/pkg_reg.json", "w+") as f:
    json.dump(json_object, f)

print(f"[pakgen] written json object of size {len(json_object)} into src/pkg_reg.json!")
