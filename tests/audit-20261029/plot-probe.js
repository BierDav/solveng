const fs = require('fs'), vm=require('vm');
const src=fs.readFileSync('src/plot.js','utf8');
const begin=src.indexOf('  const fmt ='), end=src.indexOf('\n  // a dot',begin);
const valBegin=src.indexOf('  function valueAt'), valEnd=src.indexOf('\n  function render',valBegin);
const ctx={};vm.createContext(ctx);vm.runInContext(src.slice(begin,end)+'\n'+src.slice(valBegin,valEnd)+'\nthis.fmt=fmt;this.valueAt=valueAt;',ctx);
console.log('small formatter',ctx.fmt(1e-15,{digits:4,sep:'.'},0));
console.log('underflow interval outside domain',ctx.valueAt({x0:0,x1:1e-200,series:[{c:0,data:false,p:[[1e-200,1],[2e-200,2]]}]},0,0,0));
console.log('overflow interpolation',ctx.valueAt({x0:0,x1:1,series:[{c:0,data:false,p:[[0,-1e308],[1,1e308]]}]},0,.5,0));
console.log('wide interval',ctx.valueAt({x0:-1e308,x1:1e308,series:[{c:0,data:false,p:[[-1e308,0],[1e308,1]]}]},0,0,0));
