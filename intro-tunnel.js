(() => {
  const intro = document.getElementById('introSection');
  const plane = intro.querySelector('.intro-tunnel-plane');
  const copy = document.getElementById('typewriterText');
  const images = [
    'c3/c3fbd2afa140223999a441b32ed986f61d473930002708df805a81532a326aac.jpg',
    '9d/9d8b2f675a5814476ada3736ce439bcda23c5c389e0ebf9b25d8a0c09f3bbcfe.jpg',
    '8b/8bb738bf2f00d7f7ed3c3ec034adb9938ad38f4be4ac191d632a6a11ad525574.jpg',
    'ac/ac5ac700b188d7ef16ee9dab868098b59f15c440db3f71d4e8c03262a3287577.jpg',
    'ef/ef25eff6ce655fba459b3b7e7711524dc92499aa18f6c7dbea9a0fcc56a8f908.jpg',
    'f2/f28b370a5dcbf76240f1504579c4b05fc0ff0bb990a42aff8dc2f73a5dc46072.jpg',
    'f2/f26a1eac85704dda924820392dedfb72d51d8261d35354f83b35d683617295f3.jpg',
    '77/7755d23eaaa0aa5b403343f44afa7a29b2d06d3065fe54c602b14ba01d00f2aa.jpg',
    '65/657317761fe9058a1d43711b13c543a71eb5199b7a5d9abf4c28479674e39cf7.jpg'
  ];
  const titles = ['NexusAI PhoneAgent','AI-POST','NexusAI Agent','Ven-s Finder','FBMTAL Chat','Mini Sumo Robot','Hype News','StartupCMD','Web OS'];
  const cards = images.map((src, i) => { const card = document.createElement('div'); card.className = 'intro-tunnel-card'; const img = document.createElement('img'); img.src = `https://cdn.21st.dev/assets/mirror/${src}`; img.alt = ''; img.draggable = false; const label = document.createElement('span'); label.textContent = titles[i]; card.append(img,label); plane.append(card); return card; });
  const lines = introLines.map((line,i) => { const el = document.createElement('span'); el.className = 'intro-line'; el.textContent = line; el.classList.toggle('is-current',i===0); copy.append(el); return el; });
  const glitches = intro.querySelector('.intro-glitches');
  for (let i=0;i<22;i++) { const el=document.createElement('i'); el.className='glitch-square'; el.style.cssText=`--left:${(i*37+8)%96}%;--top:${(i*61+11)%92}%;--size:${4+(i%4)*3}px;--duration:${2+(i%5)*.7}s;--delay:-${i*.37}s`; glitches.append(el); }
  let selected=0, target=.18, value=.18, pointerX=0, pointerY=0, lastStep=0;
  const reduced=window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  function draw() {
    value += (target-value) * (reduced ? 1 : .075);
    const mobile=innerWidth<768, w=intro.clientWidth, h=intro.clientHeight;
    cards.forEach((card,i) => { const start=(i/cards.length)*.45, p=Math.max(0,Math.min(1,(value-start)/.55)); const a=i*137.5*Math.PI/180, rx=.28+(i%3)*.08, ry=.26+(i%2)*.09; const z=(mobile?-1200:-1700)+p*(mobile?1550:2200); const x=Math.cos(a)*w*rx-pointerX*(mobile?20:45); const y=Math.sin(a)*h*ry-pointerY*(mobile?20:45); const rotation=(i%2===0?1:-1)*(3+(i%5)*2); card.style.width=`${mobile?Math.max(130,Math.min(180,w*.38)):Math.max(180,Math.min(270,w*.18))}px`; card.style.transform=`translate3d(calc(-50% + ${x}px),calc(-50% + ${y}px),${z}px) rotateZ(${rotation}deg)`; card.style.opacity=String(Math.max(0,Math.min(1,p/.18,(.98-p)/.16))); });
    requestAnimationFrame(draw);
  }
  requestAnimationFrame(draw);
  function step(direction) { const now=Date.now(); if(now-lastStep<450) return; lastStep=now; if(direction>0 && selected===lines.length-1){triggerTransition();return;} selected=Math.max(0,Math.min(lines.length-1,selected+direction)); lines.forEach((line,i)=>line.classList.toggle('is-current',i===selected)); target=.18+selected/(lines.length-1)*.8; }
  intro.addEventListener('wheel',e=>{if(intro.classList.contains('dismissed'))return;e.preventDefault();e.stopPropagation();if(Math.abs(e.deltaY)>3)step(e.deltaY>0?1:-1);},{passive:false,capture:true});
  let touchY=0; intro.addEventListener('touchstart',e=>{touchY=e.touches[0].clientY;},{passive:true,capture:true}); intro.addEventListener('touchend',e=>{if(intro.classList.contains('dismissed'))return;const d=touchY-e.changedTouches[0].clientY;if(Math.abs(d)>35){e.stopPropagation();step(d>0?1:-1);}},{capture:true});
  intro.addEventListener('pointermove',e=>{if(!reduced){pointerX=e.clientX/innerWidth*2-1;pointerY=e.clientY/innerHeight*2-1;}});
  const prompt=intro.querySelector('.scroll-text'); prompt.textContent='Kaydırarak keşfet';
})();
