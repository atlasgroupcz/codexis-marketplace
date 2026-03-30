(function () {
  const form = document.getElementById('echo-form');
  const input = document.getElementById('inputText');
  const output = document.getElementById('echo-output');
  const jsonPreview = document.getElementById('json-preview');

  if (form && input && output) {
    input.addEventListener('input', function () {
      output.textContent = input.value.trim() || '(empty)';
    });
  }

  fetch('./assets/sample-data.json')
    .then(function (response) {
      if (!response.ok) {
        throw new Error('HTTP ' + response.status);
      }
      return response.json();
    })
    .then(function (data) {
      if (jsonPreview) {
        jsonPreview.textContent = JSON.stringify(data, null, 2);
      }
    })
    .catch(function (error) {
      if (jsonPreview) {
        jsonPreview.textContent = 'Failed loading ./assets/sample-data.json: ' + error.message;
      }
    });
})();
