import json, base64

items = json.load(open("/tmp/review_data.json"))
data_b64 = base64.b64encode(json.dumps(items).encode()).decode()
assert len(json.loads(base64.b64decode(data_b64))) == len(items)

CSS = """*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,sans-serif;background:#0d1117;color:#c9d1d9;padding:20px 40px 80px}
h1{color:#58a6ff;margin-bottom:12px}
#stats{background:#161b22;padding:14px 20px;border-radius:8px;margin-bottom:16px;display:flex;gap:24px;flex-wrap:wrap}
#stats b{color:#58a6ff}
#fil{margin-bottom:16px;display:flex;gap:6px;flex-wrap:wrap}
#fil button{padding:5px 12px;border:1px solid #30363d;background:#21262d;color:#c9d1d9;border-radius:6px;cursor:pointer}
#fil button.on{background:#1f6feb;border-color:#1f6feb;color:#fff}
.card{background:#161b22;border:1px solid #30363d;border-radius:8px;margin-bottom:20px}
.card.done{border-color:#238636}
.hdr{padding:10px 16px;background:#21262d;display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:6px}
.tag{padding:3px 8px;border-radius:4px;font-size:11px;font-weight:700;display:inline-block;margin-right:4px}
.t-narr{background:#1f6feb;color:#fff}.t-prop{background:#da3633;color:#fff}.t-not{background:#238636;color:#fff}.t-pend{background:#6e7681;color:#fff}
.docid{color:#8b949e;font-size:11px;margin-right:8px}
.kwds{color:#d2a8ff;font-size:12px}
.reason{padding:8px 16px;background:#1c2128;color:#8b949e;font-size:13px;line-height:1.5;border-bottom:1px solid #21262d}
.text{padding:20px;font-size:15px;line-height:1.9;white-space:pre-wrap;word-wrap:break-word;max-height:700px;overflow-y:auto;border-bottom:1px solid #21262d}
.kwhl{background:rgba(255,166,87,0.25);color:#ffa657;padding:1px 4px;border-radius:3px;font-weight:700}
.btns{padding:10px 16px;background:#21262d;display:flex;gap:10px;align-items:center}
.btns button{padding:10px 28px;border:none;border-radius:6px;font-size:15px;font-weight:700;cursor:pointer}
.btn-p{background:#da3633;color:#fff}.btn-n{background:#238636;color:#fff}
.btn-p.sel,.btn-n.sel{outline:3px solid #58a6ff;outline-offset:2px}
.note{flex:1;padding:8px 12px;background:#0d1117;border:1px solid #30363d;border-radius:6px;color:#c9d1d9}
.bottom{position:fixed;bottom:0;left:0;right:0;background:#161b22;border-top:1px solid #30363d;padding:10px 40px;display:flex;justify-content:space-between;align-items:center;z-index:100}
.bottom button{padding:8px 20px;background:#1f6feb;color:#fff;border:none;border-radius:6px;cursor:pointer;margin-left:8px}"""

