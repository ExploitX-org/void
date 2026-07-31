document.addEventListener('DOMContentLoaded', () => {
  const snapSections = () =>
    Array.from(document.querySelectorAll('main > section, footer'))
      .sort((a, b) => a.offsetTop - b.offsetTop);

  function scrollToSection(dir) {
    const sections = snapSections();
    const cur = window.scrollY;
    let target = null;
    if (dir === 1) {
      for (const s of sections) {
        if (s.offsetTop > cur + 5) { target = s; break; }
      }
    } else {
      for (let i = sections.length - 1; i >= 0; i--) {
        if (sections[i].offsetTop < cur - 5) { target = sections[i]; break; }
      }
    }
    if (target) target.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }

  window.addEventListener('keydown', (e) => {
    if (['ArrowDown', 'ArrowUp', 'PageDown', 'PageUp', ' '].includes(e.key)) {
      const tag = document.activeElement?.tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea') return;
      e.preventDefault();
      const dir = (e.key === 'ArrowUp' || e.key === 'PageUp') ? -1 : 1;
      scrollToSection(dir);
    }
  }, { passive: false });
});
