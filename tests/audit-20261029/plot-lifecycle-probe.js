const fs=require('fs'),vm=require('vm');const code=fs.readFileSync('src/plot.js','utf8');const events=[],counts={style:0,mutation:0};
const ctx={window:null,document:{currentScript:{textContent:'first'},readyState:'loading',addEventListener:(n,f)=>events.push([n,f]),removeEventListener:(n,f)=>{const i=events.findIndex(e=>e[0]===n&&e[1]===f);if(i>=0)events.splice(i,1)},createElement:()=>({remove(){},style:{}}),head:{appendChild(){counts.style++}},body:{},querySelectorAll:()=>[]},MutationObserver:class{constructor(){counts.mutation++}observe(){}disconnect(){}},setTimeout,clearTimeout,console};
ctx.window=ctx;ctx.addEventListener=()=>{};ctx.removeEventListener=()=>{};ctx.matchMedia=()=>({matches:false});vm.createContext(ctx);vm.runInContext(code,ctx);ctx.document.currentScript.textContent='second';vm.runInContext(code,ctx);for(const[n,f]of events)if(n==='DOMContentLoaded')f();console.log(JSON.stringify({listeners:events.length,...counts}));

require("node:assert/strict").equal(counts.style,1);
require("node:assert/strict").equal(counts.mutation,1);
