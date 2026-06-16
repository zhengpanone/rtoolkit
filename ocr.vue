<template>
  <div class="ocr-page">
    <div class="ocr-header">
      <div>
        <h2>PaddleOCR 识别工作台</h2>
        <p>支持 PP-OCRv5 通用识别与 PaddleOCR-VL 文档识别</p>
      </div>

      <div class="header-actions">
        <el-radio-group v-model="engine" size="small">
          <el-radio-button label="pp">PP-OCRv5</el-radio-button>
          <el-radio-button label="vl">PaddleOCR-VL</el-radio-button>
        </el-radio-group>

        <el-radio-group v-model="resultType" size="small">
          <el-radio-button label="json">结构化</el-radio-button>
          <el-radio-button label="md">Markdown</el-radio-button>
        </el-radio-group>

        <el-button type="primary" size="small" :loading="loading" :disabled="!currentFile" @click="handleRecognize">
          开始识别
        </el-button>
      </div>
    </div>

    <div class="status-strip">
      <div class="status-item">
        <span>当前服务</span>
        <strong>{{ serviceLabel }}</strong>
      </div>
      <div class="status-item">
        <span>接口地址</span>
        <strong>{{ requestUrl }}</strong>
      </div>
      <div class="status-item">
        <span>识别条数</span>
        <strong>{{ results.length }}</strong>
      </div>
      <div class="status-item">
        <span>平均置信度</span>
        <strong>{{ averageConfidenceText }}</strong>
      </div>
    </div>

    <div class="ocr-layout">
      <aside class="panel upload-panel">
        <div class="panel-title">文件</div>

        <el-upload drag action="" :auto-upload="false" :show-file-list="false"
          accept="image/png,image/jpeg,image/jpg,image/bmp" :on-change="handleFileChange">
          <i class="el-icon-upload"></i>
          <div class="el-upload__text">
            拖拽图片到这里，或 <em>点击上传</em>
          </div>
          <div slot="tip" class="el-upload__tip">
            支持 PNG / JPG / JPEG / BMP，建议单张小于 10MB
          </div>
        </el-upload>

        <div v-if="currentFile" class="file-card">
          <div class="file-icon">
            <i class="el-icon-picture-outline"></i>
          </div>
          <div class="file-info">
            <div class="file-name" :title="currentFile.name">{{ currentFile.name }}</div>
            <div class="file-meta">{{ fileSizeText }}</div>
          </div>
          <el-button type="text" @click="clearFile">移除</el-button>
        </div>

        <el-divider></el-divider>

        <div class="panel-title small">服务状态</div>
        <el-button size="small" icon="el-icon-connection" :loading="healthLoading" @click="checkHealth">
          检查健康状态
        </el-button>

        <div v-if="healthStatus" class="health-box" :class="healthStatus.status">
          <i :class="healthStatus.status === 'healthy' ? 'el-icon-success' : 'el-icon-warning-outline'"></i>
          <span>{{ healthStatus.text }}</span>
        </div>
      </aside>

      <main class="panel preview-panel">
        <div class="panel-toolbar">
          <div>
            <div class="panel-title">原图预览</div>
            <div class="panel-subtitle">结构化识别会在图片上标记文本区域</div>
          </div>

          <div class="toolbar-actions">
            <el-switch v-model="showBoxes" active-text="显示框" :disabled="!results.length"></el-switch>
          </div>
        </div>

        <div class="preview-canvas">
          <div v-if="!previewUrl" class="empty-state">
            <i class="el-icon-document-add"></i>
            <p>上传一张图片开始识别</p>
          </div>

          <div v-else class="image-wrap">
            <img ref="previewImage" :src="previewUrl" class="preview-image" @load="handleImageLoad" />

            <template v-if="showBoxes && resultType === 'json'">
              <button v-for="(item, index) in results" :key="index" class="ocr-box"
                :class="{ weak: item.confidence < 0.85, active: activeIndex === index }" :style="getBoxStyle(item.box)"
                @click="activeIndex = index">
                {{ index + 1 }}
              </button>
            </template>
          </div>
        </div>
      </main>

      <section class="panel result-panel">
        <div class="panel-toolbar">
          <div>
            <div class="panel-title">识别结果</div>
            <div class="panel-subtitle">
              {{ resultType === 'json' ? '可编辑文本与置信度明细' : 'Markdown 输出' }}
            </div>
          </div>

          <div class="toolbar-actions">
            <el-button size="small" icon="el-icon-document-copy" @click="copyResult">
              复制
            </el-button>
            <el-button size="small" icon="el-icon-download" @click="downloadResult">
              导出
            </el-button>
          </div>
        </div>

        <el-alert v-if="errorMessage" type="error" :closable="false" show-icon class="result-alert"
          :title="errorMessage"></el-alert>

        <div v-if="loading" class="loading-state">
          <i class="el-icon-loading"></i>
          <span>正在识别，首次调用可能需要等待模型加载...</span>
        </div>

        <template v-else>
          <el-input v-model="editableText" type="textarea" :rows="10" resize="none" placeholder="识别结果会显示在这里"></el-input>

          <div v-if="resultType === 'json' && results.length" class="result-list">
            <div v-for="(item, index) in results" :key="index" class="result-row"
              :class="{ active: activeIndex === index }" @click="activeIndex = index">
              <div class="row-index">{{ index + 1 }}</div>
              <div class="row-main">
                <el-input v-model="item.text" size="mini" @input="syncEditableText"></el-input>
                <div class="row-meta">
                  <el-tag size="mini" :type="item.confidence >= 0.85 ? 'success' : 'warning'">
                    置信度 {{ formatConfidence(item.confidence) }}
                  </el-tag>
                  <span>坐标 {{ formatBox(item.box) }}</span>
                </div>
              </div>
            </div>
          </div>

          <div v-if="!results.length && !markdownResult && !errorMessage" class="empty-result">
            暂无识别结果
          </div>
        </template>
      </section>
    </div>
  </div>
