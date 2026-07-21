use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=frontend/dist");

    let dist = Path::new("frontend/dist");
    if dist.exists() {
        return;
    }

    std::fs::create_dir_all(dist).expect("failed to create frontend/dist");

    let has_bun = std::process::Command::new("bun")
        .arg("--version")
        .output()
        .is_ok();

    if has_bun {
        let res = std::process::Command::new("bun")
            .args(["install"])
            .current_dir("frontend")
            .status();
        if res.map(|s| s.success()).unwrap_or(false) {
            let res = std::process::Command::new("bun")
                .args(["run", "build"])
                .current_dir("frontend")
                .status();
            if res.map(|s| s.success()).unwrap_or(false) {
                return;
            }
        }
    }

    let has_npm = std::process::Command::new("npm")
        .arg("--version")
        .output()
        .is_ok();

    if has_npm {
        let res = std::process::Command::new("npm")
            .args(["install"])
            .current_dir("frontend")
            .status();
        if res.map(|s| s.success()).unwrap_or(false) {
            let res = std::process::Command::new("npm")
                .args(["run", "build"])
                .current_dir("frontend")
                .status();
            if res.map(|s| s.success()).unwrap_or(false) {
                return;
            }
        }
    }

    let placeholder = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>MediaCleaner Pro</title>
<style>
:root{--bg:#0f0f13;--surface:#1a1a24;--card:#22222f;--border:#2d2d3d;--text:#e4e4ed;--muted:#8888a0;--primary:#6c5ce7;--accent:#00cec9;--success:#00b894;--warning:#fdcb6e;--error:#e17055}
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:var(--bg);color:var(--text);min-height:100vh;display:flex;flex-direction:column;align-items:center}
header{width:100%;padding:1.5rem 2rem;border-bottom:1px solid var(--border);display:flex;align-items:center;justify-content:space-between}
header h1{font-size:1.5rem;font-weight:700}
header h1 span{color:var(--primary)}
header h1 small{font-size:.75rem;color:var(--muted);font-weight:400;margin-left:.5rem}
.status-badge{padding:.35rem .75rem;border-radius:8px;font-size:.8rem;font-weight:500;border:1px solid var(--border)}
.status-badge.running{color:var(--success);border-color:var(--success)}
.status-badge.paused{color:var(--warning);border-color:var(--warning)}
.status-badge.idle{color:var(--muted);border-color:var(--border)}
.container{width:100%;max-width:960px;padding:2rem}
.card{background:var(--surface);border:1px solid var(--border);border-radius:12px;padding:1.5rem;margin-bottom:1rem}
.card h2{font-size:1.1rem;margin-bottom:1rem;color:var(--text)}
.controls{display:flex;gap:.75rem;margin-bottom:1.5rem}
.btn{display:flex;align-items:center;gap:.5rem;padding:.6rem 1.25rem;border-radius:8px;border:none;font-size:.9rem;font-weight:600;cursor:pointer;transition:all .15s}
.btn:hover{transform:scale(1.02)}
.btn:active{transform:scale(.98)}
.btn-primary{background:linear-gradient(135deg,var(--primary),#a29bfe);color:#fff}
.btn-outline{background:var(--card);color:var(--text);border:1px solid var(--border)}
.btn-outline:hover{border-color:var(--primary)}
.stage-list{display:grid;grid-template-columns:1fr 1fr;gap:.75rem}
.stage{background:var(--card);border:1px solid var(--border);border-radius:8px;padding:1rem}
.stage .name{font-size:.85rem;font-weight:600;margin-bottom:.25rem}
.stage .desc{font-size:.75rem;color:var(--muted);margin-bottom:.5rem}
.stage .bar{height:4px;background:var(--border);border-radius:2px;overflow:hidden}
.stage .bar .fill{height:100%;border-radius:2px;transition:width .3s}
.stage .bar .fill.pending{width:0%}
.stage .bar .fill.running{width:50%;background:var(--primary);animation:pulse 1.5s infinite}
.stage .bar .fill.completed{width:100%;background:var(--success)}
.stage .bar .fill.failed{width:100%;background:var(--error)}
.stage .bar .fill.skipped{width:100%;background:var(--muted)}
.stats-row{display:grid;grid-template-columns:repeat(4,1fr);gap:.75rem;margin-top:1rem}
.stat{padding:.75rem;border-radius:8px;text-align:center}
.stat .label{font-size:.7rem;text-transform:uppercase;letter-spacing:.05em;color:var(--muted);margin-bottom:.25rem}
.stat .value{font-size:1.25rem;font-weight:700;font-family:monospace}
.stat.speed{background:var(--primary)15}
.stat.unique{background:var(--success)15}
.stat.duplicates{background:var(--warning)15}
.stat.errors{background:var(--error)15}
.logs{max-height:300px;overflow-y:auto;font-family:monospace;font-size:.8rem}
.logs .line{padding:.25rem 0;border-bottom:1px solid var(--border);color:var(--muted)}
.logs .line:first-child{padding-top:0}
.logs .line .ts{color:var(--muted);margin-right:.5rem}
.logs .line .msg{color:var(--text)}
.progress-overall{margin-bottom:1.5rem}
.progress-overall .bar{height:8px;background:var(--border);border-radius:4px;overflow:hidden}
.progress-overall .bar .fill{height:100%;background:linear-gradient(90deg,var(--primary),var(--accent));border-radius:4px;transition:width .5s}
.progress-overall .info{display:flex;justify-content:space-between;margin-top:.5rem;font-size:.85rem;color:var(--muted)}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.5}}
footer{margin-top:auto;padding:1.5rem;text-align:center;font-size:.75rem;color:var(--muted)}
</style>
</head>
<body>
<header>
<div><h1><span>MediaCleaner</span><small>Pro</small></h1></div>
<div><span class="status-badge idle" id="statusBadge">Connecting...</span></div>
</header>
<div class="container">
<div class="controls">
<button class="btn btn-primary" id="btnStart" onclick="startJob()"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><polygon points="5,3 19,12 5,21"/></svg>Start Processing</button>
<button class="btn btn-outline" id="btnPause" onclick="controlJob('pause')" style="display:none"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>Pause</button>
<button class="btn btn-outline" id="btnResume" onclick="controlJob('resume')" style="display:none"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><polygon points="5,3 19,12 5,21"/></svg>Resume</button>
<button class="btn btn-outline" id="btnCancel" onclick="controlJob('cancel')" style="display:none"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>Cancel</button>
</div>

