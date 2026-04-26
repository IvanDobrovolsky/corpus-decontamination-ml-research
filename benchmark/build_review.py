"""Build the human review HTML from classification data."""
import json

items = json.load(open("/tmp/review_data.json"))
data_json = json.dumps(items, ensure_ascii=True)

html = r"""<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Propaganda Classification Review</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,system-ui,sans-serif;background:#0d1117;color:#c9d1d9;padding:20px;padding-bottom:70px}
h1{color:#58a6ff;margin-bottom:10px}
.stats{background:#161b22;padding:15px;border-radius:8px;margin-bottom:20px;font-family:monospace;display:flex;gap:20px;flex-wrap:wrap}
.stat-v{color:#58a6ff;font-weight:bold}
.filters{margin-bottom:15px;display:flex;gap:8px;flex-wrap:wrap}
.filters button{padding:6px 14px;border:1px solid #30363d;background:#21262d;color:#c9d1d9;border-radius:6px;cursor:pointer;font-size:13px}
.filters button.active{background:#1f6feb;border-color:#1f6feb;color:#fff}
.card{background:#161b22;border:1px solid #30363d;border-radius:8px;margin-bottom:16px;overflow:hidden}
.card.done{border-color:#238636}
.header{padding:12px 16px;background:#21262d;display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:8px}
.badge{padding:2px 8px;border-radius:4px;font-size:12px;font-weight:600;display:inline-block}
.b-narr{background:#1f6feb;color:#fff}
.b-prop{background:#da3633;color:#fff}
.b-not{background:#238636;color:#fff}
.b-pending{background:#6e7681;color:#fff}
.doc-id{color:#8b949e;font-size:12px}
.kw{color:#d2a8ff;font-size:13px}
.reason{padding:8px 16px;background:#1c2128;color:#8b949e;font-size:13px;border-bottom:1px solid #30363d}
.ctx{padding:16px;font-size:14px;line-height:1.7;white-space:pre-wrap;word-wrap:break-word;max-height:500px;overflow-y:auto}
.hl{background:#ffa65733;color:#ffa657;padding:1px 3px;border-radius:3px;font-weight:600}
.actions{padding:12px 16px;background:#21262d;display:flex;gap:10px;align-items:center}
.actions button{padding:8px 20px;border:none;border-radius:6px;font-size:14px;font-weight:600;cursor:pointer}
.bp{background:#da3633;color:#fff}.bp:hover{background:#f85149}
.bn{background:#238636;color:#fff}.bn:hover{background:#2ea043}
.bp.sel,.bn.sel{outline:3px solid #58a6ff;outline-offset:2px}
.ni{flex:1;padding:6px 10px;background:#0d1117;border:1px solid #30363d;border-radius:6px;color:#c9d1d9;font-size:13px}
.bar{position:fixed;bottom:0;left:0;right:0;background:#161b22;border-top:1px solid #30363d;padding:12px 20px;display:flex;justify-content:space-between;align-items:center;z-index:100}
.bar button{padding:8px 20px;background:#1f6feb;color:#fff;border:none;border-radius:6px;font-size:14px;cursor:pointer;margin-left:10px}
</style>
</head>
<body>
<h1>Propaganda Classification Review</h1>
<div class="stats" id="stats"></div>
<div class="filters" id="filters"></div>
<div id="cards"></div>
<div class="bar"><span id="prog"></span><div><button onclick="xCSV()">Export CSV</button><button onclick="xJSON()">Export JSON</button></div></div>
<script>
var DATA = __DATA__;
var SK = 'prop_review_v2';
var state = {};
try { state = JSON.parse(localStorage.getItem(SK)) || {}; } catch(e) {}
function save() { localStorage.setItem(SK, JSON.stringify(state)); }
var fN = 'all', fS = 'all';

function hl(text, kws) {
  var r = text.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
  kws.forEach(function(kw) {
    var re = new RegExp('(' + kw.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&') + ')', 'gi');
    r = r.replace(re, '<span class="hl">$1</span>');
  });
  return r;
}

function classify(id, val) {
  if (!state[id]) state[id] = {};
  state[id].h = val;
  save();
  render();
}

function setNote(id, v) {
  if (!state[id]) state[id] = {};
  state[id].n = v;
  save();
}

function render() {
  var done=0,prop=0,notp=0,agree=0,dis=0;
  DATA.forEach(function(item) {
    var s = state[item.id];
    if (s && s.h) {
      done++;
      if (s.h === 'P') prop++; else notp++;
      var llm = item.llm_class === 'ASSERTING' ? 'P' : 'N';
      if (s.h === llm) agree++; else dis++;
    }
  });
  document.getElementById('stats').innerHTML =
    '<div>Done: <span class="stat-v">' + done + '/' + DATA.length + '</span></div>' +
    '<div>Propaganda: <span class="stat-v">' + prop + '</span></div>' +
    '<div>Not: <span class="stat-v">' + notp + '</span></div>' +
    '<div>Agree LLM: <span class="stat-v">' + agree + '</span></div>' +
    '<div>Disagree: <span class="stat-v">' + dis + '</span></div>';
  document.getElementById('prog').textContent = done + '/' + DATA.length;

  var narrs = [];
  DATA.forEach(function(d) { if (narrs.indexOf(d.narrative) < 0) narrs.push(d.narrative); });
  var fb = '<button class="' + (fN==='all'?'active':'') + '" onclick="fN=\'all\';render()">All</button>';
  narrs.forEach(function(n) { fb += '<button class="' + (fN===n?'active':'') + '" onclick="fN=\'' + n + '\';render()">' + n + '</button>'; });
  fb += ' &nbsp;|&nbsp; ';
  ['all','pending','done'].forEach(function(st) { fb += '<button class="' + (fS===st?'active':'') + '" onclick="fS=\'' + st + '\';render()">' + st + '</button>'; });
  document.getElementById('filters').innerHTML = fb;

  var html = '';
  DATA.forEach(function(item) {
    if (fN !== 'all' && item.narrative !== fN) return;
    var s = state[item.id] || {};
    if (fS === 'pending' && s.h) return;
    if (fS === 'done' && !s.h) return;
    var llmIs = item.llm_class === 'ASSERTING' ? 'PROPAGANDA' : 'NOT_PROPAGANDA';
    var llmBc = item.llm_class === 'ASSERTING' ? 'b-prop' : 'b-not';
    var hLabel = !s.h ? 'PENDING' : (s.h === 'P' ? 'PROPAGANDA' : 'NOT_PROPAGANDA');
    var hBc = !s.h ? 'b-pending' : (s.h === 'P' ? 'b-prop' : 'b-not');
    html += '<div class="card' + (s.h ? ' done' : '') + '">' +
      '<div class="header"><div>' +
        '<span class="badge b-narr">' + item.narrative + '</span> ' +
        '<span class="doc-id">#' + item.id + ' doc=' + item.doc_id + '</span> ' +
        '<span class="kw">' + item.keywords.join(', ') + '</span>' +
      '</div><div>' +
        'LLM: <span class="badge ' + llmBc + '">' + llmIs + '</span> ' +
        'Human: <span class="badge ' + hBc + '">' + hLabel + '</span>' +
      '</div></div>' +
      '<div class="reason">' + item.llm_raw.replace(/</g,'&lt;') + '</div>' +
      '<div class="ctx">' + hl(item.context, item.keywords) + '</div>' +
      '<div class="actions">' +
        '<button class="bp' + (s.h==='P'?' sel':'') + '" onclick="classify(' + item.id + ',\'P\')">PROPAGANDA</button>' +
        '<button class="bn' + (s.h==='N'?' sel':'') + '" onclick="classify(' + item.id + ',\'N\')">NOT PROPAGANDA</button>' +
        '<input class="ni" placeholder="Notes" value="' + ((s.n||'').replace(/"/g,'&quot;')) + '" onchange="setNote(' + item.id + ',this.value)">' +
      '</div></div>';
  });
  document.getElementById('cards').innerHTML = html;
}

function xCSV() {
  var c = 'id,doc_id,narrative,keywords,llm,human,agree,notes\n';
  DATA.forEach(function(item) {
    var s = state[item.id] || {};
    var llm = item.llm_class === 'ASSERTING' ? 'PROPAGANDA' : 'NOT_PROPAGANDA';
    var human = s.h === 'P' ? 'PROPAGANDA' : (s.h === 'N' ? 'NOT_PROPAGANDA' : '');
    var ag = human ? (((s.h==='P'?'PROPAGANDA':'NOT_PROPAGANDA') === llm) ? 'Y' : 'N') : '';
    c += [item.id,item.doc_id,item.narrative,'"'+item.keywords.join('; ')+'"',llm,human,ag,'"'+(s.n||'')+'"'].join(',') + '\n';
  });
  dl('review_results.csv', c, 'text/csv');
}

function xJSON() {
  var out = DATA.map(function(item) {
    var s = state[item.id] || {};
    var llm = item.llm_class === 'ASSERTING' ? 'PROPAGANDA' : 'NOT_PROPAGANDA';
    var human = s.h === 'P' ? 'PROPAGANDA' : (s.h === 'N' ? 'NOT_PROPAGANDA' : null);
    return {id:item.id,doc_id:item.doc_id,narrative:item.narrative,llm:llm,human:human,note:s.n||''};
  });
  dl('review_results.json', JSON.stringify(out,null,2), 'application/json');
}

function dl(name, content, type) {
  var b = new Blob([content], {type:type});
  var a = document.createElement('a');
  a.href = URL.createObjectURL(b);
  a.download = name;
  a.click();
}

render();
</script>
</body>
</html>"""

html = html.replace("__DATA__", data_json)

with open("benchmark/human_review.html", "w") as f:
    f.write(html)

print(f"Written {len(items)} items, {len(html)} bytes")
