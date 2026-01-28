/**
 * A2UI JSON → D3 config transformer
 * Transforms A2UI visualization schemas to D3-ready configurations
 */
const A2UIAdapter = {
  /**
   * Transform graph schema to D3 force layout config
   */
  graph: (spec) => ({
    nodes: spec.nodes.map(n => ({ ...n, x: 0, y: 0 })),
    links: spec.links.map(l => ({ ...l })),
    title: spec.title || ''
  }),

  /**
   * Transform timeline schema with Date objects
   */
  timeline: (spec) => ({
    events: spec.events.map(e => ({
      ...e,
      date: e.date ? new Date(e.date) : null,
      start: e.start ? new Date(e.start) : null,
      end: e.end ? new Date(e.end) : null
    })),
    orientation: spec.orientation || 'horizontal',
    title: spec.title || ''
  }),

  /**
   * Transform tree schema to D3 hierarchy
   */
  tree: (spec) => ({
    hierarchy: spec.root,
    title: spec.title || ''
  }),

  /**
   * Transform chart schema for bar/line/area/pie
   */
  chart: (spec) => ({
    data: spec.data,
    type: spec.chartType || 'bar',
    xAxis: spec.xAxis,
    series: spec.series,
    title: spec.title || ''
  }),

  /**
   * Transform calendar schema for heatmap
   */
  calendar: (spec) => ({
    year: spec.year,
    data: new Map(spec.data.map(d => [d.date, { value: d.value, label: d.label }])),
    colorScale: spec.colorScale || ['#ebedf0', '#9be9a8', '#40c463', '#30a14e', '#216e39'],
    title: spec.title || ''
  }),

  /**
   * Transform flowchart schema for directed graph
   */
  flowchart: (spec) => ({
    nodes: spec.nodes,
    edges: spec.edges,
    direction: spec.direction || 'TB',
    title: spec.title || ''
  }),

  /**
   * Transform grid schema for AG Grid (pass-through)
   */
  grid: (spec) => ({
    columnDefs: spec.columnDefs,
    rowData: spec.rowData,
    title: spec.title || ''
  }),

  /**
   * Transform kanban schema for CSS layout
   */
  kanban: (spec) => ({
    columns: spec.columns,
    cards: spec.cards,
    title: spec.title || ''
  }),

  /**
   * Transform map schema for Leaflet
   */
  map: (spec) => ({
    center: spec.center,
    zoom: spec.zoom || 10,
    markers: spec.markers || [],
    title: spec.title || ''
  })
};

// Export for browser and Node.js
if (typeof module !== 'undefined' && module.exports) {
  module.exports = A2UIAdapter;
}
