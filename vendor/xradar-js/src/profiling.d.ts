// TypeScript declarations for the xRadar profiling span builders.
// Field names are snake_case and identical to the Rust/Python SDK versions.

export interface LlmInferenceData {
    model: string;
    input_tokens?: number | null;
    output_tokens?: number | null;
    ttft_ms?: number | null;
    total_latency_ms?: number | null;
    tokens_per_second?: number | null;
    supplier?: string | null;
    batch_size?: number | null;
    kv_cache_hit_ratio?: number | null;
    cost_usd?: number | null;
    gpu_id?: string | null;
    stop_reason?: string | null;
}

export interface NetworkRequestData {
    target_host: string;
    method: string;
    total_latency_ms?: number | null;
    dns_ms?: number | null;
    tcp_connect_ms?: number | null;
    tls_handshake_ms?: number | null;
    ttfb_ms?: number | null;
    transfer_ms?: number | null;
    response_status?: number | null;
    bytes_sent?: number | null;
    bytes_received?: number | null;
    tls_version?: string | null;
    tls_session_reused?: boolean | null;
    cache_status?: string | null;
    protocol?: string | null;
}

export interface VideoPipelineData {
    camera_id: string;
    stage: string;
    stage_latency_ms?: number | null;
    algorithm_id?: string | null;
    provider?: string | null;
    fps?: number | null;
    frame_width?: number | null;
    frame_height?: number | null;
    frames_dropped?: number | null;
    gpu_memory_mb?: number | null;
    npu_core_id?: number | null;
}

export interface PipelineStageData {
    pipeline_id: string;
    stage_name: string;
    stage_latency_ms?: number | null;
    records_in?: number | null;
    records_out?: number | null;
    error_count?: number | null;
    quality_score?: number | null;
    storage_backend?: string | null;
    bytes_processed?: number | null;
}

export interface InfraHealthData {
    component: string;
    status: string;
    pool_total?: number | null;
    pool_idle?: number | null;
    pool_waiting?: number | null;
    buffer_depth?: number | null;
    flush_rate_per_sec?: number | null;
    uptime_seconds?: number | null;
    panic_count?: number | null;
    detection_latency_ms?: number | null;
    rules_matched?: number | null;
    cache_hit_ratio?: number | null;
}

export function buildLlmInferenceSpan(data: LlmInferenceData): Record<string, unknown>;
export function buildNetworkRequestSpan(data: NetworkRequestData): Record<string, unknown>;
export function buildVideoPipelineSpan(data: VideoPipelineData): Record<string, unknown>;
export function buildPipelineStageSpan(data: PipelineStageData): Record<string, unknown>;
export function buildInfraHealthSpan(data: InfraHealthData): Record<string, unknown>;
