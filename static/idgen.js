Vue.createApp({
  data: function () {
    return {
      loading: false,
      errorMessage: '',
      metrics: [],
      records: [],
      lastText: '',
      form: {
        count: 5,
        gender: 'any',
        region: '',
        birth: '',
        min_birth: '1970-01-01',
        max_birth: '2010-12-31'
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

    generateIds: async function () {
      this.loading = true;
      this.errorMessage = '';
      this.records = [];
      this.metrics = [];
      this.lastText = '';

      try {
        var response = await fetch('/api/idgen', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(this.normalizedPayload())
        });
        var text = await response.text();
        var data = text ? JSON.parse(text) : {};
        if (!response.ok) {
          throw new Error(data.error || 'request failed');
        }

        this.records = data.records || [];
        this.metrics = [
          { label: '生成数量', value: this.records.length },
          { label: '地区模式', value: this.form.region || '随机' },
          { label: '性别', value: this.genderLabel(this.form.gender) },
          { label: '结果格式', value: '结构化数据' }
        ];
        this.lastText = this.records.map(function (row) {
          return row.name + '\t' + row.id_number + '\t' + row.address;
        }).join('\n');
      } catch (error) {
        this.errorMessage = error.message || String(error);
      } finally {
        this.loading = false;
      }
    },

    copyResult: async function () {
      if (!this.lastText) return;
      await navigator.clipboard.writeText(this.lastText);
    },

    genderLabel: function (value) {
      if (value === 'male') return '男';
      if (value === 'female') return '女';
      return '随机';
    }
  }
}).mount('#app');
