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
    print("[packgen] using cached README.md")
except:
    req = requests.get(README_URL)
    if req.status_code != 200:
        print("failed to download README.md!")
        exit(0)
    page = req.content.decode()
    print("[packgen] downloaded live README.md")
# The content is all that is needed.

with open("scrape_cache.txt", "w+") as f:
    if len(f.readlines()) == 0:
        print("cached scraped projects")
        f.write(page)

# Extracted lines that we think matter, so only ones that begin with * and the next non-whitespace
# character is a [
lines_that_matter = []
lines = page.split("\\n")

print(lines)

for line in lines:
    if not line.startswith("*"):
        print("skipping line that does not start with `*`")
        continue
    if len(line) < 3:
        print("skipping line not larger than 3 characters.")
        continue
    if not line[1:].lstrip().startswith("["):
        print("skipping line that doesn't have an inline link.")
        continue
    if "\\" in line:
        # skip lines with weird escape sequences.
        continue
    lines_that_matter.append(line)

count = len(lines_that_matter)
print(f"processed {count} lines that seem okay for code generation.")

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

    if "cpp" in name or "c++" in name:
        lang = "CXX"
    else:
        lang = "C"

    name = name.replace(" ", "-")
    name = name.lower()

    return {"name": name, "link": link, "description": desc, "language": lang}


parsed_info = []

for line in lines_that_matter:
    data = try_parse_info(line)
    parsed_info.append(data)

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

print("written json into src/pkg_reg.json!")