</template>

<script>
import axios from 'axios'

export default {
  name: 'OcrWorkbench',

  data() {
    return {
      engine: 'pp',
      resultType: 'json',
      currentFile: null,
      previewUrl: '',
      loading: false,
      healthLoading: false,
      healthStatus: null,
      errorMessage: '',
      results: [],
      markdownResult: '',
      editableText: '',
      showBoxes: true,
      activeIndex: -1,
      imageNaturalSize: {
        width: 0,
        height: 0
      },
      imageDisplaySize: {
        width: 0,
        height: 0
      }
    }
  },

  computed: {
    // serviceBase() {
    //   return this.engine === 'pp'
    //     ? 'http://192.168.0.41:8000'
    //     : 'http://192.168.0.41:8001'
    // },

    serviceBase() {
      return this.engine === 'pp'
        ? '/paddle-ocr'
        : '/paddle-vl'
    },

    serviceLabel() {
      return this.engine === 'pp'
        ? 'PP-OCRv5 通用文字识别'
        : 'PaddleOCR-VL 文档识别'
    },

    requestUrl() {
      if (this.engine === 'pp') {
        return this.resultType === 'json'
          ? `${this.serviceBase}/ocr`
          : `${this.serviceBase}/ocr/md`
      }

      return this.resultType === 'json'
        ? `${this.serviceBase}/vl_ocr`
        : `${this.serviceBase}/vl_ocr/md`
    },

    healthUrl() {
      return `${this.serviceBase}/health`
    },

    fileSizeText() {
      if (!this.currentFile) return '-'
      const size = this.currentFile.size || 0
      if (size < 1024) return `${size} B`
      if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
      return `${(size / 1024 / 1024).toFixed(2)} MB`
    },

    averageConfidenceText() {
      if (!this.results.length) return '-'
      const total = this.results.reduce((sum, item) => {
        return sum + Number(item.confidence || 0)
      }, 0)
      return `${((total / this.results.length) * 100).toFixed(1)}%`
    }
  },

  mounted() {
    window.addEventListener('resize', this.updateImageDisplaySize)
  },

  beforeDestroy() {
    window.removeEventListener('resize', this.updateImageDisplaySize)
    this.revokePreviewUrl()
  },

  methods: {
    handleFileChange(file) {
      const raw = file.raw
      if (!raw) return

      const isImage = /^image\/(png|jpe?g|bmp)$/i.test(raw.type)
      const isLt10M = raw.size / 1024 / 1024 <= 10

      if (!isImage) {
        this.$message.error('请上传 PNG / JPG / JPEG / BMP 图片')
        return
      }

      if (!isLt10M) {
        this.$message.warning('图片超过 10MB，可能会影响识别速度')
      }

      this.revokePreviewUrl()
      this.currentFile = raw
      this.previewUrl = URL.createObjectURL(raw)
      this.results = []
      this.markdownResult = ''
      this.editableText = ''
      this.errorMessage = ''
      this.activeIndex = -1
    },

    clearFile() {
      this.revokePreviewUrl()
      this.currentFile = null
      this.previewUrl = ''
      this.results = []
      this.markdownResult = ''
      this.editableText = ''
      this.errorMessage = ''
      this.activeIndex = -1
      this.imageNaturalSize = { width: 0, height: 0 }
      this.imageDisplaySize = { width: 0, height: 0 }
    },

    revokePreviewUrl() {
      if (this.previewUrl) {
        URL.revokeObjectURL(this.previewUrl)
      }
    },

    handleImageLoad(event) {
      const img = event.target
      this.imageNaturalSize = {
        width: img.naturalWidth,
        height: img.naturalHeight
      }
      this.updateImageDisplaySize()
    },

    updateImageDisplaySize() {
      this.$nextTick(() => {
        const img = this.$refs.previewImage
        if (!img) return

        this.imageDisplaySize = {
          width: img.clientWidth,
          height: img.clientHeight
        }
      })
    },

    async checkHealth() {
      this.healthLoading = true
      this.healthStatus = null

      try {
        const res = await axios.get(this.healthUrl, { timeout: 8000 })
        this.healthStatus = {
          status: res.data && res.data.status === 'healthy' ? 'healthy' : 'warning',
          text: `${res.data.service || this.serviceLabel}：${res.data.status || 'unknown'}`
        }
      } catch (error) {
        this.healthStatus = {
          status: 'warning',
          text: this.getErrorText(error)
        }
      } finally {
        this.healthLoading = false
      }
    },

    async handleRecognize() {
      if (!this.currentFile) {
        this.$message.warning('请先上传图片')
        return
      }

      this.loading = true
      this.errorMessage = ''
      this.results = []
      this.markdownResult = ''
      this.editableText = ''
      this.activeIndex = -1

      const formData = new FormData()
      formData.append('file', this.currentFile)

      try {
        const res = await axios.post(this.requestUrl, formData, {
          headers: {
            'Content-Type': 'multipart/form-data'
          },
          timeout: 60000
        })

        if (this.resultType === 'md') {
          this.markdownResult = res.data.markdown || ''
          this.editableText = this.markdownResult
          return
        }

        this.results = Array.isArray(res.data.results) ? res.data.results : []
        this.editableText = this.results.map(item => item.text).join('\n')

        if (!this.results.length) {
          this.$message.warning('接口返回成功，但没有识别到文本')
        }

        this.updateImageDisplaySize()
      } catch (error) {
        this.errorMessage = this.getErrorText(error)
      } finally {
        this.loading = false
      }
    },

    syncEditableText() {
      this.editableText = this.results.map(item => item.text).join('\n')
    },

    getBoxStyle(box) {
      if (
        !Array.isArray(box) ||
        !this.imageNaturalSize.width ||
        !this.imageNaturalSize.height ||
        !this.imageDisplaySize.width ||
        !this.imageDisplaySize.height
      ) {
        return {}
      }

      const xs = box.map(point => Number(point[0]))
      const ys = box.map(point => Number(point[1]))

      const minX = Math.min.apply(null, xs)
      const maxX = Math.max.apply(null, xs)
      const minY = Math.min.apply(null, ys)
      const maxY = Math.max.apply(null, ys)

      const scaleX = this.imageDisplaySize.width / this.imageNaturalSize.width
      const scaleY = this.imageDisplaySize.height / this.imageNaturalSize.height

      return {
        left: `${minX * scaleX}px`,
        top: `${minY * scaleY}px`,
        width: `${Math.max((maxX - minX) * scaleX, 18)}px`,
        height: `${Math.max((maxY - minY) * scaleY, 18)}px`
      }
    },

    formatConfidence(value) {
      if (value === undefined || value === null) return '-'
      return `${(Number(value) * 100).toFixed(1)}%`
    },

    formatBox(box) {
      if (!Array.isArray(box)) return '-'
      return box
        .map(point => `[${Math.round(point[0])}, ${Math.round(point[1])}]`)
        .join(' ')
    },

    async copyResult() {
      if (!this.editableText) {
        this.$message.warning('暂无可复制内容')
        return
      }

      try {
        await navigator.clipboard.writeText(this.editableText)
        this.$message.success('已复制到剪贴板')
      } catch (error) {
        const textarea = document.createElement('textarea')
        textarea.value = this.editableText
        document.body.appendChild(textarea)
        textarea.select()
        document.execCommand('copy')
        document.body.removeChild(textarea)
        this.$message.success('已复制到剪贴板')
      }
    },

    downloadResult() {
      if (!this.editableText) {
        this.$message.warning('暂无可导出内容')
        return
      }

      const ext = this.resultType === 'md' ? 'md' : 'txt'
      const blob = new Blob([this.editableText], {
        type: 'text/plain;charset=utf-8'
      })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')

      link.href = url
      link.download = `ocr-result.${ext}`
      link.click()

      URL.revokeObjectURL(url)
    },

    getErrorText(error) {
      if (error.response && error.response.data) {
        return error.response.data.detail || JSON.stringify(error.response.data)
      }

      if (error.message && error.message.includes('Network Error')) {
        return '网络请求失败，可能是服务不可达或浏览器跨域限制'
      }

      return error.message || '识别失败，请稍后重试'
    }
  }
}
</script>

