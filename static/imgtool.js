Vue.createApp({
  data: function () {
    return {
      loading: false,
      errorMessage: '',
      files: [],
      dragOver: false,
      results: [],
      metrics: [],
      toastMessage: '',
      toastType: 'success',
      toastTimer: null,
      progress: { current: 0, total: 0 },
      previewUrl: '',
      previewName: '',
      previewSizeText: '',
      previewFile: null,
      previewResult: null,
      fileObjectUrls: {},
      lightboxUrl: '',
      lightboxName: '',
      lightboxSize: '',
      form: {
        formats: ['png']
      },
      formatOptions: [
        { value: 'png', label: 'PNG' },
        { value: 'jpg', label: 'JPG' },
        { value: 'webp', label: 'WebP' },
        { value: 'bmp', label: 'BMP' },
        { value: 'gif', label: 'GIF' },
        { value: 'tiff', label: 'TIFF' },
        { value: 'ico', label: 'ICO' }
      ]
    };
  },

  computed: {
    successResults: function () {
      return this.results.filter(function (r) { return !r.error; });
    }
  },

  methods: {
    triggerFileInput: function () {
      this.$refs.fileInput.click();
    },

    triggerFolderInput: function () {
      this.$refs.folderInput.click();
    },

    onFileChange: function (event) {
      var inputFiles = event.target.files || [];
      this.addFiles(inputFiles);
      event.target.value = '';
    },

    onFolderChange: function (event) {
      var inputFiles = event.target.files || [];
      var imageFiles = [];
      for (var i = 0; i < inputFiles.length; i++) {
        var file = inputFiles[i];
        if (file.type.startsWith('image/') || this.isImageByExt(file.name)) {
          imageFiles.push(file);
        }
      }
      if (imageFiles.length > 0) {
        this.addFiles(imageFiles);
        this.notify('已添加 ' + imageFiles.length + ' 个文件（来自文件夹）', 'success');
      } else {
        this.notify('文件夹中未找到图片文件', 'warn');
      }
      event.target.value = '';
    },

    onDrop: function (event) {
      this.dragOver = false;
      var items = event.dataTransfer.items || [];
      var allFiles = [];
      var app = this;

      function collectFiles(entry) {
        return new Promise(function (resolve) {
          if (entry.isFile) {
            entry.file(function (file) {
              allFiles.push(file);
              resolve();
            });
          } else if (entry.isDirectory) {
            var dirReader = entry.createReader();
            readAllEntries(dirReader, function (entries) {
              var promises = [];
              for (var i = 0; i < entries.length; i++) {
                promises.push(collectFiles(entries[i]));
              }
              Promise.all(promises).then(resolve);
            });
          } else {
            resolve();
          }
        });
      }

      function readAllEntries(dirReader, callback) {
        var allEntries = [];
        function readBatch() {
          dirReader.readEntries(function (entries) {
            if (entries.length === 0) {
              callback(allEntries);
            } else {
              allEntries = allEntries.concat(Array.prototype.slice.call(entries));
              readBatch();
            }
          });
        }
        readBatch();
      }

      var promises = [];
      for (var i = 0; i < items.length; i++) {
        var entry = items[i].webkitGetAsEntry ? items[i].webkitGetAsEntry() : null;
        if (entry) {
          promises.push(collectFiles(entry));
        }
      }

      if (promises.length > 0) {
        Promise.all(promises).then(function () {
          app.addFiles(allFiles);
        });
      } else {
        var droppedFiles = event.dataTransfer.files || [];
        this.addFiles(droppedFiles);
      }
    },

    addFiles: function (newFiles) {
      var added = 0;
      for (var i = 0; i < newFiles.length; i++) {
        var file = newFiles[i];
        if (file.type.startsWith('image/') || this.isImageByExt(file.name)) {
          // 去重：按 name + size 判断
          var exists = false;
          for (var j = 0; j < this.files.length; j++) {
            if (this.files[j].name === file.name && this.files[j].size === file.size) {
              exists = true;
              break;
            }
          }
          if (!exists) {
            this.files.push(file);
            added++;
          }
        }
      }
      if (added > 0) {
        this.notify('已添加 ' + added + ' 个文件', 'success');
      }
    },

    isImageByExt: function (name) {
      var ext = name.split('.').pop().toLowerCase();
      var imageExts = ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif', 'tiff', 'tif', 'ico'];
      return imageExts.indexOf(ext) !== -1;
    },

    removeFile: function (index) {
      var file = this.files[index];
      var key = file.name + '_' + file.size;
      if (this.fileObjectUrls[key]) {
        URL.revokeObjectURL(this.fileObjectUrls[key]);
        delete this.fileObjectUrls[key];
      }
      if (this.previewFile === file) {
        this.clearPreview();
      }
      this.files.splice(index, 1);
    },

    clearFiles: function () {
      // 清理预览相关
      this.clearPreview();
      for (var key in this.fileObjectUrls) {
        URL.revokeObjectURL(this.fileObjectUrls[key]);
      }
      this.fileObjectUrls = {};

      this.files = [];
      this.results = [];
      this.errorMessage = '';
      this.metrics = [];
      this.progress = { current: 0, total: 0 };
    },

    previewUpload: function (file) {
      if (this.previewFile === file) return;
      this.previewResult = null;
      this.previewFile = file;

      if (!this.fileObjectUrls[file.name + '_' + file.size]) {
        this.fileObjectUrls[file.name + '_' + file.size] = URL.createObjectURL(file);
      }
      this.previewUrl = this.fileObjectUrls[file.name + '_' + file.size];
      this.previewName = file.name;
      this.previewSizeText = this.formatSize(file.size);
    },

    previewResultItem: function (item) {
      if (item.error) return;
      if (this.previewResult === item) return;
      this.previewFile = null;
      this.previewResult = item;
      this.previewUrl = item.url;
      this.previewName = this.getOutputName(item.source, item.format);
      this.previewSizeText = this.formatSize(item.size);
    },

    clearPreview: function () {
      this.previewUrl = '';
      this.previewName = '';
      this.previewSizeText = '';
      this.previewFile = null;
      this.previewResult = null;
    },

    openLightbox: function () {
      this.lightboxUrl = this.previewUrl;
      this.lightboxName = this.previewName;
      this.lightboxSize = this.previewSizeText;
    },

    closeLightbox: function () {
      this.lightboxUrl = '';
      this.lightboxName = '';
      this.lightboxSize = '';
    },

    startConvert: async function () {
      if (!this.files.length || !this.form.formats.length) return;

      this.loading = true;
      this.errorMessage = '';
      this.results = [];
      this.metrics = [];
      this.progress = { current: 0, total: this.files.length * this.form.formats.length };

      var allResults = [];
      var totalConverted = 0;
      var totalErrors = 0;

      try {
        for (var i = 0; i < this.files.length; i++) {
          var file = this.files[i];
          for (var j = 0; j < this.form.formats.length; j++) {
            var fmt = this.form.formats[j];
            try {
              var formData = new FormData();
              formData.append('file', file);
              formData.append('format', fmt);

              var response = await fetch('/api/imgtool/convert', {
                method: 'POST',
                body: formData
              });

              if (!response.ok) {
                var errText = await response.text();
                var errData = {};
                try { errData = JSON.parse(errText); } catch (e) {}
                throw new Error(errData.error || '请求失败');
              }

              var blob = await response.blob();
              var url = URL.createObjectURL(blob);

              allResults.push({
                source: file.name,
                format: fmt,
                blob: blob,
                url: url,
                size: blob.size
              });
              totalConverted++;
            } catch (err) {
              totalErrors++;
              allResults.push({
                source: file.name,
                format: fmt,
                error: err.message || String(err)
              });
            }
            this.progress.current++;
          }
        }

        this.results = allResults;
        this.metrics = [
          { label: '成功转换', value: String(totalConverted) + ' 个' },
          { label: '失败', value: String(totalErrors) + ' 个' },
          { label: '文件数', value: String(this.files.length) }
        ];

        if (totalErrors > 0) {
          this.notify('转换完成，' + totalConverted + ' 成功 / ' + totalErrors + ' 失败', 'error');
        } else {
          this.notify('全部转换完成（' + totalConverted + ' 个）', 'success');
        }
      } catch (error) {
        this.errorMessage = error.message || String(error);
        this.notify('转换失败', 'error');
      } finally {
        this.loading = false;
      }
    },

    downloadItem: function (item) {
      if (item.error) return;
      var a = document.createElement('a');
      a.href = item.url;
      a.download = this.getOutputName(item.source, item.format);
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
    },

    downloadAll: async function () {
      var items = this.successResults;
      if (!items.length) return;

      this.notify('正在下载 ' + items.length + ' 个文件...', 'success');

      for (var i = 0; i < items.length; i++) {
        var item = items[i];
        var a = document.createElement('a');
        a.href = item.url;
        a.download = this.getOutputName(item.source, item.format);
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        await this.sleep(200);
      }

      this.notify('全部下载完成', 'success');
    },

    getOutputName: function (source, format) {
      var name = source.replace(/\.[^.]+$/, '');
      return name + '.' + format;
    },

    formatSize: function (bytes) {
      if (!bytes && bytes !== 0) return '-';
      if (bytes < 1024) return String(bytes) + ' B';
      if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
      return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    },

    sleep: function (ms) {
      return new Promise(function (resolve) {
        window.setTimeout(resolve, ms);
      });
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
      }, 2000);
    }
  }
}).mount('#app');
