var Server;(()=>{"use strict";var t={631:(t,e,s)=>{s.r(e),s.d(e,{Process:()=>r,ProcessFile:()=>i});var n=s(412);function i(t,e,s){for(let s of t)e.readAsText(s,"UTF-8")}function r(t,e){var s;const i=null===(s=t.target)||void 0===s?void 0:s.result;if(null==i)return void alert("File is empty or invalid");const r=i.split("\r\n").map((t=>t.split("\t"))).map((t=>t.map((t=>parseFloat(t))))).filter((t=>8.85<t[0]&&t[0]<36)),l=r.map((t=>t[0])),h=r.map((t=>t[1])),c=l.map((t=>t%150==0?t:"")),u=function(t){const e=[0];for(;e[e.length-1]+1<t.length;){let s=Number.MAX_SAFE_INTEGER,n=e[e.length-1];const i=e[e.length-1]+5;for(let e=i+1;e<t.length;e++){let r=(t[e]-t[i])/(e-i);r<s&&(s=r,n=e)}e.push(n)}return e}(h),d=function(t,e){const s=[];for(let n=1;n<t.length;n++){let i=t[n-1],r=t[n],o=(e[r]-e[i])/(r-i);for(let t=0;t<r-i;t++)s.push(o*t+e[i])}return s}(u,h),{peaks:p,gapped:m,first:f,second:g,valleys:v}=function(t,e){const s=a(t),n=a(s),i=[];for(let n=1;n<s.length;n++)i.push(s[n-1]*s[n]<=-e?t[n-1]:null);const r=[];for(let s=0;s<n.length;s++)r.push(n[s-1]*n[s]<=-e?t[s-1]:null);const o={peaks:[],valleys:[],gapped:[],first:i,second:r};for(let e=1;e<t.length;e++){for(;t[e-1]>t[e];)e++;if(e>=t.length)break;for(o.valleys.push(e),o.gapped[e]=t[e];t[e-1]<t[e];)e++;if(e>=t.length)break;o.peaks.push(e),o.gapped[e]=t[e]}return o}(h,e),{areas:x,ranges:y}=function(t,e,s){const n=[],i=[];for(let r=1;r<s.length;r++)n.push([s[r-1],s[r]]),i.push(o(t,e,s[r-1],s[r]));return{ranges:n,areas:i}}(h,d,v),w=[];for(let t=0;t<x.length;t++)w.push([y[t].map((t=>l[t])).join("-"),x[t],l[p[t]]].join(", "));document.getElementById("heya").innerHTML=w.join("<br>");const b={filteredData:r,times:l,values:h,areas:x,baseline:u,floor:d,peaks:p,gapped:m,labels:c,first:f,second:g};(0,n.p)(b)}function o(t,e,s,n){let i=0;for(let r=s;r<n;r++)i+=(t[r]-e[r]+t[r+1]-e[r+1])/2;return i}function a(t){const e=[t[0],t[1]];for(let s=2;s<t.length;s++)e.push((t[s]-t[s-2])/2);return e}},412:(t,e,s)=>{s.d(e,{p:()=>H});const n={svg:"http://www.w3.org/2000/svg",xmlns:"http://www.w3.org/2000/xmlns/",xhtml:"http://www.w3.org/1999/xhtml",xlink:"http://www.w3.org/1999/xlink",ct:"http://gionkunz.github.com/chartist-js/ct"},i=8,r={"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#039;"};function o(t,e){return"number"==typeof t?t+e:t}function a(t){if("string"==typeof t){const e=/^(\d+)\s*(.*)$/g.exec(t);return{value:e?+e[1]:0,unit:(null==e?void 0:e[2])||void 0}}return{value:Number(t)}}const l=2221e-19;function h(t,e,s){return e/s.range*t}function c(t,e){const s=Math.pow(10,e||i);return Math.round(t*s)/s}function u(){let t=arguments.length>0&&void 0!==arguments[0]?arguments[0]:{};for(var e=arguments.length,s=new Array(e>1?e-1:0),n=1;n<e;n++)s[n-1]=arguments[n];for(let e=0;e<s.length;e++){const n=s[e];for(const e in n){const s=n[e];t[e]="object"!=typeof s||null===s||s instanceof Array?s:u(t[e],s)}}return t}const d=t=>t;function p(t,e){return Array.from({length:t},e?(t,s)=>e(s):()=>{})}function m(t,e){return null!==t&&"object"==typeof t&&Reflect.has(t,e)}function f(t){return null!==t&&isFinite(t)}function g(t){return f(t)?Number(t):void 0}function v(t,e){const s=Array.isArray(t)?t[e]:m(t,"data")?t.data[e]:null;return m(s,"meta")?s.meta:void 0}function x(t){return null==t||"number"==typeof t&&isNaN(t)}function y(t){let e=arguments.length>1&&void 0!==arguments[1]?arguments[1]:"y";return function(t){return"object"==typeof t&&null!==t&&(Reflect.has(t,"x")||Reflect.has(t,"y"))}(t)&&m(t,e)?g(t[e]):g(t)}function w(t,e){if(!x(t))return e?function(t,e){let s,n;if("object"!=typeof t){const i=g(t);"x"===e?s=i:n=i}else m(t,"x")&&(s=g(t.x)),m(t,"y")&&(n=g(t.y));if(void 0!==s||void 0!==n)return{x:s,y:n}}(t,e):g(t)}function b(t,e){return Array.isArray(t)?t.map((t=>m(t,"value")?w(t.value,e):w(t,e))):b(t.data,e)}function E(t,e,s){if(n=t,Array.isArray(n)&&n.every((t=>Array.isArray(t)||m(t,"data"))))return t.map((t=>b(t,e)));var n;const i=b(t,e);return s?i.map((t=>[t])):i}function A(t){let e="";return null==t?t:(e="number"==typeof t?""+t:"object"==typeof t?JSON.stringify({data:t}):String(t),Object.keys(r).reduce(((t,e)=>t.replaceAll(e,r[e])),e))}class C{call(t,e){return this.svgElements.forEach((s=>Reflect.apply(s[t],s,e))),this}attr(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("attr",e)}elem(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("elem",e)}root(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("root",e)}getNode(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("getNode",e)}foreignObject(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("foreignObject",e)}text(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("text",e)}empty(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("empty",e)}remove(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("remove",e)}addClass(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("addClass",e)}removeClass(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("removeClass",e)}removeAllClasses(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("removeAllClasses",e)}animate(){for(var t=arguments.length,e=new Array(t),s=0;s<t;s++)e[s]=arguments[s];return this.call("animate",e)}constructor(t){this.svgElements=[];for(let e=0;e<t.length;e++)this.svgElements.push(new L(t[e]))}}const M={easeInSine:[.47,0,.745,.715],easeOutSine:[.39,.575,.565,1],easeInOutSine:[.445,.05,.55,.95],easeInQuad:[.55,.085,.68,.53],easeOutQuad:[.25,.46,.45,.94],easeInOutQuad:[.455,.03,.515,.955],easeInCubic:[.55,.055,.675,.19],easeOutCubic:[.215,.61,.355,1],easeInOutCubic:[.645,.045,.355,1],easeInQuart:[.895,.03,.685,.22],easeOutQuart:[.165,.84,.44,1],easeInOutQuart:[.77,0,.175,1],easeInQuint:[.755,.05,.855,.06],easeOutQuint:[.23,1,.32,1],easeInOutQuint:[.86,0,.07,1],easeInExpo:[.95,.05,.795,.035],easeOutExpo:[.19,1,.22,1],easeInOutExpo:[1,0,0,1],easeInCirc:[.6,.04,.98,.335],easeOutCirc:[.075,.82,.165,1],easeInOutCirc:[.785,.135,.15,.86],easeInBack:[.6,-.28,.735,.045],easeOutBack:[.175,.885,.32,1.275],easeInOutBack:[.68,-.55,.265,1.55]};function O(t,e,s){let n=arguments.length>3&&void 0!==arguments[3]&&arguments[3],i=arguments.length>4?arguments[4]:void 0;const{easing:r,...l}=s,h={};let c,u;r&&(c=Array.isArray(r)?r:M[r]),l.begin=o(l.begin,"ms"),l.dur=o(l.dur,"ms"),c&&(l.calcMode="spline",l.keySplines=c.join(" "),l.keyTimes="0;1"),n&&(l.fill="freeze",h[e]=l.from,t.attr(h),u=a(l.begin||0).value,l.begin="indefinite");const d=t.elem("animate",{attributeName:e,...l});n&&setTimeout((()=>{try{d._node.beginElement()}catch(s){h[e]=l.to,t.attr(h),d.remove()}}),u);const p=d.getNode();i&&p.addEventListener("beginEvent",(()=>i.emit("animationBegin",{element:t,animate:p,params:s}))),p.addEventListener("endEvent",(()=>{i&&i.emit("animationEnd",{element:t,animate:p,params:s}),n&&(h[e]=l.to,t.attr(h),d.remove())}))}class L{attr(t,e){return"string"==typeof t?e?this._node.getAttributeNS(e,t):this._node.getAttribute(t):(Object.keys(t).forEach((e=>{if(void 0!==t[e])if(-1!==e.indexOf(":")){const s=e.split(":");this._node.setAttributeNS(n[s[0]],e,String(t[e]))}else this._node.setAttribute(e,String(t[e]))})),this)}elem(t,e,s){return new L(t,e,s,this,arguments.length>3&&void 0!==arguments[3]&&arguments[3])}parent(){return this._node.parentNode instanceof SVGElement?new L(this._node.parentNode):null}root(){let t=this._node;for(;"svg"!==t.nodeName&&t.parentElement;)t=t.parentElement;return new L(t)}querySelector(t){const e=this._node.querySelector(t);return e?new L(e):null}querySelectorAll(t){const e=this._node.querySelectorAll(t);return new C(e)}getNode(){return this._node}foreignObject(t,e,s){let i,r=arguments.length>3&&void 0!==arguments[3]&&arguments[3];if("string"==typeof t){const e=document.createElement("div");e.innerHTML=t,i=e.firstChild}else i=t;i instanceof Element&&i.setAttribute("xmlns",n.xmlns);const o=this.elem("foreignObject",e,s,r);return o._node.appendChild(i),o}text(t){return this._node.appendChild(document.createTextNode(t)),this}empty(){for(;this._node.firstChild;)this._node.removeChild(this._node.firstChild);return this}remove(){var t;return null===(t=this._node.parentNode)||void 0===t||t.removeChild(this._node),this.parent()}replace(t){var e;return null===(e=this._node.parentNode)||void 0===e||e.replaceChild(t._node,this._node),t}append(t){return arguments.length>1&&void 0!==arguments[1]&&arguments[1]&&this._node.firstChild?this._node.insertBefore(t._node,this._node.firstChild):this._node.appendChild(t._node),this}classes(){const t=this._node.getAttribute("class");return t?t.trim().split(/\s+/):[]}addClass(t){return this._node.setAttribute("class",this.classes().concat(t.trim().split(/\s+/)).filter((function(t,e,s){return s.indexOf(t)===e})).join(" ")),this}removeClass(t){const e=t.trim().split(/\s+/);return this._node.setAttribute("class",this.classes().filter((t=>-1===e.indexOf(t))).join(" ")),this}removeAllClasses(){return this._node.setAttribute("class",""),this}height(){return this._node.getBoundingClientRect().height}width(){return this._node.getBoundingClientRect().width}animate(t){let e=!(arguments.length>1&&void 0!==arguments[1])||arguments[1],s=arguments.length>2?arguments[2]:void 0;return Object.keys(t).forEach((n=>{const i=t[n];Array.isArray(i)?i.forEach((t=>O(this,n,t,!1,s))):O(this,n,i,e,s)})),this}constructor(t,e,s,i,r=!1){t instanceof Element?this._node=t:(this._node=document.createElementNS(n.svg,t),"svg"===t&&this.attr({"xmlns:ct":n.ct})),e&&this.attr(e),s&&this.addClass(s),i&&(r&&i._node.firstChild?i._node.insertBefore(this._node,i._node.firstChild):i._node.appendChild(this._node))}}function N(t,e,s){let n;const i=[];function r(i){const r=n;n=u({},t),e&&e.forEach((t=>{window.matchMedia(t[0]).matches&&(n=u(n,t[1]))})),s&&i&&s.emit("optionsChanged",{previousOptions:r,currentOptions:n})}if(!window.matchMedia)throw new Error("window.matchMedia not found! Make sure you're using a polyfill.");return e&&e.forEach((t=>{const e=window.matchMedia(t[0]);e.addEventListener("change",r),i.push(e)})),r(),{removeMediaQueryListeners:function(){i.forEach((t=>t.removeEventListener("change",r)))},getCurrentOptions:()=>n}}L.Easing=M;const _={m:["x","y"],l:["x","y"],c:["x1","y1","x2","y2","x","y"],a:["rx","ry","xAr","lAf","sf","x","y"]},S={accuracy:3};function j(t,e,s,n,i,r){const o={command:i?t.toLowerCase():t.toUpperCase(),...e,...r?{data:r}:{}};s.splice(n,0,o)}function I(t,e){t.forEach(((s,n)=>{_[s.command.toLowerCase()].forEach(((i,r)=>{e(s,i,n,r,t)}))}))}class k{static join(t){const e=new k(arguments.length>1&&void 0!==arguments[1]&&arguments[1],arguments.length>2?arguments[2]:void 0);for(let s=0;s<t.length;s++){const n=t[s];for(let t=0;t<n.pathElements.length;t++)e.pathElements.push(n.pathElements[t])}return e}position(t){return void 0!==t?(this.pos=Math.max(0,Math.min(this.pathElements.length,t)),this):this.pos}remove(t){return this.pathElements.splice(this.pos,t),this}move(t,e){let s=arguments.length>2&&void 0!==arguments[2]&&arguments[2],n=arguments.length>3?arguments[3]:void 0;return j("M",{x:+t,y:+e},this.pathElements,this.pos++,s,n),this}line(t,e){let s=arguments.length>2&&void 0!==arguments[2]&&arguments[2],n=arguments.length>3?arguments[3]:void 0;return j("L",{x:+t,y:+e},this.pathElements,this.pos++,s,n),this}curve(t,e,s,n,i,r){let o=arguments.length>6&&void 0!==arguments[6]&&arguments[6],a=arguments.length>7?arguments[7]:void 0;return j("C",{x1:+t,y1:+e,x2:+s,y2:+n,x:+i,y:+r},this.pathElements,this.pos++,o,a),this}arc(t,e,s,n,i,r,o){let a=arguments.length>7&&void 0!==arguments[7]&&arguments[7],l=arguments.length>8?arguments[8]:void 0;return j("A",{rx:t,ry:e,xAr:s,lAf:n,sf:i,x:r,y:o},this.pathElements,this.pos++,a,l),this}parse(t){const e=t.replace(/([A-Za-z])(-?[0-9])/g,"$1 $2").replace(/([0-9])([A-Za-z])/g,"$1 $2").split(/[\s,]+/).reduce(((t,e)=>(e.match(/[A-Za-z]/)&&t.push([]),t[t.length-1].push(e),t)),[]);"Z"===e[e.length-1][0].toUpperCase()&&e.pop();const s=e.map((t=>{const e=t.shift(),s=_[e.toLowerCase()];return{command:e,...s.reduce(((e,s,n)=>(e[s]=+t[n],e)),{})}}));return this.pathElements.splice(this.pos,0,...s),this.pos+=s.length,this}stringify(){const t=Math.pow(10,this.options.accuracy);return this.pathElements.reduce(((e,s)=>{const n=_[s.command.toLowerCase()].map((e=>{const n=s[e];return this.options.accuracy?Math.round(n*t)/t:n}));return e+s.command+n.join(",")}),"")+(this.close?"Z":"")}scale(t,e){return I(this.pathElements,((s,n)=>{s[n]*="x"===n[0]?t:e})),this}translate(t,e){return I(this.pathElements,((s,n)=>{s[n]+="x"===n[0]?t:e})),this}transform(t){return I(this.pathElements,((e,s,n,i,r)=>{const o=t(e,s,n,i,r);(o||0===o)&&(e[s]=o)})),this}clone(){const t=new k(arguments.length>0&&void 0!==arguments[0]&&arguments[0]||this.close);return t.pos=this.pos,t.pathElements=this.pathElements.slice().map((t=>({...t}))),t.options={...this.options},t}splitByCommand(t){const e=[new k];return this.pathElements.forEach((s=>{s.command===t.toUpperCase()&&0!==e[e.length-1].pathElements.length&&e.push(new k),e[e.length-1].pathElements.push(s)})),e}constructor(t=!1,e){this.close=t,this.pathElements=[],this.pos=0,this.options={...S,...e}}}function z(t){const e={fillHoles:!1,...t};return function(t,s){const n=new k;let i=!0;for(let r=0;r<t.length;r+=2){const o=t[r],a=t[r+1],l=s[r/2];void 0!==y(l.value)?(i?n.move(o,a,!1,l):n.line(o,a,!1,l),i=!1):e.fillHoles||(i=!0)}return n}}function P(t){const e={fillHoles:!1,...t};return function t(s,n){const i=function(t,e,s){const n={increasingX:!1,fillHoles:!1,...s},i=[];let r=!0;for(let s=0;s<t.length;s+=2)void 0===y(e[s/2].value)?n.fillHoles||(r=!0):(n.increasingX&&s>=2&&t[s]<=t[s-2]&&(r=!0),r&&(i.push({pathCoordinates:[],valueData:[]}),r=!1),i[i.length-1].pathCoordinates.push(t[s],t[s+1]),i[i.length-1].valueData.push(e[s/2]));return i}(s,n,{fillHoles:e.fillHoles,increasingX:!0});if(i.length){if(i.length>1)return k.join(i.map((e=>t(e.pathCoordinates,e.valueData))));{if(s=i[0].pathCoordinates,n=i[0].valueData,s.length<=4)return z()(s,n);const t=[],e=[],r=s.length/2,o=[],a=[],l=[],h=[];for(let n=0;n<r;n++)t[n]=s[2*n],e[n]=s[2*n+1];for(let s=0;s<r-1;s++)l[s]=e[s+1]-e[s],h[s]=t[s+1]-t[s],a[s]=l[s]/h[s];o[0]=a[0],o[r-1]=a[r-2];for(let t=1;t<r-1;t++)0===a[t]||0===a[t-1]||a[t-1]>0!=a[t]>0?o[t]=0:(o[t]=3*(h[t-1]+h[t])/((2*h[t]+h[t-1])/a[t-1]+(h[t]+2*h[t-1])/a[t]),isFinite(o[t])||(o[t]=0));const c=(new k).move(t[0],e[0],!1,n[0]);for(let s=0;s<r-1;s++)c.curve(t[s]+h[s]/3,e[s]+o[s]*h[s]/3,t[s+1]-h[s]/3,e[s+1]-o[s+1]*h[s]/3,t[s+1],e[s+1],!1,n[s+1]);return c}}return z()([],[])}}class X{on(t,e){const{allListeners:s,listeners:n}=this;"*"===t?s.add(e):(n.has(t)||n.set(t,new Set),n.get(t).add(e))}off(t,e){const{allListeners:s,listeners:n}=this;if("*"===t)e?s.delete(e):s.clear();else if(n.has(t)){const s=n.get(t);e?s.delete(e):s.clear(),s.size||n.delete(t)}}emit(t,e){const{allListeners:s,listeners:n}=this;n.has(t)&&n.get(t).forEach((t=>t(e))),s.forEach((s=>s(t,e)))}constructor(){this.listeners=new Map,this.allListeners=new Set}}const R=new WeakMap;class B{update(t,e){let s=arguments.length>2&&void 0!==arguments[2]&&arguments[2];var n;return t&&(this.data=t||{},this.data.labels=this.data.labels||[],this.data.series=this.data.series||[],this.eventEmitter.emit("data",{type:"update",data:this.data})),e&&(this.options=u({},s?this.options:this.defaultOptions,e),this.initializeTimeoutId||(null===(n=this.optionsProvider)||void 0===n||n.removeMediaQueryListeners(),this.optionsProvider=N(this.options,this.responsiveOptions,this.eventEmitter))),!this.initializeTimeoutId&&this.optionsProvider&&this.createChart(this.optionsProvider.getCurrentOptions()),this}detach(){var t;return this.initializeTimeoutId?window.clearTimeout(this.initializeTimeoutId):(window.removeEventListener("resize",this.resizeListener),null===(t=this.optionsProvider)||void 0===t||t.removeMediaQueryListeners()),R.delete(this.container),this}on(t,e){return this.eventEmitter.on(t,e),this}off(t,e){return this.eventEmitter.off(t,e),this}initialize(){window.addEventListener("resize",this.resizeListener),this.optionsProvider=N(this.options,this.responsiveOptions,this.eventEmitter),this.eventEmitter.on("optionsChanged",(()=>this.update())),this.options.plugins&&this.options.plugins.forEach((t=>{Array.isArray(t)?t[0](this,t[1]):t(this)})),this.eventEmitter.emit("data",{type:"initial",data:this.data}),this.createChart(this.optionsProvider.getCurrentOptions()),this.initializeTimeoutId=null}constructor(t,e,s,n,i){this.data=e,this.defaultOptions=s,this.options=n,this.responsiveOptions=i,this.eventEmitter=new X,this.resizeListener=()=>this.update(),this.initializeTimeoutId=setTimeout((()=>this.initialize()),0);const r="string"==typeof t?document.querySelector(t):t;if(!r)throw new Error("Target element is not found");this.container=r;const o=R.get(r);o&&o.detach(),R.set(r,this)}}const Y={x:{pos:"x",len:"width",dir:"horizontal",rectStart:"x1",rectEnd:"x2",rectOffset:"y2"},y:{pos:"y",len:"height",dir:"vertical",rectStart:"y2",rectEnd:"y1",rectOffset:"x1"}};class T{createGridAndLabels(t,e,s,n){const i="x"===this.units.pos?s.axisX:s.axisY,r=this.ticks.map(((t,e)=>this.projectValue(t,e))),o=this.ticks.map(i.labelInterpolationFnc);r.forEach(((a,l)=>{const h=o[l],c={x:0,y:0};let u;var d;u=r[l+1]?r[l+1]-a:Math.max(this.axisLength-a,this.axisLength/this.ticks.length),""!==h&&(!(d=h)&&0!==d)||("x"===this.units.pos?(a=this.chartRect.x1+a,c.x=s.axisX.labelOffset.x,"start"===s.axisX.position?c.y=this.chartRect.padding.top+s.axisX.labelOffset.y+5:c.y=this.chartRect.y1+s.axisX.labelOffset.y+5):(a=this.chartRect.y1-a,c.y=s.axisY.labelOffset.y-u,"start"===s.axisY.position?c.x=this.chartRect.padding.left+s.axisY.labelOffset.x:c.x=this.chartRect.x2+s.axisY.labelOffset.x+10),i.showGrid&&function(t,e,s,n,i,r,o,a){const l={["".concat(s.units.pos,"1")]:t,["".concat(s.units.pos,"2")]:t,["".concat(s.counterUnits.pos,"1")]:n,["".concat(s.counterUnits.pos,"2")]:n+i},h=r.elem("line",l,o.join(" "));a.emit("draw",{type:"grid",axis:s,index:e,group:r,element:h,...l})}(a,l,this,this.gridOffset,this.chartRect[this.counterUnits.len](),t,[s.classNames.grid,s.classNames[this.units.dir]],n),i.showLabel&&function(t,e,s,n,i,r,o,a,l,h){const c={[i.units.pos]:t+o[i.units.pos],[i.counterUnits.pos]:o[i.counterUnits.pos],[i.units.len]:e,[i.counterUnits.len]:Math.max(0,r-10)},u=Math.round(c[i.units.len]),d=Math.round(c[i.counterUnits.len]),p=document.createElement("span");p.className=l.join(" "),p.style[i.units.len]=u+"px",p.style[i.counterUnits.len]=d+"px",p.textContent=String(n);const m=a.foreignObject(p,{style:"overflow: visible;",...c});h.emit("draw",{type:"label",axis:i,index:s,group:a,element:m,text:n,...c})}(a,u,l,h,this,i.offset,c,e,[s.classNames.label,s.classNames[this.units.dir],"start"===i.position?s.classNames[i.position]:s.classNames.end],n))}))}constructor(t,e,s){this.units=t,this.chartRect=e,this.ticks=s,this.counterUnits=t===Y.x?Y.y:Y.x,this.axisLength=e[this.units.rectEnd]-e[this.units.rectStart],this.gridOffset=e[this.units.rectOffset]}}class U extends T{projectValue(t){const e=Number(y(t,this.units.pos));return this.axisLength*(e-this.bounds.min)/this.bounds.range}constructor(t,e,s,n){const i=n.highLow||function(t,e,s){const n={high:void 0===(e={...e,...s?"x"===s?e.axisX:e.axisY:{}}).high?-Number.MAX_VALUE:+e.high,low:void 0===e.low?Number.MAX_VALUE:+e.low},i=void 0===e.high,r=void 0===e.low;return(i||r)&&function t(e){if(!x(e))if(Array.isArray(e))for(let s=0;s<e.length;s++)t(e[s]);else{const t=Number(s&&m(e,s)?e[s]:e);i&&t>n.high&&(n.high=t),r&&t<n.low&&(n.low=t)}}(t),(e.referenceValue||0===e.referenceValue)&&(n.high=Math.max(e.referenceValue,n.high),n.low=Math.min(e.referenceValue,n.low)),n.high<=n.low&&(0===n.low?n.high=1:n.low<0?n.high=0:(n.high>0||(n.high=1),n.low=0)),n}(e,n,t.pos),r=function(t,e,s){let n=arguments.length>3&&void 0!==arguments[3]&&arguments[3];const i={high:e.high,low:e.low,valueRange:0,oom:0,step:0,min:0,max:0,range:0,numberOfSteps:0,values:[]};var r;i.valueRange=i.high-i.low,i.oom=(r=i.valueRange,Math.floor(Math.log(Math.abs(r))/Math.LN10)),i.step=Math.pow(10,i.oom),i.min=Math.floor(i.low/i.step)*i.step,i.max=Math.ceil(i.high/i.step)*i.step,i.range=i.max-i.min,i.numberOfSteps=Math.round(i.range/i.step);const o=h(t,i.step,i)<s,a=n?function(t){if(1===t)return t;function e(t,s){return t%s==0?s:e(s,t%s)}function s(t){return t*t+1}let n,i=2,r=2;if(t%2==0)return 2;do{i=s(i)%t,r=s(s(r))%t,n=e(Math.abs(i-r),t)}while(1===n);return n}(i.range):0;if(n&&h(t,1,i)>=s)i.step=1;else if(n&&a<i.step&&h(t,a,i)>=s)i.step=a;else{let e=0;for(;;){if(o&&h(t,i.step,i)<=s)i.step*=2;else{if(o||!(h(t,i.step/2,i)>=s))break;if(i.step/=2,n&&i.step%1!=0){i.step*=2;break}}if(e++>1e3)throw new Error("Exceeded maximum number of iterations while optimizing scale step!")}}function u(t,e){return t===(t+=e)&&(t*=1+(e>0?l:-l)),t}i.step=Math.max(i.step,l);let d=i.min,p=i.max;for(;d+i.step<=i.low;)d=u(d,i.step);for(;p-i.step>=i.high;)p=u(p,-i.step);i.min=d,i.max=p,i.range=i.max-i.min;const m=[];for(let t=i.min;t<=i.max;t=u(t,i.step)){const e=c(t);e!==m[m.length-1]&&m.push(e)}return i.values=m,i}(s[t.rectEnd]-s[t.rectStart],i,n.scaleMinSpace||20,n.onlyInteger),o={min:r.min,max:r.max};super(t,s,r.values),this.bounds=r,this.range=o}}class G extends T{projectValue(t,e){return this.stepLength*e}constructor(t,e,s,n){const i=n.ticks||[];super(t,s,i);const r=Math.max(1,i.length-(n.stretch?1:0));this.stepLength=this.axisLength/r,this.stretch=Boolean(n.stretch)}}function V(t,e,s){var n;if(m(t,"name")&&t.name&&(null===(n=e.series)||void 0===n?void 0:n[t.name])){const n=(null==e?void 0:e.series[t.name])[s];return void 0===n?e[s]:n}return e[s]}const Q={axisX:{offset:30,position:"end",labelOffset:{x:0,y:0},showLabel:!0,showGrid:!0,labelInterpolationFnc:d,type:void 0},axisY:{offset:40,position:"start",labelOffset:{x:0,y:0},showLabel:!0,showGrid:!0,labelInterpolationFnc:d,type:void 0,scaleMinSpace:20,onlyInteger:!1},width:void 0,height:void 0,showLine:!0,showPoint:!0,showArea:!1,areaBase:0,lineSmooth:!0,showGridBackground:!1,low:void 0,high:void 0,chartPadding:{top:15,right:15,bottom:5,left:10},fullWidth:!1,reverseData:!1,classNames:{chart:"ct-chart-line",label:"ct-label",labelGroup:"ct-labels",series:"ct-series",line:"ct-line",point:"ct-point",area:"ct-area",grid:"ct-grid",gridGroup:"ct-grids",gridBackground:"ct-grid-background",vertical:"ct-vertical",horizontal:"ct-horizontal",start:"ct-start",end:"ct-end"}};class F extends B{createChart(t){const{data:e}=this,s=function(t){let e,s=arguments.length>1&&void 0!==arguments[1]&&arguments[1],n=arguments.length>2?arguments[2]:void 0,i=arguments.length>3?arguments[3]:void 0;const r={labels:(t.labels||[]).slice(),series:E(t.series,n,i)},o=r.labels.length;return function(t){return!!Array.isArray(t)&&t.every(Array.isArray)}(r.series)?(e=Math.max(o,...r.series.map((t=>t.length))),r.series.forEach((t=>{t.push(...p(Math.max(0,e-t.length)))}))):e=r.series.length,r.labels.push(...p(Math.max(0,e-o),(()=>""))),s&&function(t){var e;null===(e=t.labels)||void 0===e||e.reverse(),t.series.reverse();for(const e of t.series)m(e,"data")?e.data.reverse():Array.isArray(e)&&e.reverse()}(r),r}(e,t.reverseData,!0),i=function(t){let e=arguments.length>1&&void 0!==arguments[1]?arguments[1]:"100%",s=arguments.length>2&&void 0!==arguments[2]?arguments[2]:"100%",i=arguments.length>3?arguments[3]:void 0;Array.from(t.querySelectorAll("svg")).filter((t=>t.getAttributeNS(n.xmlns,"ct"))).forEach((e=>t.removeChild(e)));const r=new L("svg").attr({width:e,height:s}).attr({style:"width: ".concat(e,"; height: ").concat(s,";")});return i&&r.addClass(i),t.appendChild(r.getNode()),r}(this.container,t.width,t.height,t.classNames.chart);this.svg=i;const r=i.elem("g").addClass(t.classNames.gridGroup),o=i.elem("g"),l=i.elem("g").addClass(t.classNames.labelGroup),h=function(t,e){var s,n,i,r;const o=Boolean(e.axisX||e.axisY),l=(null===(s=e.axisY)||void 0===s?void 0:s.offset)||0,h=(null===(n=e.axisX)||void 0===n?void 0:n.offset)||0,c=null===(i=e.axisY)||void 0===i?void 0:i.position,u=null===(r=e.axisX)||void 0===r?void 0:r.position;let d=t.width()||a(e.width).value||0,p=t.height()||a(e.height).value||0;const m="number"==typeof(f=e.chartPadding)?{top:f,right:f,bottom:f,left:f}:void 0===f?{top:0,right:0,bottom:0,left:0}:{top:"number"==typeof f.top?f.top:0,right:"number"==typeof f.right?f.right:0,bottom:"number"==typeof f.bottom?f.bottom:0,left:"number"==typeof f.left?f.left:0};var f;d=Math.max(d,l+m.left+m.right),p=Math.max(p,h+m.top+m.bottom);const g={x1:0,x2:0,y1:0,y2:0,padding:m,width(){return this.x2-this.x1},height(){return this.y1-this.y2}};return o?("start"===u?(g.y2=m.top+h,g.y1=Math.max(p-m.bottom,g.y2+1)):(g.y2=m.top,g.y1=Math.max(p-m.bottom-h,g.y2+1)),"start"===c?(g.x1=m.left+l,g.x2=Math.max(d-m.right,g.x1+1)):(g.x1=m.left,g.x2=Math.max(d-m.right-l,g.x1+1))):(g.x1=m.left,g.x2=Math.max(d-m.right,g.x1+1),g.y2=m.top,g.y1=Math.max(p-m.bottom,g.y2+1)),g}(i,t);let c,u;c=void 0===t.axisX.type?new G(Y.x,s.series,h,{...t.axisX,ticks:s.labels,stretch:t.fullWidth}):new t.axisX.type(Y.x,s.series,h,t.axisX),u=void 0===t.axisY.type?new U(Y.y,s.series,h,{...t.axisY,high:f(t.high)?t.high:t.axisY.high,low:f(t.low)?t.low:t.axisY.low}):new t.axisY.type(Y.y,s.series,h,t.axisY),c.createGridAndLabels(r,l,t,this.eventEmitter),u.createGridAndLabels(r,l,t,this.eventEmitter),t.showGridBackground&&function(t,e,s,n){const i=t.elem("rect",{x:e.x1,y:e.y2,width:e.width(),height:e.height()},s,!0);n.emit("draw",{type:"gridBackground",group:t,element:i})}(r,h,t.classNames.gridBackground,this.eventEmitter),function(t,e){let s=0;t[arguments.length>2&&void 0!==arguments[2]&&arguments[2]?"reduceRight":"reduce"](((t,n,i)=>e(n,s++,i)),void 0)}(e.series,((e,n)=>{const i=o.elem("g"),r=m(e,"name")&&e.name,a=m(e,"className")&&e.className,l=m(e,"meta")?e.meta:void 0;var d;r&&i.attr({"ct:series-name":r}),l&&i.attr({"ct:meta":A(l)}),i.addClass([t.classNames.series,a||"".concat(t.classNames.series,"-").concat((d=n,String.fromCharCode(97+d%26)))].join(" "));const p=[],g=[];s.series[n].forEach(((t,i)=>{const r={x:h.x1+c.projectValue(t,i,s.series[n]),y:h.y1-u.projectValue(t,i,s.series[n])};p.push(r.x,r.y),g.push({value:t,valueIndex:i,meta:v(e,i)})}));const x={lineSmooth:V(e,t,"lineSmooth"),showPoint:V(e,t,"showPoint"),showLine:V(e,t,"showLine"),showArea:V(e,t,"showArea"),areaBase:V(e,t,"areaBase")};let y;y="function"==typeof x.lineSmooth?x.lineSmooth:x.lineSmooth?P():z();const w=y(p,g);if(x.showPoint&&w.pathElements.forEach((s=>{const{data:r}=s,o=i.elem("line",{x1:s.x,y1:s.y,x2:s.x+.01,y2:s.y},t.classNames.point);if(r){let t,e;m(r.value,"x")&&(t=r.value.x),m(r.value,"y")&&(e=r.value.y),o.attr({"ct:value":[t,e].filter(f).join(","),"ct:meta":A(r.meta)})}this.eventEmitter.emit("draw",{type:"point",value:null==r?void 0:r.value,index:(null==r?void 0:r.valueIndex)||0,meta:null==r?void 0:r.meta,series:e,seriesIndex:n,axisX:c,axisY:u,group:i,element:o,x:s.x,y:s.y,chartRect:h})})),x.showLine){const r=i.elem("path",{d:w.stringify()},t.classNames.line,!0);this.eventEmitter.emit("draw",{type:"line",values:s.series[n],path:w.clone(),chartRect:h,index:n,series:e,seriesIndex:n,meta:l,axisX:c,axisY:u,group:i,element:r})}if(x.showArea&&u.range){const r=Math.max(Math.min(x.areaBase,u.range.max),u.range.min),o=h.y1-u.projectValue(r);w.splitByCommand("M").filter((t=>t.pathElements.length>1)).map((t=>{const e=t.pathElements[0],s=t.pathElements[t.pathElements.length-1];return t.clone(!0).position(0).remove(1).move(e.x,o).line(e.x,e.y).position(t.pathElements.length+1).line(s.x,o)})).forEach((r=>{const o=i.elem("path",{d:r.stringify()},t.classNames.area,!0);this.eventEmitter.emit("draw",{type:"area",values:s.series[n],path:r.clone(),series:e,seriesIndex:n,axisX:c,axisY:u,chartRect:h,index:n,group:i,element:o,meta:l})}))}}),t.reverseData),this.eventEmitter.emit("created",{chartRect:h,axisX:c,axisY:u,svg:i,options:t})}constructor(t,e,s,n){super(t,e,Q,u({},Q,s),n),this.data=e}}function H(t){const e={series:[{name:"series-1",data:t.values},{name:"series-2",data:t.floor},{name:"series-3",data:t.gapped},{name:"series-4",data:t.first},{name:"series-5",data:t.second}]};new F("#my-chart",e,{fullWidth:!0,height:"500px",series:{"series-1":{showLine:!0,showPoint:!1},"series-2":{showPoint:!1},"series-3":{showLine:!1,showPoint:!0},"series-4":{showLine:!1,showPoint:!0},"series-5":{showLine:!1,showPoint:!0}}}).on("draw",(function(t){if("point"===t.type){const e=new L("path",{d:["M",t.x,t.y-5,"L",t.x-5,t.y+3,"L",t.x+5,t.y+3,"z"].join(" "),style:"fill-opacity: 1"},"ct-area");t.element.replace(e)}}))}}},e={};function s(n){var i=e[n];if(void 0!==i)return i.exports;var r=e[n]={exports:{}};return t[n](r,r.exports,s),r.exports}s.d=(t,e)=>{for(var n in e)s.o(e,n)&&!s.o(t,n)&&Object.defineProperty(t,n,{enumerable:!0,get:e[n]})},s.o=(t,e)=>Object.prototype.hasOwnProperty.call(t,e),s.r=t=>{"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(t,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(t,"__esModule",{value:!0})},s(412);var n=s(631);Server=n})();