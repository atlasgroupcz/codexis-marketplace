(function () {
  'use strict';

  var form = document.getElementById('query-form');
  var uuidInput = document.getElementById('uuid-input');
  var loadOverviewButton = document.getElementById('load-overview-button');
  var statusEl = document.getElementById('status');
  var overviewBody = document.getElementById('overview-body');
  var overviewEmpty = document.getElementById('overview-empty');
  var jsonOutput = document.getElementById('json-output');

  if (
    !form ||
    !uuidInput ||
    !loadOverviewButton ||
    !statusEl ||
    !overviewBody ||
    !overviewEmpty ||
    !jsonOutput
  ) {
    return;
  }

  function setStatus(message, kind) {
    statusEl.textContent = message || '';
    statusEl.className = 'status';
    if (kind) {
      statusEl.classList.add('status-' + kind);
    }
  }

  function escapeHtml(value) {
    if (value === null || value === undefined) {
      return '';
    }
    return String(value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function buildEndpoint(uuid) {
    var endpoint = new URL('./', window.location.href);
    if (uuid) {
      endpoint.searchParams.set('uuid', uuid);
    }
    return endpoint.toString();
  }

  function updateBrowserUrl(uuid) {
    var nextUrl = new URL(window.location.href);
    if (uuid) {
      nextUrl.searchParams.set('uuid', uuid);
    } else {
      nextUrl.searchParams.delete('uuid');
    }
    window.history.replaceState(null, '', nextUrl.toString());
  }

  function clearOverview() {
    overviewBody.innerHTML = '';
    overviewEmpty.style.display = 'block';
  }

  function renderOverviewRows(items) {
    overviewBody.innerHTML = '';

    if (!Array.isArray(items) || items.length === 0) {
      overviewEmpty.style.display = 'block';
      return;
    }

    overviewEmpty.style.display = 'none';

    items.forEach(function (item) {
      var row = document.createElement('tr');
      row.innerHTML =
        '<td><button type="button" class="uuid-link" data-uuid="' + escapeHtml(item.uuid) + '">' +
        escapeHtml(item.uuid) +
        '</button></td>' +
        '<td>' +
        '<div class="doc-name">' + escapeHtml(item.name) + '</div>' +
        '<div class="doc-id">' + escapeHtml(item.codexisId) + '</div>' +
        '</td>' +
        '<td>' + escapeHtml(item.unconfirmed_changes) + '</td>' +
        '<td>' + escapeHtml(item.total_changes) + '</td>';
      overviewBody.appendChild(row);
    });
  }

  async function loadData(uuid) {
    var endpoint = buildEndpoint(uuid);
    setStatus('Loading ' + endpoint, 'info');

    try {
      var response = await fetch(endpoint, {
        headers: {
          Accept: 'application/json',
        },
        cache: 'no-store',
      });
      var text = await response.text();
      var payload = {};
      try {
        payload = JSON.parse(text);
      } catch (error) {
        throw new Error('Response is not valid JSON.');
      }

      jsonOutput.textContent = JSON.stringify(payload, null, 2);

      if (!response.ok) {
        clearOverview();
        setStatus('Request failed with HTTP ' + response.status + '.', 'error');
        return;
      }

      if (payload.mode === 'overview') {
        renderOverviewRows(payload.tracked_documents);
        setStatus('Overview loaded.', 'success');
      } else if (payload.mode === 'detail') {
        clearOverview();
        var documentData = payload.document || {};
        setStatus(
          'Detail loaded for ' +
            (documentData.uuid ? documentData.uuid : 'requested uuid') +
            '.',
          'success',
        );
      } else {
        clearOverview();
        setStatus('Unknown response mode.', 'error');
      }
    } catch (error) {
      clearOverview();
      jsonOutput.textContent = '';
      setStatus(error.message || 'Failed to load data.', 'error');
    }
  }

  form.addEventListener('submit', function (event) {
    event.preventDefault();
    var uuid = uuidInput.value.trim();
    updateBrowserUrl(uuid);
    void loadData(uuid);
  });

  loadOverviewButton.addEventListener('click', function () {
    uuidInput.value = '';
    updateBrowserUrl('');
    void loadData('');
  });

  overviewBody.addEventListener('click', function (event) {
    var target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }
    if (!target.classList.contains('uuid-link')) {
      return;
    }
    var uuid = target.getAttribute('data-uuid') || '';
    uuidInput.value = uuid;
    updateBrowserUrl(uuid);
    void loadData(uuid);
  });

  function initialize() {
    var initialUuid = new URL(window.location.href).searchParams.get('uuid') || '';
    uuidInput.value = initialUuid;
    void loadData(initialUuid);
  }

  initialize();
})();
