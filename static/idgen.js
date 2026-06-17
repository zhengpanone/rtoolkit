Vue.createApp({
  data: function () {
    return {
      loading: false,
      errorMessage: '',
      metrics: [],
      records: [],
      lastText: '',
      exportFormat: 'txt',
      directExportFormat: 'csv',
      copiedIdNumber: '',
      regionsLoading: false,
      regionOptions: {
        provinces: [],
        cities: [],
        regions: []
      },
      selectedRegion: {
        province: '',
        city: '',
        area: ''
      },
      addressPopover: {
        visible: false,
        text: '',
        x: 0,
        y: 0,
        copied: false
      },
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

  mounted: function () {
    document.addEventListener('click', this.handleDocumentClick);
    document.addEventListener('keydown', this.handleDocumentKeydown);
    this.loadRegionOptions();
  },

  beforeUnmount: function () {
    document.removeEventListener('click', this.handleDocumentClick);
    document.removeEventListener('keydown', this.handleDocumentKeydown);
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
          { label: '可下载', value: '4 种格式' }
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

    downloadGeneratedFile: async function () {
      this.errorMessage = '';
      if (this.directExportFormat === 'excel' && Number(this.form.count || 1) > 1048575) {
        this.errorMessage = 'Excel 单个工作表最多支持 1048575 条数据，请选择 CSV/TEXT/JSON 或降低数量。';
        return;
      }
      var query = this.directDownloadQuery();
      var link = document.createElement('a');
      link.href = '/api/idgen/download?' + query;
      link.download = 'idgen-' + this.timestamp() + '.' + this.directExportExt();
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    },

    directExportExt: function () {
      if (this.directExportFormat === 'excel') return 'xlsx';
      if (this.directExportFormat === 'json') return 'json';
      if (this.directExportFormat === 'txt') return 'txt';
      return 'csv';
    },

    directDownloadQuery: function () {
      var params = [];
      var payload = this.normalizedPayload();
      payload.format = this.directExportFormat || 'csv';
      Object.keys(payload).forEach(function (key) {
        if (payload[key] !== null && payload[key] !== undefined && payload[key] !== '') {
          params.push(encodeURIComponent(key) + '=' + encodeURIComponent(payload[key]));
        }
      });
      return params.join('&');
    },

    loadRegionOptions: async function () {
      this.regionsLoading = true;
      try {
        var response = await fetch('/api/regions');
        var data = await response.json();
        if (!response.ok) {
          throw new Error(data.error || 'load regions failed');
        }
        this.regionOptions.provinces = data.provinces || [];
        this.regionOptions.cities = data.cities || [];
        this.regionOptions.regions = data.regions || [];
        this.syncRegionSelectorsFromCode();
      } catch (error) {
        this.errorMessage = error.message || String(error);
      } finally {
        this.regionsLoading = false;
      }
    },

    filteredCities: function () {
      var province = this.selectedRegion.province;
      if (!province) return [];
      return this.regionOptions.cities.filter(function (city) {
        return city.provinceCode === province ||
          city.province_code === province ||
          city.province === province;
      });
    },

    filteredAreas: function () {
      var city = this.selectedRegion.city;
      if (!city) return [];
      return this.regionOptions.regions.filter(function (area) {
        return area.cityCode === city ||
          area.city_code === city ||
          area.city === city;
      });
    },

    selectedProvinceOption: function () {
      var province = this.selectedRegion.province;
      if (!province) return null;
      return this.regionOptions.provinces.find(function (item) {
        return item.code === province;
      }) || null;
    },

    selectedCityOption: function () {
      var city = this.selectedRegion.city;
      if (!city) return null;
      return this.regionOptions.cities.find(function (item) {
        return item.code === city;
      }) || null;
    },

    selectedAreaOption: function () {
      var area = this.selectedRegion.area;
      if (!area) return null;
      return this.regionOptions.regions.find(function (item) {
        return item.code === area;
      }) || null;
    },

    onProvinceChange: function () {
      this.selectedRegion.city = '';
      this.selectedRegion.area = '';
      this.form.region = this.selectedRegion.province || '';
    },

    onCityChange: function () {
      this.selectedRegion.area = '';
      this.form.region = this.selectedRegion.city || this.selectedRegion.province || '';
    },

    onAreaChange: function () {
      this.form.region = this.selectedRegion.area || '';
    },

    clearRegionSelection: function () {
      this.form.region = '';
      this.selectedRegion.province = '';
      this.selectedRegion.city = '';
      this.selectedRegion.area = '';
    },

    clearBirthSelection: function () {
      this.form.birth = '';
      this.form.min_birth = '';
      this.form.max_birth = '';
    },

    onRegionCodeInput: function () {
      this.syncRegionSelectorsFromCode();
    },

    syncRegionSelectorsFromCode: function () {
      var code = (this.form.region || '').trim();
      if (!/^\d{2,6}$/.test(code)) {
        if (!code) {
          this.selectedRegion.province = '';
          this.selectedRegion.city = '';
          this.selectedRegion.area = '';
        }
        return;
      }

      if (code.length === 2) {
        var matchedProvince = this.regionOptions.provinces.find(function (province) {
          return province.code === code;
        });
        if (!matchedProvince) {
          this.selectedRegion.province = '';
          this.selectedRegion.city = '';
          this.selectedRegion.area = '';
          return;
        }
        this.selectedRegion.province = matchedProvince.code;
        this.selectedRegion.city = '';
        this.selectedRegion.area = '';
        return;
      }

      if (code.length === 4) {
        var matchedCity = this.regionOptions.cities.find(function (city) {
          return city.code === code;
        });
        if (!matchedCity) {
          this.selectedRegion.province = '';
          this.selectedRegion.city = '';
          this.selectedRegion.area = '';
          return;
        }
        var cityProvinceCode = matchedCity.provinceCode ||
          matchedCity.province_code ||
          matchedCity.province;
        this.selectedRegion.province = cityProvinceCode;
        this.selectedRegion.city = matchedCity.code;
        this.selectedRegion.area = '';
        return;
      }

      if (code.length !== 6) return;

      var matchedArea = this.regionOptions.regions.find(function (area) {
        return area.code === code;
      });
      if (!matchedArea) {
        this.selectedRegion.province = '';
        this.selectedRegion.city = '';
        this.selectedRegion.area = '';
        return;
      }

      var cityCode = matchedArea.cityCode || matchedArea.city_code || matchedArea.city;
      var provinceCode = matchedArea.provinceCode || matchedArea.province_code || matchedArea.province;
      this.selectedRegion.province = provinceCode;
      this.selectedRegion.city = cityCode;
      this.selectedRegion.area = matchedArea.code;
    },

    copyIdNumber: async function (idNumber) {
      if (!idNumber) return;
      await navigator.clipboard.writeText(idNumber);
      this.copiedIdNumber = idNumber;
      var app = this;
      window.setTimeout(function () {
        if (app.copiedIdNumber === idNumber) {
          app.copiedIdNumber = '';
        }
      }, 1400);
    },

    openAddressPopover: function (address, event) {
      if (!address) return;
      if (!event) return;

      var target = event.currentTarget;
      var rect = target.getBoundingClientRect();
      var popoverWidth = Math.min(520, window.innerWidth - 32);
      var x = rect.left;
      var y = rect.bottom + 8;
      if (x + popoverWidth > window.innerWidth - 16) {
        x = window.innerWidth - popoverWidth - 16;
      }
      if (y > window.innerHeight - 180) {
        y = Math.max(16, rect.top - 160);
      }

      this.addressPopover.text = address;
      this.addressPopover.x = Math.max(16, x);
      this.addressPopover.y = Math.max(16, y);
      this.addressPopover.copied = false;
      this.addressPopover.visible = true;
    },

    closeAddressPopover: function () {
      this.addressPopover.visible = false;
      this.addressPopover.text = '';
      this.addressPopover.copied = false;
    },

    copyAddress: async function () {
      if (!this.addressPopover.text) return;
      await navigator.clipboard.writeText(this.addressPopover.text);
      this.addressPopover.copied = true;
      var app = this;
      window.setTimeout(function () {
        app.addressPopover.copied = false;
      }, 1400);
    },

    handleDocumentClick: function (event) {
      if (!this.addressPopover.visible) return;
      if (event.target.closest('.address-popover')) return;
      if (event.target.closest('.address-button')) return;
      this.closeAddressPopover();
    },

    handleDocumentKeydown: function (event) {
      if (event.key === 'Escape') {
        this.closeAddressPopover();
      }
    },

    downloadResult: function () {
      if (!this.records.length) return;
      var format = this.exportFormat || 'txt';
      var exportData = this.buildExport(format);
      var blob = new Blob([exportData.content], { type: exportData.type });
      var link = document.createElement('a');
      link.href = window.URL.createObjectURL(blob);
      link.download = 'idgen-' + this.timestamp() + '.' + exportData.ext;
      document.body.appendChild(link);
      link.click();
      window.URL.revokeObjectURL(link.href);
      document.body.removeChild(link);
    },

    buildExport: function (format) {
      if (format === 'json') {
        return {
          ext: 'json',
          type: 'application/json;charset=utf-8',
          content: JSON.stringify(this.records, null, 2)
        };
      }

      if (format === 'csv') {
        return {
          ext: 'csv',
          type: 'text/csv;charset=utf-8',
          content: '\ufeff' + this.toCsv()
        };
      }

      if (format === 'xlsx') {
        return {
          ext: 'xlsx',
          type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
          content: this.toXlsx()
        };
      }

      return {
        ext: 'txt',
        type: 'text/plain;charset=utf-8',
        content: this.toTxt()
      };
    },

    toTxt: function () {
      var app = this;
      var rows = [['姓名', '身份证号', '生日', '性别', '地址']];
      this.records.forEach(function (row) {
        rows.push([
          row.name,
          row.id_number,
          row.birthday,
          app.genderLabel(row.gender),
          row.address
        ]);
      });
      return rows.map(function (row) {
        return row.join('\t');
      }).join('\n');
    },

    toCsv: function () {
      var app = this;
      var rows = [['姓名', '身份证号', '生日', '性别', '地址']];
      this.records.forEach(function (row) {
        rows.push([
          row.name,
          row.id_number,
          row.birthday,
          app.genderLabel(row.gender),
          row.address
        ]);
      });
      return rows.map(function (row) {
        return row.map(function (cell) {
          return app.escapeCsv(cell);
        }).join(',');
      }).join('\n');
    },

    toXlsx: function () {
      var app = this;
      var rows = [['姓名', '身份证号', '生日', '性别', '地址']];
      var generatedAt = new Date().toLocaleString('zh-CN', { hour12: false });
      this.records.forEach(function (row) {
        rows.push([
          row.name,
          row.id_number,
          row.birthday,
          app.genderLabel(row.gender),
          row.address
        ]);
      });

      function colName(index) {
        var name = '';
        var current = index + 1;
        while (current > 0) {
          var remainder = (current - 1) % 26;
          name = String.fromCharCode(65 + remainder) + name;
          current = Math.floor((current - 1) / 26);
        }
        return name;
      }

      function inlineCell(ref, value, styleIndex) {
        return [
          '<c r="' + ref + '" t="inlineStr" s="' + styleIndex + '">',
          '<is><t>' + app.escapeXml(value) + '</t></is>',
          '</c>'
        ].join('');
      }

      var tableRows = rows.map(function (row, rowIndex) {
        var cells = row.map(function (cell, cellIndex) {
          var styleIndex = 2;
          if (rowIndex === 0) {
            styleIndex = 1;
          } else if (cellIndex === 1) {
            styleIndex = 4;
          } else if (cellIndex === 0 || cellIndex === 2 || cellIndex === 3) {
            styleIndex = 3;
          }
          return inlineCell(colName(cellIndex) + (rowIndex + 3), cell, styleIndex);
        }).join('');
        return '<row r="' + (rowIndex + 3) + '" ht="' + (rowIndex === 0 ? '24' : '22') + '" customHeight="1">' + cells + '</row>';
      }).join('');

      var sheetXml = [
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
        '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">',
        '<sheetViews><sheetView workbookViewId="0"><pane ySplit="3" topLeftCell="A4" activePane="bottomLeft" state="frozen"/></sheetView></sheetViews>',
        '<cols>',
        '<col min="1" max="1" width="12" customWidth="1"/>',
        '<col min="2" max="2" width="24" customWidth="1"/>',
        '<col min="3" max="3" width="14" customWidth="1"/>',
        '<col min="4" max="4" width="10" customWidth="1"/>',
        '<col min="5" max="5" width="42" customWidth="1"/>',
        '</cols>',
        '<sheetData>',
        '<row r="1" ht="30" customHeight="1">' + inlineCell('A1', '身份证生成结果', 5) + '</row>',
        '<row r="2" ht="22" customHeight="1">' + inlineCell('A2', '生成时间：' + generatedAt + '　记录数：' + this.records.length, 6) + '</row>',
        tableRows,
        '</sheetData>',
        '<mergeCells count="2"><mergeCell ref="A1:E1"/><mergeCell ref="A2:E2"/></mergeCells>',
        '</worksheet>'
      ].join('');

      return this.buildXlsxPackage(sheetXml);
    },

    buildXlsxPackage: function (sheetXml) {
      var files = [
        {
          name: '[Content_Types].xml',
          content: [
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
            '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">',
            '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>',
            '<Default Extension="xml" ContentType="application/xml"/>',
            '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>',
            '<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>',
            '<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>',
            '<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>',
            '<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>',
            '</Types>'
          ].join('')
        },
        {
          name: '_rels/.rels',
          content: [
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">',
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>',
            '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>',
            '<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>',
            '</Relationships>'
          ].join('')
        },
        {
          name: 'docProps/core.xml',
          content: [
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
            '<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">',
            '<dc:creator>rtoolkit</dc:creator>',
            '<cp:lastModifiedBy>rtoolkit</cp:lastModifiedBy>',
            '<dcterms:created xsi:type="dcterms:W3CDTF">' + this.escapeXml(new Date().toISOString()) + '</dcterms:created>',
            '<dcterms:modified xsi:type="dcterms:W3CDTF">' + this.escapeXml(new Date().toISOString()) + '</dcterms:modified>',
            '</cp:coreProperties>'
          ].join('')
        },
        {
          name: 'docProps/app.xml',
          content: [
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
            '<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">',
            '<Application>rtoolkit</Application>',
            '</Properties>'
          ].join('')
        },
        {
          name: 'xl/workbook.xml',
          content: [
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
            '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">',
            '<sheets><sheet name="身份证生成结果" sheetId="1" r:id="rId1"/></sheets>',
            '</workbook>'
          ].join('')
        },
        {
          name: 'xl/_rels/workbook.xml.rels',
          content: [
            '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
            '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">',
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>',
            '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>',
            '</Relationships>'
          ].join('')
        },
        {
          name: 'xl/styles.xml',
          content: this.xlsxStyles()
        },
        {
          name: 'xl/worksheets/sheet1.xml',
          content: sheetXml
        }
      ];
      return this.zipStore(files);
    },

    xlsxStyles: function () {
      return [
        '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>',
        '<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">',
        '<fonts count="5">',
        '<font><sz val="11"/><color rgb="FF1F2937"/><name val="Microsoft YaHei"/></font>',
        '<font><b/><sz val="11"/><color rgb="FFFFFFFF"/><name val="Microsoft YaHei"/></font>',
        '<font><sz val="11"/><color rgb="FF1F2937"/><name val="Microsoft YaHei"/></font>',
        '<font><sz val="11"/><color rgb="FF1D4ED8"/><name val="Consolas"/></font>',
        '<font><b/><sz val="16"/><color rgb="FF17365D"/><name val="Microsoft YaHei"/></font>',
        '</fonts>',
        '<fills count="5">',
        '<fill><patternFill patternType="none"/></fill>',
        '<fill><patternFill patternType="gray125"/></fill>',
        '<fill><patternFill patternType="solid"><fgColor rgb="FF1F4E79"/><bgColor indexed="64"/></patternFill></fill>',
        '<fill><patternFill patternType="solid"><fgColor rgb="FFEAF2F8"/><bgColor indexed="64"/></patternFill></fill>',
        '<fill><patternFill patternType="solid"><fgColor rgb="FFF8FAFC"/><bgColor indexed="64"/></patternFill></fill>',
        '</fills>',
        '<borders count="2">',
        '<border><left/><right/><top/><bottom/><diagonal/></border>',
        '<border><left style="thin"><color rgb="FFCBD5E1"/></left><right style="thin"><color rgb="FFCBD5E1"/></right><top style="thin"><color rgb="FFCBD5E1"/></top><bottom style="thin"><color rgb="FFCBD5E1"/></bottom><diagonal/></border>',
        '</borders>',
        '<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>',
        '<cellXfs count="7">',
        '<xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>',
        '<xf numFmtId="0" fontId="1" fillId="2" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment horizontal="center" vertical="center"/></xf>',
        '<xf numFmtId="0" fontId="0" fillId="0" borderId="1" xfId="0" applyBorder="1" applyAlignment="1"><alignment vertical="center"/></xf>',
        '<xf numFmtId="0" fontId="0" fillId="0" borderId="1" xfId="0" applyBorder="1" applyAlignment="1"><alignment horizontal="center" vertical="center"/></xf>',
        '<xf numFmtId="49" fontId="3" fillId="0" borderId="1" xfId="0" applyNumberFormat="1" applyFont="1" applyBorder="1" applyAlignment="1"><alignment horizontal="center" vertical="center"/></xf>',
        '<xf numFmtId="0" fontId="4" fillId="3" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment horizontal="center" vertical="center"/></xf>',
        '<xf numFmtId="0" fontId="0" fillId="4" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment horizontal="right" vertical="center"/></xf>',
        '</cellXfs>',
        '<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>',
        '</styleSheet>'
      ].join('');
    },

    zipStore: function (files) {
      var encoder = new TextEncoder();
      var localParts = [];
      var centralParts = [];
      var offset = 0;
      var app = this;

      files.forEach(function (file) {
        var nameBytes = encoder.encode(file.name);
        var contentBytes = encoder.encode(file.content);
        var crc = app.crc32(contentBytes);
        var localHeader = app.zipLocalHeader(crc, contentBytes.length, nameBytes.length);
        localParts.push(localHeader, nameBytes, contentBytes);

        var centralHeader = app.zipCentralHeader(crc, contentBytes.length, nameBytes.length, offset);
        centralParts.push(centralHeader, nameBytes);
        offset += localHeader.length + nameBytes.length + contentBytes.length;
      });

      var centralSize = centralParts.reduce(function (sum, part) {
        return sum + part.length;
      }, 0);
      var endRecord = this.zipEndRecord(files.length, centralSize, offset);
      return new Blob(localParts.concat(centralParts, [endRecord]), {
        type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
      });
    },

    zipLocalHeader: function (crc, size, nameLength) {
      var array = new Uint8Array(30);
      var view = new DataView(array.buffer);
      view.setUint32(0, 0x04034b50, true);
      view.setUint16(4, 20, true);
      view.setUint16(6, 0, true);
      view.setUint16(8, 0, true);
      view.setUint16(10, 0, true);
      view.setUint16(12, 0, true);
      view.setUint32(14, crc, true);
      view.setUint32(18, size, true);
      view.setUint32(22, size, true);
      view.setUint16(26, nameLength, true);
      view.setUint16(28, 0, true);
      return array;
    },

    zipCentralHeader: function (crc, size, nameLength, offset) {
      var array = new Uint8Array(46);
      var view = new DataView(array.buffer);
      view.setUint32(0, 0x02014b50, true);
      view.setUint16(4, 20, true);
      view.setUint16(6, 20, true);
      view.setUint16(8, 0, true);
      view.setUint16(10, 0, true);
      view.setUint16(12, 0, true);
      view.setUint16(14, 0, true);
      view.setUint32(16, crc, true);
      view.setUint32(20, size, true);
      view.setUint32(24, size, true);
      view.setUint16(28, nameLength, true);
      view.setUint16(30, 0, true);
      view.setUint16(32, 0, true);
      view.setUint16(34, 0, true);
      view.setUint16(36, 0, true);
      view.setUint32(38, 0, true);
      view.setUint32(42, offset, true);
      return array;
    },

    zipEndRecord: function (fileCount, centralSize, centralOffset) {
      var array = new Uint8Array(22);
      var view = new DataView(array.buffer);
      view.setUint32(0, 0x06054b50, true);
      view.setUint16(4, 0, true);
      view.setUint16(6, 0, true);
      view.setUint16(8, fileCount, true);
      view.setUint16(10, fileCount, true);
      view.setUint32(12, centralSize, true);
      view.setUint32(16, centralOffset, true);
      view.setUint16(20, 0, true);
      return array;
    },

    crc32: function (bytes) {
      if (!this.crcTable) {
        this.crcTable = [];
        for (var n = 0; n < 256; n += 1) {
          var c = n;
          for (var k = 0; k < 8; k += 1) {
            c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
          }
          this.crcTable[n] = c >>> 0;
        }
      }

      var crc = 0xffffffff;
      for (var i = 0; i < bytes.length; i += 1) {
        crc = this.crcTable[(crc ^ bytes[i]) & 0xff] ^ (crc >>> 8);
      }
      return (crc ^ 0xffffffff) >>> 0;
    },

    escapeXml: function (value) {
      var text = value == null ? '' : String(value);
      return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&apos;');
    },

    escapeCsv: function (value) {
      var text = value == null ? '' : String(value);
      if (/[",\r\n]/.test(text)) {
        return '"' + text.replace(/"/g, '""') + '"';
      }
      return text;
    },

    escapeHtml: function (value) {
      var text = value == null ? '' : String(value);
      return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
    },

    timestamp: function () {
      var now = new Date();
      function pad(value) {
        return String(value).padStart(2, '0');
      }
      return String(now.getFullYear()) +
        pad(now.getMonth() + 1) +
        pad(now.getDate()) +
        '-' +
        pad(now.getHours()) +
        pad(now.getMinutes()) +
        pad(now.getSeconds());
    },

    genderLabel: function (value) {
      if (value === 'male') return '男';
      if (value === 'female') return '女';
      return '随机';
    }
  }
}).mount('#app');
