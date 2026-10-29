// Unit tests evaluate helpers from the actual shipped script.
const fs = require('fs'), vm = require('vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/plot.js', 'utf8');
const begin = source.indexOf('  const fmt ='), end = source.indexOf('\n  // a dot', begin);
const context = {}; vm.createContext(context);
vm.runInContext(source.slice(begin, end) + '\nthis.fmt=fmt;', context);
assert.notEqual(context.fmt(1e-15, {digits: 4, sep: '.'}, 0), '0');
assert.notEqual(context.fmt(-1e-200, {digits: 4, sep: ','}, 0), '0');
assert.equal(context.fmt(0, {digits: 4}, 0), '0');
assert.equal(context.fmt(1.234567, {digits: 4}, 0), '1.235');
console.log('plot formatting regression passed');

const valueBegin = source.indexOf('  function valueAt'), valueEnd = source.indexOf('\n  function render', valueBegin);
vm.runInContext(source.slice(valueBegin, valueEnd) + '\nthis.valueAt=valueAt;', context);
const at = (points, x, x0=0, x1=1) => context.valueAt({x0,x1,series:[{c:0,data:false,p:points}]},0,x,0);
assert.equal(at([[1e-200,1],[2e-200,2]],0,0,1e-200),null);
assert.equal(at([[0,-1e308],[1,1e308]],0.5).y,0);
assert.equal(at([[-1e308,0],[1e308,1]],0,-1e308,1e308).y,0.5);
console.log('plot interpolation regressions passed');
