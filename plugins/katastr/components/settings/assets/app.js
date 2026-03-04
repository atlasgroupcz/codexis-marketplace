(function () {
  var form = document.getElementById('api-key-form');
  var input = document.getElementById('apiKey');
  var statusEl = document.getElementById('status');
  var messageEl = document.getElementById('message');

  function showMessage(text, type) {
    messageEl.textContent = text;
    messageEl.className = 'message ' + type;
  }

  function loadStatus() {
    fetch('?status', { headers: { 'Accept': 'application/json' } })
      .then(function (r) { return r.json(); })
      .then(function (data) {
        if (data.configured) {
          statusEl.textContent = 'Aktuální klíč: ' + data.maskedKey;
        } else {
          statusEl.textContent = 'API klíč není nastaven.';
        }
      })
      .catch(function () {
        statusEl.textContent = 'Nepodařilo se načíst stav.';
      });
  }

  form.addEventListener('submit', function (e) {
    e.preventDefault();
    var key = input.value.trim();
    if (!key) {
      showMessage('Zadejte API klíč.', 'error');
      return;
    }
    fetch('', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: JSON.stringify({ apiKey: key })
    })
      .then(function (r) { return r.json(); })
      .then(function (data) {
        if (data.saved) {
          showMessage('Klíč uložen.', 'success');
          input.value = '';
          loadStatus();
        } else {
          showMessage(data.error || 'Nepodařilo se uložit.', 'error');
        }
      })
      .catch(function () {
        showMessage('Chyba při ukládání.', 'error');
      });
  });

  loadStatus();
})();
