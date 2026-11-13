// Typed profiling span builders for the xRadar JS SDK.
// Mirrors sdk/rust/xradar (canonical) + sdk/python/xradar — field names are
// identical (snake_case) and wire format must stay consistent across all SDKs.
//
// Each builder accepts a data object and returns a plain object with a
// `span_kind` discriminator plus all non-null/undefined fields from `data`.

/**
 * Filter out null and undefined values from an object so optional fields
 * are omitted from the wire payload entirely.
 * @param {Record<string, unknown>} obj
 * @returns {Record<string, unknown>}
 */
function omitNulls(obj) {
  const result = {};
  for (const [key, value] of Object.entries(obj)) {
    if (value !== null && value !== undefined) {
      result[key] = value;
    }
  }
  return result;
}

/**
 * Build an LLM inference profiling span.
 * @param {{ model: string, input_tokens?: number|null, output_tokens?: number|null, ttft_ms?: number|null, total_latency_ms?: number|null, tokens_per_second?: number|null, supplier?: string|null, batch_size?: number|null, kv_cache_hit_ratio?: number|null, cost_usd?: number|null, gpu_id?: string|null, stop_reason?: string|null }} data
 * @returns {Record<string, unknown>}
 */
export function buildLlmInferenceSpan(data) {
  return omitNulls({
    span_kind: 'llm.inference',
    model: data.model,
    input_tokens: data.input_tokens,
    output_tokens: data.output_tokens,
    ttft_ms: data.ttft_ms,
    total_latency_ms: data.total_latency_ms,
    tokens_per_second: data.tokens_per_second,
    supplier: data.supplier,
    batch_size: data.batch_size,
    kv_cache_hit_ratio: data.kv_cache_hit_ratio,
    cost_usd: data.cost_usd,
    gpu_id: data.gpu_id,
    stop_reason: data.stop_reason,
  });
}

/**
 * Build a network request profiling span.
 * @param {{ target_host: string, method: string, total_latency_ms?: number|null, dns_ms?: number|null, tcp_connect_ms?: number|null, tls_handshake_ms?: number|null, ttfb_ms?: number|null, transfer_ms?: number|null, response_status?: number|null, bytes_sent?: number|null, bytes_received?: number|null, tls_version?: string|null, tls_session_reused?: boolean|null, cache_status?: string|null, protocol?: string|null }} data
 * @returns {Record<string, unknown>}
 */
export function buildNetworkRequestSpan(data) {
  return omitNulls({
    span_kind: 'network.request',
    target_host: data.target_host,
    method: data.method,
    total_latency_ms: data.total_latency_ms,
    dns_ms: data.dns_ms,
    tcp_connect_ms: data.tcp_connect_ms,
    tls_handshake_ms: data.tls_handshake_ms,
    ttfb_ms: data.ttfb_ms,
    transfer_ms: data.transfer_ms,
    response_status: data.response_status,
    bytes_sent: data.bytes_sent,
    bytes_received: data.bytes_received,
    tls_version: data.tls_version,
    tls_session_reused: data.tls_session_reused,
    cache_status: data.cache_status,
    protocol: data.protocol,
  });
}

/**
 * Build a video pipeline profiling span.
 * @param {{ camera_id: string, stage: string, stage_latency_ms?: number|null, algorithm_id?: string|null, provider?: string|null, fps?: number|null, frame_width?: number|null, frame_height?: number|null, frames_dropped?: number|null, gpu_memory_mb?: number|null, npu_core_id?: number|null }} data
 * @returns {Record<string, unknown>}
 */
export function buildVideoPipelineSpan(data) {
  return omitNulls({
    span_kind: 'video.pipeline',
    camera_id: data.camera_id,
    stage: data.stage,
    stage_latency_ms: data.stage_latency_ms,
    algorithm_id: data.algorithm_id,
    provider: data.provider,
    fps: data.fps,
    frame_width: data.frame_width,
    frame_height: data.frame_height,
    frames_dropped: data.frames_dropped,
    gpu_memory_mb: data.gpu_memory_mb,
    npu_core_id: data.npu_core_id,
  });
}

/**
 * Build a pipeline stage profiling span.
 * @param {{ pipeline_id: string, stage_name: string, stage_latency_ms?: number|null, records_in?: number|null, records_out?: number|null, error_count?: number|null, quality_score?: number|null, storage_backend?: string|null, bytes_processed?: number|null }} data
 * @returns {Record<string, unknown>}
 */
export function buildPipelineStageSpan(data) {
  return omitNulls({
    span_kind: 'pipeline.stage',
    pipeline_id: data.pipeline_id,
    stage_name: data.stage_name,
    stage_latency_ms: data.stage_latency_ms,
    records_in: data.records_in,
    records_out: data.records_out,
    error_count: data.error_count,
    quality_score: data.quality_score,
    storage_backend: data.storage_backend,
    bytes_processed: data.bytes_processed,
  });
}

/**
 * Build an infra health profiling span.
 * @param {{ component: string, status: string, pool_total?: number|null, pool_idle?: number|null, pool_waiting?: number|null, buffer_depth?: number|null, flush_rate_per_sec?: number|null, uptime_seconds?: number|null, panic_count?: number|null, detection_latency_ms?: number|null, rules_matched?: number|null, cache_hit_ratio?: number|null }} data
 * @returns {Record<string, unknown>}
 */
export function buildInfraHealthSpan(data) {
  return omitNulls({
    span_kind: 'infra.health',
    component: data.component,
    status: data.status,
    pool_total: data.pool_total,
    pool_idle: data.pool_idle,
    pool_waiting: data.pool_waiting,
    buffer_depth: data.buffer_depth,
    flush_rate_per_sec: data.flush_rate_per_sec,
    uptime_seconds: data.uptime_seconds,
    panic_count: data.panic_count,
    detection_latency_ms: data.detection_latency_ms,
    rules_matched: data.rules_matched,
    cache_hit_ratio: data.cache_hit_ratio,
  });
}
