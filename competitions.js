const competitionStage = document.querySelector('.competition-stage');
document.querySelector('#competitionsSection').append(competitionStage);
const badgePhotos = ['assets/ai_post_ecosystem.png', 'assets/sumo_robot.png', 'assets/nexusai_phone.png'];
document.querySelectorAll('.competition-card').forEach((card, index) => {
  const photo = document.createElement('img');
  photo.className = 'card-photo'; photo.src = badgePhotos[index]; photo.alt = '';
  card.querySelector('.card-icon').after(photo);
  const barcode = document.createElement('span'); barcode.className = 'badge-barcode'; barcode.setAttribute('aria-hidden', 'true');
  card.querySelector('.card-hint').before(barcode);
  let startX = 0, startY = 0, moved = false, dragging = false, frame = 0, x = 0, y = 0, angle = 0;
  const paint = () => { card.style.setProperty('--pull-x', `${x}px`); card.style.setProperty('--pull-y', `${y}px`); card.style.setProperty('--swing', `${angle}deg`); card.style.setProperty('--stretch', `${Math.max(0, y)}px`); };
  card.addEventListener('pointerdown', e => { cancelAnimationFrame(frame); startX = e.clientX; startY = e.clientY; moved = false; dragging = true; card.setPointerCapture(e.pointerId); card.classList.add('is-dragging'); });
  card.addEventListener('pointermove', e => {
    if (!dragging) return;
    const dx = e.clientX - startX, dy = e.clientY - startY;
    if (Math.hypot(dx, dy) > 6) moved = true;
    x = Math.max(-65, Math.min(65, dx * .6)); y = Math.max(-12, Math.min(65, dy * .65)); angle = Math.max(-18, Math.min(18, dx * .16)); paint();
  });
  const release = () => {
    if (!dragging) return; dragging = false; card.classList.remove('is-dragging');
    let vx = 0, vy = 0, va = 0, last;
    const tick = now => { const dt = Math.min((now - (last || now)) / 1000, .032); last = now;
      vx += (-75*x-11*vx)*dt; vy += (-75*y-11*vy)*dt; va += (-48*angle-7*va)*dt;
      x += vx*dt; y += vy*dt; angle += va*dt; paint();
      if (Math.abs(x)+Math.abs(y)+Math.abs(angle)+Math.abs(vx)+Math.abs(vy)+Math.abs(va) > .2) frame = requestAnimationFrame(tick);
      else { x = y = angle = 0; paint(); }
    }; frame = requestAnimationFrame(tick);
  };
  card.addEventListener('pointerup', release);
  card.addEventListener('pointercancel', release);
  card.addEventListener('click', () => {
    if (moved) { moved = false; return; }
    const flipped = card.classList.toggle('is-flipped');
    card.setAttribute('aria-pressed', String(flipped));
  });
});

let deckStart = null;
const deckArea = document.querySelector('.deck-outer-container');
deckArea.addEventListener('pointerdown', e => { if (e.pointerType === 'mouse') deckStart = e.clientX; });
window.addEventListener('pointerup', e => {
  if (deckStart === null) return;
  const delta = deckStart - e.clientX; deckStart = null;
  if (Math.abs(delta) > 45) { activeIndex = Math.max(0, Math.min(projects.length - 1, activeIndex + (delta > 0 ? 1 : -1))); updateDeckLayout(); }
});
document.querySelector('.scroll-instruction > span:last-child').textContent = '← Projeleri kaydır →  ·  Yarışmalar için aşağı ↓';
document.querySelector('.section-desc').textContent = 'Projeleri sağa sola kaydır; yarışmalar için aşağı in.';
