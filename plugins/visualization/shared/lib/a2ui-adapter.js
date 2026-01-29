/**
 * A2UI Adapter v2.0
 * Transforms A2UI-inspired visualization schemas to render-ready configurations
 *
 * Features:
 * - BoundValue resolution (literalString, literalNumber, literalBoolean, path)
 * - Schema validation with helpful error messages
 * - Type-specific transformations for all 9 visualization types
 */
const A2UIAdapter = {
  VERSION: '2.0',

  // ============================================================================
  // 1. VALIDATE - Schema validation per type
  // ============================================================================
  validate: {
    /**
     * Main validation entry point
     * @param {Object} spec - The visualization specification
     * @returns {{ valid: boolean, errors: Array<{ code: string, message: string }> }}
     */
    spec(spec) {
      const errors = [];

      if (!spec || typeof spec !== 'object') {
        errors.push({ code: 'INVALID_SPEC', message: 'Specification must be an object' });
        return { valid: false, errors };
      }

      if (!spec.type) {
        errors.push({ code: 'MISSING_TYPE', message: 'Missing required field: type' });
        return { valid: false, errors };
      }

      const validTypes = ['chart', 'graph', 'timeline', 'tree', 'calendar', 'flowchart', 'grid', 'kanban', 'map'];
      if (!validTypes.includes(spec.type)) {
        errors.push({ code: 'INVALID_TYPE', message: `Invalid type "${spec.type}". Must be one of: ${validTypes.join(', ')}` });
        return { valid: false, errors };
      }

      // Type-specific validation
      const typeValidator = this[spec.type];
      if (typeValidator) {
        const typeErrors = typeValidator(spec);
        errors.push(...typeErrors);
      }

      return { valid: errors.length === 0, errors };
    },

    /**
     * Check required fields exist
     */
    required(spec, fields) {
      const errors = [];
      for (const field of fields) {
        const parts = field.split('.');
        let value = spec;
        for (const part of parts) {
          value = value?.[part];
        }
        if (value === undefined || value === null) {
          errors.push({ code: 'MISSING_FIELD', message: `Missing required field: ${field}` });
        }
      }
      return errors;
    },

    chart(spec) {
      const errors = [];

      // Check config
      if (!spec.config) {
        errors.push({ code: 'MISSING_CONFIG', message: 'Chart requires config object' });
      } else {
        const validChartTypes = ['bar', 'line', 'area', 'pie'];
        if (spec.config.chartType && !validChartTypes.includes(spec.config.chartType)) {
          errors.push({ code: 'INVALID_CHART_TYPE', message: `Invalid chartType "${spec.config.chartType}". Must be: ${validChartTypes.join(', ')}` });
        }
        if (spec.config.chartType !== 'pie') {
          if (!spec.config.xAxis?.field) {
            errors.push({ code: 'MISSING_X_AXIS', message: 'Non-pie charts require config.xAxis.field' });
          }
        }
        if (!spec.config.series || !Array.isArray(spec.config.series) || spec.config.series.length === 0) {
          errors.push({ code: 'MISSING_SERIES', message: 'Chart requires config.series array with at least one series' });
        }
      }

      // Check data
      if (!spec.data?.rows || !Array.isArray(spec.data.rows)) {
        errors.push({ code: 'MISSING_DATA', message: 'Chart requires data.rows array' });
      }

      return errors;
    },

    graph(spec) {
      const errors = [];

      if (!spec.data?.nodes || !Array.isArray(spec.data.nodes)) {
        errors.push({ code: 'MISSING_NODES', message: 'Graph requires data.nodes array' });
      } else {
        spec.data.nodes.forEach((node, i) => {
          if (!node.id) {
            errors.push({ code: 'MISSING_NODE_ID', message: `Node at index ${i} missing required id` });
          }
        });
      }

      if (!spec.data?.edges || !Array.isArray(spec.data.edges)) {
        errors.push({ code: 'MISSING_EDGES', message: 'Graph requires data.edges array' });
      } else {
        spec.data.edges.forEach((edge, i) => {
          if (!edge.source) {
            errors.push({ code: 'MISSING_EDGE_SOURCE', message: `Edge at index ${i} missing required source` });
          }
          if (!edge.target) {
            errors.push({ code: 'MISSING_EDGE_TARGET', message: `Edge at index ${i} missing required target` });
          }
        });
      }

      return errors;
    },

    timeline(spec) {
      const errors = [];

      if (!spec.data?.resources || !Array.isArray(spec.data.resources)) {
        errors.push({ code: 'MISSING_RESOURCES', message: 'Timeline requires data.resources array' });
      } else {
        spec.data.resources.forEach((resource, i) => {
          if (!resource.id) {
            errors.push({ code: 'MISSING_RESOURCE_ID', message: `Resource at index ${i} missing required id` });
          }
        });
      }

      if (!spec.data?.events || !Array.isArray(spec.data.events)) {
        errors.push({ code: 'MISSING_EVENTS', message: 'Timeline requires data.events array' });
      } else {
        spec.data.events.forEach((event, i) => {
          if (!event.id) {
            errors.push({ code: 'MISSING_EVENT_ID', message: `Event at index ${i} missing required id` });
          }
          if (!event.resourceId) {
            errors.push({ code: 'MISSING_RESOURCE_ID', message: `Event at index ${i} missing required resourceId` });
          }
          if (!event.start) {
            errors.push({ code: 'MISSING_EVENT_START', message: `Event at index ${i} missing required start date` });
          }
          if (!event.end) {
            errors.push({ code: 'MISSING_EVENT_END', message: `Event at index ${i} missing required end date` });
          }
        });
      }

      return errors;
    },

    tree(spec) {
      const errors = [];

      if (!spec.data?.hierarchy) {
        errors.push({ code: 'MISSING_HIERARCHY', message: 'Tree requires data.hierarchy object' });
      } else {
        if (!spec.data.hierarchy.id && !spec.data.hierarchy.name) {
          errors.push({ code: 'INVALID_ROOT', message: 'Tree root must have id or name' });
        }
      }

      return errors;
    },

    calendar(spec) {
      const errors = [];

      if (!spec.data?.events || !Array.isArray(spec.data.events)) {
        errors.push({ code: 'MISSING_EVENTS', message: 'Calendar requires data.events array' });
      } else {
        spec.data.events.forEach((event, i) => {
          if (!event.id) {
            errors.push({ code: 'MISSING_EVENT_ID', message: `Event at index ${i} missing required id` });
          }
          if (!event.start) {
            errors.push({ code: 'MISSING_EVENT_START', message: `Event at index ${i} missing required start date` });
          }
        });
      }

      return errors;
    },

    flowchart(spec) {
      const errors = [];

      if (!spec.data?.nodes || !Array.isArray(spec.data.nodes)) {
        errors.push({ code: 'MISSING_NODES', message: 'Flowchart requires data.nodes array' });
      } else {
        spec.data.nodes.forEach((node, i) => {
          if (!node.id) {
            errors.push({ code: 'MISSING_NODE_ID', message: `Node at index ${i} missing required id` });
          }
        });
      }

      if (!spec.data?.edges || !Array.isArray(spec.data.edges)) {
        errors.push({ code: 'MISSING_EDGES', message: 'Flowchart requires data.edges array' });
      } else {
        spec.data.edges.forEach((edge, i) => {
          if (!edge.from) {
            errors.push({ code: 'MISSING_EDGE_FROM', message: `Edge at index ${i} missing required from` });
          }
          if (!edge.to) {
            errors.push({ code: 'MISSING_EDGE_TO', message: `Edge at index ${i} missing required to` });
          }
        });
      }

      return errors;
    },

    grid(spec) {
      const errors = [];

      if (!spec.config?.columns || !Array.isArray(spec.config.columns)) {
        errors.push({ code: 'MISSING_COLUMNS', message: 'Grid requires config.columns array' });
      } else {
        spec.config.columns.forEach((col, i) => {
          if (!col.field) {
            errors.push({ code: 'MISSING_COLUMN_FIELD', message: `Column at index ${i} missing required field` });
          }
        });
      }

      if (!spec.data?.rows || !Array.isArray(spec.data.rows)) {
        errors.push({ code: 'MISSING_ROWS', message: 'Grid requires data.rows array' });
      }

      return errors;
    },

    kanban(spec) {
      const errors = [];

      if (!spec.data?.columns || !Array.isArray(spec.data.columns)) {
        errors.push({ code: 'MISSING_COLUMNS', message: 'Kanban requires data.columns array' });
      } else {
        spec.data.columns.forEach((col, i) => {
          if (!col.id) {
            errors.push({ code: 'MISSING_COLUMN_ID', message: `Column at index ${i} missing required id` });
          }
        });
      }

      if (!spec.data?.cards || !Array.isArray(spec.data.cards)) {
        errors.push({ code: 'MISSING_CARDS', message: 'Kanban requires data.cards array' });
      } else {
        spec.data.cards.forEach((card, i) => {
          if (!card.id) {
            errors.push({ code: 'MISSING_CARD_ID', message: `Card at index ${i} missing required id` });
          }
          if (!card.columnId) {
            errors.push({ code: 'MISSING_CARD_COLUMN', message: `Card at index ${i} missing required columnId` });
          }
        });
      }

      return errors;
    },

    map(spec) {
      const errors = [];

      if (!spec.config?.center) {
        errors.push({ code: 'MISSING_CENTER', message: 'Map requires config.center [lat, lng]' });
      } else if (!Array.isArray(spec.config.center) || spec.config.center.length !== 2) {
        errors.push({ code: 'INVALID_CENTER', message: 'Map config.center must be [lat, lng] array' });
      }

      if (spec.data?.markers && Array.isArray(spec.data.markers)) {
        spec.data.markers.forEach((marker, i) => {
          if (!marker.id) {
            errors.push({ code: 'MISSING_MARKER_ID', message: `Marker at index ${i} missing required id` });
          }
          if (!marker.coordinates || marker.coordinates.lat === undefined || marker.coordinates.lng === undefined) {
            errors.push({ code: 'MISSING_MARKER_COORDS', message: `Marker at index ${i} missing required coordinates {lat, lng}` });
          }
        });
      }

      return errors;
    }
  },

  // ============================================================================
  // 2. RESOLVE - BoundValue resolution
  // ============================================================================
  resolve: {
    /**
     * Resolve a single BoundValue to its actual value
     * @param {*} boundValue - The value to resolve (may be BoundValue or literal)
     * @param {Object} context - Optional context for path resolution
     * @returns {*} The resolved value
     */
    value(boundValue, context = {}) {
      if (boundValue === null || boundValue === undefined) {
        return boundValue;
      }

      // Check for BoundValue types
      if (typeof boundValue === 'object' && !Array.isArray(boundValue)) {
        if ('literalString' in boundValue) {
          return boundValue.literalString;
        }
        if ('literalNumber' in boundValue) {
          return boundValue.literalNumber;
        }
        if ('literalBoolean' in boundValue) {
          return boundValue.literalBoolean;
        }
        if ('path' in boundValue && context) {
          return this.getPath(context, boundValue.path);
        }
      }

      // Pass through non-BoundValue
      return boundValue;
    },

    /**
     * Check if a value is a BoundValue
     */
    isBoundValue(v) {
      return v && typeof v === 'object' && !Array.isArray(v) &&
        ('literalString' in v || 'literalNumber' in v || 'literalBoolean' in v || 'path' in v);
    },

    /**
     * Get a value from context using a path like "/data/field"
     */
    getPath(context, path) {
      if (!path || typeof path !== 'string') return undefined;

      // Remove leading slash and split
      const parts = path.replace(/^\//, '').split('/');
      let value = context;

      for (const part of parts) {
        if (value === null || value === undefined) return undefined;
        value = value[part];
      }

      return value;
    },

    /**
     * Recursively resolve all BoundValues in an object
     */
    all(obj, context = {}) {
      if (obj === null || obj === undefined) {
        return obj;
      }

      // Handle arrays
      if (Array.isArray(obj)) {
        return obj.map(item => this.all(item, context));
      }

      // Handle objects
      if (typeof obj === 'object') {
        // Check if this is a BoundValue itself
        if (this.isBoundValue(obj)) {
          return this.value(obj, context);
        }

        // Recursively resolve all properties
        const result = {};
        for (const [key, value] of Object.entries(obj)) {
          result[key] = this.all(value, context);
        }
        return result;
      }

      // Return primitives as-is
      return obj;
    }
  },

  // ============================================================================
  // 3. NORMALIZE - Data structure normalization
  // ============================================================================
  normalize: {
    /**
     * Parse date strings to Date objects
     */
    dates(obj, dateFields) {
      if (!obj || !dateFields || dateFields.length === 0) {
        return obj;
      }

      const result = { ...obj };
      for (const field of dateFields) {
        if (result[field] && typeof result[field] === 'string') {
          result[field] = new Date(result[field]);
        }
      }
      return result;
    },

    /**
     * Ensure all items have unique IDs
     */
    ids(items, prefix = 'item') {
      if (!Array.isArray(items)) return items;

      return items.map((item, index) => {
        if (item.id) return item;
        return { ...item, id: `${prefix}-${index}` };
      });
    }
  },

  // ============================================================================
  // 4. TRANSFORM - Type-specific transformations to render-ready configs
  // ============================================================================
  transform: {
    /**
     * Transform chart schema for D3
     */
    chart(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      return {
        type: 'chart',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        chartType: config.chartType || 'bar',
        xAxis: config.xAxis?.field,
        xAxisLabel: A2UIAdapter.resolve.value(config.xAxis?.label),
        series: (config.series || []).map(s => ({
          field: s.field,
          label: A2UIAdapter.resolve.value(s.label) || s.field
        })),
        data: data.rows || [],
        options: {
          showLegend: config.showLegend !== false,
          stacked: config.stacked || false,
          showValues: config.showValues || false
        }
      };
    },

    /**
     * Transform graph schema for D3 force layout
     */
    graph(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      return {
        type: 'graph',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        nodes: (data.nodes || []).map(n => ({
          id: n.id,
          label: A2UIAdapter.resolve.value(n.label) || n.id,
          group: n.group || 'default',
          x: 0,
          y: 0
        })),
        links: (data.edges || []).map(e => ({
          source: e.source,
          target: e.target,
          label: A2UIAdapter.resolve.value(e.label) || ''
        })),
        options: {
          layout: config.layout || 'force',
          nodeSize: config.nodeSize || 20,
          linkDistance: config.linkDistance || 100
        }
      };
    },

    /**
     * Transform timeline schema for Event Calendar ResourceTimeline
     */
    timeline(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      return {
        type: 'timeline',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        defaultView: config.defaultView || 'resourceTimelineMonth',
        slotDuration: config.slotDuration || '1 day',
        resources: (data.resources || []).map(r => ({
          id: r.id,
          title: A2UIAdapter.resolve.value(r.title) || r.id,
          color: r.color
        })),
        events: (data.events || []).map(e => ({
          id: e.id,
          resourceId: e.resourceId,
          title: A2UIAdapter.resolve.value(e.title) || '',
          start: e.start,
          end: e.end,
          color: e.color,
          description: A2UIAdapter.resolve.value(e.description) || ''
        }))
      };
    },

    /**
     * Transform tree schema for D3 hierarchy
     */
    tree(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      const transformNode = (node) => {
        if (!node) return null;
        return {
          id: node.id,
          name: A2UIAdapter.resolve.value(node.name) || node.id || 'Node',
          value: node.value,
          children: node.children ? node.children.map(transformNode) : undefined
        };
      };

      return {
        type: 'tree',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        hierarchy: transformNode(data.hierarchy),
        options: {
          orientation: config.orientation || 'horizontal',
          collapsible: config.collapsible !== false
        }
      };
    },

    /**
     * Transform calendar schema for Event Calendar
     */
    calendar(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      return {
        type: 'calendar',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        defaultView: config.defaultView || 'dayGridMonth',
        events: (data.events || []).map(e => ({
          id: e.id,
          title: A2UIAdapter.resolve.value(e.title) || '',
          start: e.start,
          end: e.end || e.start,
          allDay: e.allDay !== false,
          color: e.color,
          description: A2UIAdapter.resolve.value(e.description) || ''
        }))
      };
    },

    /**
     * Transform flowchart schema for directed graph
     */
    flowchart(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      return {
        type: 'flowchart',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        nodes: (data.nodes || []).map(n => ({
          id: n.id,
          label: A2UIAdapter.resolve.value(n.label) || n.id,
          nodeType: n.nodeType || 'process'
        })),
        edges: (data.edges || []).map(e => ({
          from: e.from,
          to: e.to,
          label: A2UIAdapter.resolve.value(e.label) || ''
        })),
        direction: config.direction || 'TB'
      };
    },

    /**
     * Transform grid schema for AG Grid
     */
    grid(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      return {
        type: 'grid',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        columnDefs: (config.columns || []).map(col => ({
          field: col.field,
          headerName: A2UIAdapter.resolve.value(col.header) || col.field,
          sortable: col.sortable !== false,
          filter: col.filter !== false,
          width: col.width,
          flex: col.flex
        })),
        rowData: data.rows || [],
        options: {
          pagination: config.pagination !== false,
          pageSize: config.pageSize || 25
        }
      };
    },

    /**
     * Transform kanban schema for CSS layout
     */
    kanban(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      return {
        type: 'kanban',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        columns: (data.columns || []).map(col => ({
          id: col.id,
          title: A2UIAdapter.resolve.value(col.title) || col.id,
          color: col.color || '#6b7280'
        })),
        cards: (data.cards || []).map(card => ({
          id: card.id,
          columnId: card.columnId,
          title: A2UIAdapter.resolve.value(card.title) || '',
          description: A2UIAdapter.resolve.value(card.description) || '',
          tags: card.tags || []
        })),
        options: {
          columnWidth: config.columnWidth || 300
        }
      };
    },

    /**
     * Transform map schema for Leaflet
     */
    map(spec) {
      const config = spec.config || {};
      const data = spec.data || {};

      return {
        type: 'map',
        title: A2UIAdapter.resolve.value(spec.title) || '',
        center: config.center || [0, 0],
        zoom: config.zoom || 10,
        markers: (data.markers || []).map(m => ({
          id: m.id,
          lat: m.coordinates?.lat,
          lng: m.coordinates?.lng,
          label: A2UIAdapter.resolve.value(m.label) || '',
          popup: A2UIAdapter.resolve.value(m.popup) || ''
        }))
      };
    }
  },

  // ============================================================================
  // 5. MAIN ENTRY POINT
  // ============================================================================
  /**
   * Process a visualization specification
   * @param {Object} spec - The A2UI visualization specification
   * @param {Object} options - Processing options
   * @param {Object} options.context - Context for BoundValue path resolution
   * @param {boolean} options.lenient - Continue processing even with validation errors
   * @returns {{ data: Object|null, errors: Array, warnings: Array }}
   */
  process(spec, options = {}) {
    const result = {
      data: null,
      errors: [],
      warnings: []
    };

    // Validate
    const validation = this.validate.spec(spec);
    if (!validation.valid) {
      result.errors = validation.errors;
      if (!options.lenient) {
        return result;
      }
      // In lenient mode, continue but report errors
      result.warnings.push({ code: 'VALIDATION_WARNINGS', message: 'Proceeding despite validation errors' });
    }

    // Resolve BoundValues if context provided
    const resolved = options.context
      ? this.resolve.all(spec, options.context)
      : spec;

    // Transform to render-ready format
    const type = resolved.type;
    if (this.transform[type]) {
      try {
        result.data = this.transform[type](resolved);
      } catch (e) {
        result.errors.push({
          code: 'TRANSFORM_ERROR',
          message: `Failed to transform ${type}: ${e.message}`
        });
      }
    } else {
      result.errors.push({
        code: 'UNKNOWN_TYPE',
        message: `Unknown visualization type: ${type}`
      });
    }

    return result;
  }
};

// Export for browser and Node.js
if (typeof module !== 'undefined' && module.exports) {
  module.exports = A2UIAdapter;
}
