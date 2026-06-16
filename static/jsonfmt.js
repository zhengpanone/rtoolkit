Vue.createApp({
  data: function () {
    return {
      loading: false,
      errorMessage: '',
      output: '',
      copyStatus: '',
      toastMessage: '',
      toastType: 'success',
      toastTimer: null,
      metrics: [],
      form: {
        input: '{\n  "b": 2,\n  "a": 1\n}',
        indent: 2,
        sort: false,
        compact: false
      }
    };
  },

  computed: {
    inputStats: function () {
      var text = this.form.input || '';
      var lines = text ? text.split('\n').length : 0;
      return String(lines) + ' 行 / ' + String(text.length) + ' 字符';
    }
  },

  methods: {
    payload: function () {
      return {
        input: this.form.input,
        indent: Number(this.form.indent),
        sort: Boolean(this.form.sort),
        compact: Boolean(this.form.compact)
      };
    },

    formatJson: async function () {
      this.loading = true;
      this.errorMessage = '';
      this.output = '';
      this.copyStatus = '';
      this.metrics = [];

      try {
        var response = await fetch('/api/jsonfmt', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(this.payload())
        });
        var text = await response.text();
        var data = text ? JSON.parse(text) : {};
        if (!response.ok) {
          throw new Error(data.error || 'request failed');
        }

        this.output = data.output || '';
        this.metrics = [
          { label: '输出行数', value: data.lines || 0 },
          { label: '输出字节', value: data.bytes || 0 },
          { label: '缩进', value: this.form.compact ? 'compact' : String(this.form.indent) },
          { label: '排序', value: this.form.sort ? '已开启' : '保持原顺序' }
        ];
        this.notify('格式化完成', 'success');
      } catch (error) {
        this.errorMessage = error.message || String(error);
        this.notify('格式化失败', 'error');
      } finally {
        this.loading = false;
      }
    },

    copyResult: async function () {
      if (!this.output) return;
      try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          await navigator.clipboard.writeText(this.output);
        } else {
          this.copyWithTextarea(this.output);
        }
        this.copyStatus = '已复制';
        this.notify('结果已复制', 'success');
        var app = this;
        window.setTimeout(function () {
          app.copyStatus = '';
        }, 1600);
      } catch (error) {
        this.errorMessage = '复制失败: ' + (error.message || String(error));
        this.notify('复制失败', 'error');
      }
    },

    copyWithTextarea: function (text) {
      var textarea = document.createElement('textarea');
      textarea.value = text;
      textarea.setAttribute('readonly', 'readonly');
      textarea.style.position = 'fixed';
      textarea.style.left = '-9999px';
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
    },

    loadExample: function () {
      this.form.input = '{\n  "b": 2,\n  "a": {\n    "d": 4,\n    "c": 3\n  }\n}';
      this.form.indent = 2;
      this.form.sort = true;
      this.form.compact = false;
      this.notify('示例已载入', 'success');
    },

    clearAll: function () {
      this.form.input = '';
      this.output = '';
      this.copyStatus = '';
      this.errorMessage = '';
      this.metrics = [];
      this.notify('已清空', 'success');
    },

    notify: function (message, type) {
      this.toastMessage = message;
      this.toastType = type || 'success';
      if (this.toastTimer) {
        window.clearTimeout(this.toastTimer);
      }
      var app = this;
      this.toastTimer = window.setTimeout(function () {
        app.toastMessage = '';
        app.toastTimer = null;
      }, 1800);
    }
  }
}).mount('#app');