<style scoped>
.upload-panel /deep/ .el-upload {
  width: 100%;
}

.upload-panel /deep/ .el-upload-dragger {
  width: 100%;
  height: 150px;
  box-sizing: border-box;
}

.upload-panel /deep/ .el-upload__text {
  padding: 0 12px;
  font-size: 13px;
  line-height: 20px;
}

.upload-panel /deep/ .el-upload__tip {
  line-height: 18px;
  word-break: break-all;
}

.ocr-page {
  min-height: 100vh;
  padding: 20px;
  background: #f5f7fa;
  color: #1f2933;
  box-sizing: border-box;
}

.ocr-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 14px;
}

.ocr-header h2 {
  margin: 0 0 6px;
  font-size: 22px;
  font-weight: 600;
  color: #17233d;
}

.ocr-header p {
  margin: 0;
  font-size: 13px;
  color: #667085;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.status-strip {
  display: grid;
  grid-template-columns: 1.1fr 2fr 0.8fr 0.8fr;
  gap: 1px;
  overflow: hidden;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  background: #e4e7ed;
  margin-bottom: 14px;
}

.status-item {
  min-width: 0;
  padding: 10px 12px;
  background: #fff;
}

.status-item span {
  display: block;
  margin-bottom: 4px;
  font-size: 12px;
  color: #8a94a6;
}

