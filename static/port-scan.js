Vue.createApp({
  data: function () {
    return {
      loading: false,
      errorMessage: '',
      metrics: [],
      result: { ports: [] },
      lastText: '',
      form: {
        target: '127.0.0.1',
        port: '80',
        concurrency: 100,
        timeout_ms: 1000
      }
    };
  },

  methods: {
    normalizedPayload: function () {
      var data = {};
      var form = this.form;
      Object.keys(form).forEach(function (key) {
        data[key] = form[key] === '' ? null : form[key];
      });
      return data;
    },

    scanPorts: async function () {
      this.loading = true;
      this.errorMessage = '';
      this.result = { ports: [] };
      this.metrics = [];
      this.lastText = '';

      try {
        var response = await fetch('/api/portscan', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(this.normalizedPayload())
        });
        var text = await response.text();
        var data = text ? JSON.parse(text) : {};
        if (!response.ok) {
          throw new Error(data.error || 'request failed');
        }

        this.result = data;
        this.metrics = [
          { label: '目标', value: data.target },
          { label: '扫描端口', value: data.total },
          { label: '开放端口', value: data.open_count },
          { label: '超时', value: String(data.timeout_ms) + 'ms' }
        ];
        this.lastText = JSON.stringify(data, null, 2);
      } catch (error) {
        this.errorMessage = error.message || String(error);
      } finally {
        this.loading = false;
      }
    }
  }
}).mount('#app');
