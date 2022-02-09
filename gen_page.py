import json
from urllib.parse import quote_plus

with open("src/fragments.json") as f:
    frags = json.load(f)

with open("src/set_effects.json") as f:
    set_effects = json.load(f)

categories = [
    'Combat (T4)', 'Combat (T3)', 'Combat (T2)', 'Combat (T1)', 'Skilling', 'Harvesting', 'Production', 'Miscellaneous'
]

# category_locations = {
#         'Combat (T4)': 0, 'Combat (T3)': 0, 'Combat (T2)': 0, 'Combat (T1)': 0,
#         'Harvesting': 0, 'Production': 0, 'Skilling': 0, 'Miscellaneous': 0

# }

def normalize(var):
    return var.lower().replace(" ", "-").replace("&", "n").replace("!", "").replace("'", "")

def genradio(id, extra):
    id = normalize(id)
    return f"""\
<div class="radio-toolbar">\
<input {extra} type="radio" id="{id}-radio-never" name="{id}-radio" value="-10000000">\
<label {extra} for="{id}-radio-never">!</label>\
<input {extra} type="radio" id="{id}-radio-less" name="{id}-radio" value="-1">\
<label {extra} for="{id}-radio-less">-</label>\
<input {extra} type="radio" id="{id}-radio-neutral" name="{id}-radio" value="0" checked>\
<label {extra} for="{id}-radio-neutral">0</label>\
<input {extra} type="radio" id="{id}-radio-more" name="{id}-radio" value="1">\
<label {extra} for="{id}-radio-more">+</label>\
<input {extra} type="radio" id="{id}-radio-doublemore" name="{id}-radio" value="2">\
<label {extra} for="{id}-radio-doublemore">&#8225;</label>\
<input {extra} type="radio" id="{id}-radio-always" name="{id}-radio" value="1000">\
<label {extra} for="{id}-radio-always">&amp;</label>\
</div>\
"""

out = []
for category in categories:
    out.append(fr"""
    <div class="tablewrapper">
    <table>
    <caption>
    <h3>{category.title()}</h3>
    </caption>
    <thead>
    </thead>
    <tbody>
    """)
    for frag in sorted(frags, key=lambda frag: frag["name"]):
        if frag["category"] != category: continue
        extra = f'data-name="{frag["name"]}"'
        icon = "https://oldschool.runescape.wiki/images/" + quote_plus(frag["name"].replace(" ", "_")) + ".png"

        out.append(fr"""
        <tr><td><img class="icon" src="{icon}" title="{frag["name"]}"></td><td>{frag["name"]}</td><td>{genradio(frag["name"], extra)}</td></tr>
        """)

    out.append(fr"""
    </tbody>
    </table>
    </div>
    """)

tablegrid = "\n".join(s.strip() for s in out)


out = []
out.append(fr"""
<div class="tablewrapper">
<table>
<caption>
<h3>Set Effects</h3>
<p>Note: <em>only the highest active set</em> in a combination is counted. If you're interested
in both (2) and (3), select both.</p>
</caption>
<thead>
</thead>
<tbody>
""")
for effect in sorted(set_effects, key=lambda s: s["name"]):
    for lvl in range(effect["minimum"], 1+effect["maximum"]):
        lvltag = f" ({lvl})" if effect["minimum"] != effect["maximum"] else ""
        extra = f'data-lvl="{lvl}" data-name="{effect["name"]}"'
        iconname = effect["name"].replace("Last Recall", "Last Recall (Shattered Relics)")
        icon = "https://oldschool.runescape.wiki/images/" + quote_plus(iconname.replace(" ", "_")) + ".png"
        out.append(fr"""
        <tr><td><img class="icon" src="{icon}" title="{effect["name"]}"></td><td>{effect["name"]}{lvltag}</td><td>{genradio(effect["name"] + "-" + str(lvl), extra)}</td></tr>
        """)
out.append(fr"""
</tbody>
</table>
</div>
""")

seteffects = "\n".join(s.strip() for s in out)



with open("index_template.html") as f:
    template = f.read()

out = template
out = out.replace("TABLEGRIDHERE", tablegrid)
out = out.replace("SETEFFECTSHERE", seteffects)

with open("docs/index.html", "w") as f:
    f.write(out)