.status-item strong {
  display: block;
  overflow: hidden;
  font-size: 13px;
  font-weight: 500;
  color: #2f3a4a;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ocr-layout {
  display: grid;
  grid-template-columns: 280px minmax(360px, 1fr) 420px;
  gap: 14px;
  min-height: calc(100vh - 145px);
}

.panel {
  min-width: 0;
  background: #fff;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  box-sizing: border-box;
}

.upload-panel {
  padding: 14px;
}

.preview-panel,
.result-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-title {
  font-size: 15px;
  font-weight: 600;
  color: #17233d;
}

.panel-title.small {
  margin-bottom: 10px;
  font-size: 14px;
}

.panel-subtitle {
  margin-top: 4px;
  font-size: 12px;
  color: #8a94a6;
}

.panel-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 58px;
  padding: 12px 14px;
  border-bottom: 1px solid #edf0f5;
  box-sizing: border-box;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.file-card {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 14px;
  padding: 10px;
  background: #f8fafc;
  border: 1px solid #edf0f5;
  border-radius: 6px;
}

.file-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  color: #409eff;
  background: #ecf5ff;
  border-radius: 6px;
  flex-shrink: 0;
}

.file-info {
  min-width: 0;
  flex: 1;
}

.file-name {
  overflow: hidden;
  font-size: 13px;
  color: #2f3a4a;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-meta {
  margin-top: 3px;
  font-size: 12px;
  color: #8a94a6;
}