<div class="progress-overall card" id="progressPanel" style="display:none">
<div class="bar"><div class="fill" id="progressFill" style="width:0%"></div></div>
<div class="info"><span id="stageInfo">Idle</span><span id="etaInfo">ETA: --:--:--</span></div>
<div class="stats-row">
<div class="stat speed"><div class="label">Speed</div><div class="value" id="statSpeed">0 img/s</div></div>
<div class="stat unique"><div class="label">Unique</div><div class="value" id="statUnique">0</div></div>
<div class="stat duplicates"><div class="label">Duplicates</div><div class="value" id="statDups">0</div></div>
<div class="stat errors"><div class="label">Errors</div><div class="value" id="statErrors">0</div></div>
</div>
</div>

<div class="card">
<h2>Pipeline Stages</h2>
<div class="stage-list" id="stageList"></div>
</div>

<div class="card">
<h2>Console</h2>
<div class="logs" id="logContainer"><div class="line">Waiting for logs...</div></div>
</div>
</div>

<footer>MediaCleaner Pro v__VERSION__ — Built with Rust</footer>

<script>
const API = '';
let state = {stages:[],stats:{},is_running:false,is_paused:false};

async function poll(){
 try {
  const r = await fetch(API + '/api/status');
  if(r.ok) state = await r.json();
 } catch(e){}
 pollLogs();
 updateUI();
 setTimeout(poll, 1000);
}

