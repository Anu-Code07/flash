// Flash site — particles, scroll reveal, counters, parallax

(function () {
  'use strict';

  // ── Particle canvas ──────────────────────────────────────────────────────
  function initParticles() {
    const canvas = document.getElementById('particle-canvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let w, h, particles, mouse = { x: -9999, y: -9999 };
    const prefersReduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (prefersReduced) return;

    function resize() {
      w = canvas.width = window.innerWidth;
      h = canvas.height = window.innerHeight;
    }

    function makeParticles(count) {
      return Array.from({ length: count }, () => ({
        x: Math.random() * w,
        y: Math.random() * h,
        r: Math.random() * 2 + 0.5,
        vx: (Math.random() - 0.5) * 0.4,
        vy: (Math.random() - 0.5) * 0.4,
        hue: Math.random() > 0.5 ? 250 : 175,
        alpha: Math.random() * 0.5 + 0.2,
      }));
    }

    resize();
    particles = makeParticles(Math.min(80, Math.floor((w * h) / 18000)));

    window.addEventListener('resize', () => {
      resize();
      particles = makeParticles(Math.min(80, Math.floor((w * h) / 18000)));
    });

    document.addEventListener('mousemove', (e) => {
      mouse.x = e.clientX;
      mouse.y = e.clientY;
    });

    function draw() {
      ctx.clearRect(0, 0, w, h);
      for (let i = 0; i < particles.length; i++) {
        const p = particles[i];
        p.x += p.vx;
        p.y += p.vy;
        if (p.x < 0) p.x = w;
        if (p.x > w) p.x = 0;
        if (p.y < 0) p.y = h;
        if (p.y > h) p.y = 0;

        const dx = mouse.x - p.x;
        const dy = mouse.y - p.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < 120) {
          p.x -= dx * 0.02;
          p.y -= dy * 0.02;
        }

        ctx.beginPath();
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fillStyle = `hsla(${p.hue}, 80%, 70%, ${p.alpha})`;
        ctx.fill();

        for (let j = i + 1; j < particles.length; j++) {
          const q = particles[j];
          const d = Math.hypot(p.x - q.x, p.y - q.y);
          if (d < 100) {
            ctx.beginPath();
            ctx.moveTo(p.x, p.y);
            ctx.lineTo(q.x, q.y);
            ctx.strokeStyle = `hsla(250, 60%, 60%, ${0.12 * (1 - d / 100)})`;
            ctx.lineWidth = 0.5;
            ctx.stroke();
          }
        }
      }
      requestAnimationFrame(draw);
    }
    draw();
  }

  // ── Scroll reveal ────────────────────────────────────────────────────────
  function initReveal() {
    const els = document.querySelectorAll('.reveal, .stagger-children');
    if (!els.length) return;
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) {
            e.target.classList.add('visible');
            io.unobserve(e.target);
          }
        });
      },
      { threshold: 0.12, rootMargin: '0px 0px -40px 0px' }
    );
    els.forEach((el) => io.observe(el));
  }

  // ── Animated counters ────────────────────────────────────────────────────
  function initCounters() {
    document.querySelectorAll('[data-count]').forEach((el) => {
      const target = parseInt(el.dataset.count, 10);
      const suffix = el.dataset.suffix || '';
      const duration = 1800;
      let started = false;

      const io = new IntersectionObserver(
        (entries) => {
          if (entries[0].isIntersecting && !started) {
            started = true;
            const start = performance.now();
            function tick(now) {
              const t = Math.min((now - start) / duration, 1);
              const eased = 1 - Math.pow(1 - t, 3);
              el.textContent = Math.floor(eased * target) + suffix;
              if (t < 1) requestAnimationFrame(tick);
            }
            requestAnimationFrame(tick);
            io.disconnect();
          }
        },
        { threshold: 0.5 }
      );
      io.observe(el);
    });
  }

  // ── Parallax orbs ────────────────────────────────────────────────────────
  function initParallax() {
    const orbs = document.querySelectorAll('.glow-orb');
    if (!orbs.length || window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    let ticking = false;
    window.addEventListener('scroll', () => {
      if (!ticking) {
        requestAnimationFrame(() => {
          const y = window.scrollY;
          orbs.forEach((orb, i) => {
            const speed = 0.08 + i * 0.04;
            orb.style.transform = `translateY(${y * speed}px)`;
          });
          ticking = false;
        });
        ticking = true;
      }
    });
  }

  // ── Typing effect for hero code ──────────────────────────────────────────
  function initTypewriter() {
    const el = document.getElementById('hero-typewriter');
    if (!el) return;
    const lines = [
      'screen Home {',
      '  state count: Int = 0',
      '  Column {',
      '    Text("Count: ${count}")',
      '    Button("Go") { count++ }',
      '  }',
      '}',
    ];
    let lineIdx = 0;
    let charIdx = 0;
    let html = '';

    function type() {
      if (lineIdx >= lines.length) {
        setTimeout(() => {
          lineIdx = 0;
          charIdx = 0;
          html = '';
          el.innerHTML = '';
          type();
        }, 4000);
        return;
      }
      const line = lines[lineIdx];
      if (charIdx < line.length) {
        html += line[charIdx] === ' ' ? '&nbsp;' : line[charIdx];
        charIdx++;
        el.innerHTML = html + '<span class="type-cursor">|</span>';
        setTimeout(type, 28 + Math.random() * 30);
      } else {
        html += '\n';
        lineIdx++;
        charIdx = 0;
        el.innerHTML = html.replace(/\n/g, '<br>') + '<span class="type-cursor">|</span>';
        setTimeout(type, 120);
      }
    }
    setTimeout(type, 800);
  }

  // ── Mobile nav ───────────────────────────────────────────────────────────
  function initMobileNav() {
    const toggle = document.querySelector('.nav-toggle');
    const links = document.querySelector('.nav-links');
    if (!toggle || !links) return;
    toggle.addEventListener('click', () => {
      links.classList.toggle('open');
      toggle.classList.toggle('open');
    });
  }

  // ── Pipeline SVG animation trigger ───────────────────────────────────────
  function initPipeline() {
    const svg = document.getElementById('pipeline-svg');
    if (!svg) return;
    const io = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          svg.classList.add('animate');
          io.disconnect();
        }
      },
      { threshold: 0.3 }
    );
    io.observe(svg);
  }

  document.addEventListener('DOMContentLoaded', () => {
    initParticles();
    initReveal();
    initCounters();
    initParallax();
    initTypewriter();
    initMobileNav();
    initPipeline();
  });
})();