.health-box {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 10px;
  padding: 9px 10px;
  font-size: 12px;
  border-radius: 6px;
}

.health-box.healthy {
  color: #237804;
  background: #f0f9eb;
}

.health-box.warning {
  color: #ad6800;
  background: #fff7e6;
}

.preview-canvas {
  position: relative;
  flex: 1;
  overflow: auto;
  padding: 14px;
  background:
    linear-gradient(45deg, #f4f6f8 25%, transparent 25%),
    linear-gradient(-45deg, #f4f6f8 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, #f4f6f8 75%),
    linear-gradient(-45deg, transparent 75%, #f4f6f8 75%);
  background-color: #ffffff;
  background-position: 0 0, 0 8px, 8px -8px, -8px 0;
  background-size: 16px 16px;
}

.image-wrap {
  position: relative;
  display: inline-block;
  max-width: 100%;
}

.preview-image {
  display: block;
  max-width: 100%;
  max-height: calc(100vh - 245px);
  border: 1px solid #dcdfe6;
  background: #fff;
}

.ocr-box {
  position: absolute;
  padding: 0;
  color: #0b5cab;
  font-size: 11px;
  line-height: 16px;
  text-align: left;
  background: rgba(64, 158, 255, 0.08);
  border: 2px solid rgba(64, 158, 255, 0.9);
  border-radius: 2px;
  cursor: pointer;
  box-sizing: border-box;
}

.ocr-box.weak {
  color: #ad6800;
  background: rgba(230, 162, 60, 0.12);
  border-color: rgba(230, 162, 60, 0.95);
}

.ocr-box.active {
  color: #fff;
  background: rgba(245, 108, 108, 0.18);
  border-color: #f56c6c;
}

.result-panel {
  padding-bottom: 14px;
}

.result-panel .el-textarea {
  padding: 14px;
  box-sizing: border-box;
}

.result-alert {
  margin: 14px 14px 0;
  width: auto;
}

.result-list {
  margin: 0 14px;
  padding-top: 4px;
  overflow: auto;
}

.result-row {
  display: flex;
  gap: 10px;
  padding: 10px 0;
  border-top: 1px solid #edf0f5;
  cursor: pointer;
}

.result-row.active {
  background: #f8fafc;
}

.row-index {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  margin-top: 1px;
  color: #606266;
  font-size: 12px;
  background: #f0f2f5;
  border-radius: 50%;
  flex-shrink: 0;
}

.row-main {
  min-width: 0;
  flex: 1;
}

.row-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 6px;
  overflow: hidden;
  font-size: 12px;
  color: #8a94a6;
  white-space: nowrap;
}

.row-meta span:last-child {
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-state,
.empty-result,
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #8a94a6;
}

.empty-state {
  flex-direction: column;
  height: 100%;
  min-height: 360px;
}

.empty-state i {
  margin-bottom: 8px;
  font-size: 40px;
  color: #c0c4cc;
}

.empty-result {
  height: 120px;
  font-size: 13px;
}

.loading-state {
  gap: 8px;
  height: 120px;
  font-size: 13px;
}

@media (max-width: 1200px) {
  .ocr-layout {
    grid-template-columns: 260px 1fr;
  }

  .result-panel {
    grid-column: 1 / -1;
    min-height: 420px;
  }
}

@media (max-width: 768px) {
  .ocr-page {
    padding: 12px;
  }

  .ocr-header {
    flex-direction: column;
  }

  .header-actions {
    justify-content: flex-start;
  }

  .status-strip,
  .ocr-layout {
    grid-template-columns: 1fr;
  }

  .preview-image {
    max-height: 60vh;
  }
}
</style>