async function pollLogs(){
 try {
  const r = await fetch(API + '/api/logs');
  if(r.ok){
   const logs = await r.json();
   const c = document.getElementById('logContainer');
   if(logs.length){
    c.innerHTML = logs.slice(-50).map(l => '<div class="line"><span class="ts">['+(l.timestamp||'')+']</span><span class="msg">'+escapeHtml(l.message||'')+'</span></div>').join('');
    c.scrollTop = c.scrollHeight;
   }
  }
 } catch(e){}
}

function escapeHtml(s){return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')}

function updateUI(){
 const badge = document.getElementById('statusBadge');
 if(state.is_running){
  badge.textContent = state.is_paused ? 'Paused' : 'Running';
  badge.className = 'status-badge '+(state.is_paused?'paused':'running');
 } else {
  badge.textContent = 'Idle';
  badge.className = 'status-badge idle';
 }

 document.getElementById('btnStart').style.display = state.is_running ? 'none' : '';
 document.getElementById('btnPause').style.display = state.is_running && !state.is_paused ? '' : 'none';
 document.getElementById('btnResume').style.display = state.is_running && state.is_paused ? '' : 'none';
 document.getElementById('btnCancel').style.display = state.is_running ? '' : 'none';

 const pp = document.getElementById('progressPanel');
 pp.style.display = state.is_running ? '' : 'none';

 const stages = state.stages || [];
 const total = stages.length;
 const pct = total ? stages.reduce((a,s) => a + (s.progress||0), 0) / total : 0;
 document.getElementById('progressFill').style.width = pct+'%';
 const cs = stages.find(s => s.status === 'running');
 document.getElementById('stageInfo').textContent = cs ? 'Stage: '+cs.name : 'Idle';
 const eta = state.stats && state.stats.eta_seconds;
 document.getElementById('etaInfo').textContent = 'ETA: '+fmtEta(eta||0);
 document.getElementById('statSpeed').textContent = (state.stats&&state.stats.speed||0).toFixed(1)+' img/s';
 document.getElementById('statUnique').textContent = (state.stats&&state.stats.unique_count||0).toLocaleString();
 document.getElementById('statDups').textContent = (state.stats&&state.stats.duplicate_count||0).toLocaleString();
 document.getElementById('statErrors').textContent = (state.stats&&state.stats.error_count||0).toLocaleString();

 const sl = document.getElementById('stageList');
 sl.innerHTML = stages.map((s,i) => {
  const barClass = s.status;
  return '<div class="stage"><div class="name">'+(i+1)+'. '+escapeHtml(s.name)+'</div><div class="desc">'+escapeHtml(s.description)+'</div><div class="bar"><div class="fill '+barClass+'" style="width:'+(s.progress||0)+'%"></div></div><div style="font-size:.7rem;color:var(--muted);margin-top:.25rem">'+(s.processed||0)+'/'+(s.total||0)+'</div></div>';
 }).join('');
}

function fmtEta(s){
 if(!s||s<=0)return'--:--:--';
 const h=Math.floor(s/3600), m=Math.floor((s%3600)/60), sec=s%60;
 return String(h).padStart(2,'0')+':'+String(m).padStart(2,'0')+':'+String(sec).padStart(2,'0');
}

async function startJob(){
 const r = await fetch(API + '/api/start', {
  method:'POST',
  headers:{'Content-Type':'application/json'},
  body: JSON.stringify({source_dir:'',dest_dir:'',hamming_threshold:4})
 });
 if(r.ok) poll();
}

async function controlJob(action){
 await fetch(API + '/api/control', {
  method:'POST',
  headers:{'Content-Type':'application/json'},
  body: JSON.stringify({action})
 });
 poll();
}

poll();
</script>
</body>
</html>"#;
    let placeholder = placeholder.replace("__VERSION__", env!("CARGO_PKG_VERSION"));
    std::fs::write(dist.join("index.html"), placeholder)
        .expect("failed to write placeholder index.html");
    eprintln!("---");
    eprintln!("  frontend/dist/ not found — using embedded placeholder UI");
    eprintln!("  Run 'cd frontend && npm install && npm run build' for the full UI");
    eprintln!("---");
}
