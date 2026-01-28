// Auto-detect theme and set CSS class
(function() {
  const theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  document.documentElement.classList.add(theme);
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', e => {
    document.documentElement.classList.remove('light', 'dark');
    document.documentElement.classList.add(e.matches ? 'dark' : 'light');
  });
})();