# JS uses ONLY double quotes, no single quotes at all, no escaping issues
JS = r"""
var DATA = JSON.parse(atob("PLACEHOLDER"));
var SK = "pr_v7";
var state = {};
try { state = JSON.parse(localStorage.getItem(SK)) || {}; } catch(e) {}
function save() { localStorage.setItem(SK, JSON.stringify(state)); }
var fN = "all";
var fS = "all";

function hlKW(text, kws) {
  var r = text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  for (var i = 0; i < kws.length; i++) {
    var e = kws[i].replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    r = r.replace(new RegExp("(" + e + ")", "gi"), "<span class=\"kwhl\">$1</span>");
  }
  return r;
}

// Use event delegation - no inline onclick needed
document.addEventListener("click", function(e) {
  var t = e.target;
  if (t.dataset.classify) {
    var parts = t.dataset.classify.split(",");
    var id = parseInt(parts[0]);
    var val = parts[1];
    state[id] = state[id] || {};
    state[id].h = val;
    save();
    render();
  }
  if (t.dataset.filtern !== undefined) {
    fN = t.dataset.filtern;
    render();
  }
  if (t.dataset.filters !== undefined) {
    fS = t.dataset.filters;
    render();
  }
});

document.addEventListener("change", function(e) {
  if (e.target.dataset.noteid) {
    var id = parseInt(e.target.dataset.noteid);
    state[id] = state[id] || {};
    state[id].n = e.target.value;
    save();
  }
});

function render() {
  var dn = 0, pp = 0, np = 0, ag = 0, dg = 0;
  for (var i = 0; i < DATA.length; i++) {
    var s = state[DATA[i].id];
    if (s && s.h) {
      dn++;
      if (s.h === "P") pp++; else np++;
      var llmP = DATA[i].llm_class === "ASSERTING" ? "P" : "N";
      if (s.h === llmP) ag++; else dg++;
    }
  }
  document.getElementById("stats").innerHTML =
    "<div>Done: <b>" + dn + "/" + DATA.length + "</b></div>" +
    "<div>Propaganda: <b>" + pp + "</b></div>" +
    "<div>Not: <b>" + np + "</b></div>" +
    "<div>Agree LLM: <b>" + ag + "</b></div>" +
    "<div>Disagree: <b>" + dg + "</b></div>";
  document.getElementById("prog").textContent = dn + "/" + DATA.length;

  var ns = {};
  for (var i = 0; i < DATA.length; i++) ns[DATA[i].narrative] = 1;
  var nl = Object.keys(ns).sort();
  var fb = "<button class=\"" + (fN === "all" ? "on" : "") + "\" data-filtern=\"all\">All</button>";
  for (var i = 0; i < nl.length; i++) {
    fb += "<button class=\"" + (fN === nl[i] ? "on" : "") + "\" data-filtern=\"" + nl[i] + "\">" + nl[i] + "</button>";
  }
  fb += " &nbsp;|&nbsp; ";
  var sts = ["all", "pending", "done"];
  for (var i = 0; i < sts.length; i++) {
    fb += "<button class=\"" + (fS === sts[i] ? "on" : "") + "\" data-filters=\"" + sts[i] + "\">" + sts[i] + "</button>";
  }
  document.getElementById("fil").innerHTML = fb;

  var h = "";
  for (var i = 0; i < DATA.length; i++) {
    var it = DATA[i];
    if (fN !== "all" && it.narrative !== fN) continue;
    var s = state[it.id] || {};
    if (fS === "pending" && s.h) continue;
    if (fS === "done" && !s.h) continue;
    var ll = it.llm_class === "ASSERTING" ? "PROPAGANDA" : "NOT_PROPAGANDA";
    var lt = it.llm_class === "ASSERTING" ? "t-prop" : "t-not";
    var humanL = !s.h ? "PENDING" : (s.h === "P" ? "PROPAGANDA" : "NOT_PROPAGANDA");
    var ht = !s.h ? "t-pend" : (s.h === "P" ? "t-prop" : "t-not");
    var nv = (s.n || "").replace(/"/g, "&quot;");
    var lr = (it.llm_raw || "").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    h += "<div class=\"card" + (s.h ? " done" : "") + "\">" +
      "<div class=\"hdr\"><div>" +
        "<span class=\"tag t-narr\">" + it.narrative + "</span>" +
        "<span class=\"docid\">#" + it.id + " doc=" + it.doc_id + "</span>" +
        "<span class=\"kwds\">" + it.keywords.join(", ") + "</span>" +
      "</div><div>" +
        "LLM: <span class=\"tag " + lt + "\">" + ll + "</span> " +
        "You: <span class=\"tag " + ht + "\">" + humanL + "</span>" +
      "</div></div>" +
      "<div class=\"reason\">" + lr + "</div>" +
      "<div class=\"text\">" + hlKW(it.context, it.keywords) + "</div>" +
      "<div class=\"btns\">" +
        "<button class=\"btn-p" + (s.h === "P" ? " sel" : "") + "\" data-classify=\"" + it.id + ",P\">PROPAGANDA</button>" +
        "<button class=\"btn-n" + (s.h === "N" ? " sel" : "") + "\" data-classify=\"" + it.id + ",N\">NOT PROPAGANDA</button>" +
        "<input class=\"note\" placeholder=\"Notes\" value=\"" + nv + "\" data-noteid=\"" + it.id + "\">" +
      "</div></div>";
  }
  document.getElementById("cards").innerHTML = h;
}

function xCSV() {
  var c = "id,doc_id,narrative,llm,human,agree,notes\n";
  for (var i = 0; i < DATA.length; i++) {
    var it = DATA[i], s = state[it.id] || {};
    var l = it.llm_class === "ASSERTING" ? "PROPAGANDA" : "NOT_PROPAGANDA";
    var hu = s.h === "P" ? "PROPAGANDA" : (s.h === "N" ? "NOT_PROPAGANDA" : "");
    var ag = hu ? (hu === l ? "Y" : "N") : "";
    c += it.id + "," + it.doc_id + "," + it.narrative + "," + l + "," + hu + "," + ag + "\n";
  }
  dl("results.csv", c, "text/csv");
}

function xJSON() {
  var o = [];
  for (var i = 0; i < DATA.length; i++) {
    var it = DATA[i], s = state[it.id] || {};
    o.push({
      id: it.id, doc_id: it.doc_id, narrative: it.narrative,
      llm: it.llm_class === "ASSERTING" ? "PROPAGANDA" : "NOT_PROPAGANDA",
      human: s.h === "P" ? "PROPAGANDA" : (s.h === "N" ? "NOT_PROPAGANDA" : null),
      note: s.n || ""
    });
  }
  dl("results.json", JSON.stringify(o, null, 2), "application/json");
}

function dl(n, c, t) {
  var b = new Blob([c], {type: t});
  var a = document.createElement("a");
  a.href = URL.createObjectURL(b);
  a.download = n;
  a.click();
}

render();
""".replace("PLACEHOLDER", data_b64)

HTML = f"""<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Propaganda Review (103 samples)</title>
<style>{CSS}</style>
</head>
<body>
<h1>Propaganda Classification Review (100 samples, full text)</h1>
<div id="stats"></div>
<div id="fil"></div>
<div id="cards"></div>
<div class="bottom"><span id="prog"></span><div><button onclick="xCSV()">Export CSV</button><button onclick="xJSON()">Export JSON</button></div></div>
<script>
{JS}
</script>
</body>
</html>"""

with open("benchmark/human_review.html", "w") as f:
    f.write(HTML)
print(f"Written {len(HTML):,} bytes")
