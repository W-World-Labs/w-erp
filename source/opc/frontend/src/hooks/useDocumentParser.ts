import { useState, useEffect } from 'react';
import { api } from '@/integrations/api/client';

interface ParseResult {
    fields: Record<string, any>;
    document_type: string;
    success: boolean;
    parse_source?: string;
    warnings?: string[];
}

interface AiCapabilities {
    document_parser_enabled: boolean;
    document_parser_backend?: string;
}

const MAX_IMAGE_BYTES = 10 * 1024 * 1024;
/** Longest edge after downscale for vision OCR (keeps payload small). */
const MAX_VISION_EDGE = 1600;
const VISION_JPEG_QUALITY = 0.85;

const STRUCTURED_INVOICE_EXTS = ['.xml', '.ofd', '.pdf'];
const STRUCTURED_INVOICE_MIMES = [
    'application/xml',
    'text/xml',
    'application/pdf',
    'application/ofd',
    'application/octet-stream',
];

export function useDocumentParser(documentType: string) {
    const [isParsing, setIsParsing] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [enabled, setEnabled] = useState(true);

    useEffect(() => {
        let cancelled = false;
        (async () => {
            try {
                const caps = await api.get<AiCapabilities>('/api/ai/capabilities');
                if (!cancelled) {
                    // Invoice supports structured XML/OFD/PDF direct-read without vision backend.
                    setEnabled(
                        documentType === 'invoice' || !!caps?.document_parser_enabled,
                    );
                }
            } catch {
                if (!cancelled) setEnabled(true);
            }
        })();
        return () => { cancelled = true; };
    }, [documentType]);

    const parseFile = async (file: File): Promise<Record<string, any> | null> => {
        setIsParsing(true);
        setError(null);
        try {
            if (file.size > MAX_IMAGE_BYTES) {
                setError('文件超过 10MB，请压缩后重试');
                return null;
            }

            const isImage = file.type.startsWith('image/');
            const isStructuredInvoice =
                documentType === 'invoice' && isStructuredInvoiceFile(file);

            if (!isImage && !isStructuredInvoice) {
                setError(
                    documentType === 'invoice'
                        ? '支持图片、PDF、XML、OFD 电子发票文件'
                        : '仅支持图片文件（image/*）',
                );
                return null;
            }

            let base64: string;
            if (isImage) {
                base64 = await preprocessImageForOcr(file);
            } else {
                base64 = await fileToBase64(file);
            }

            const result = await api.post<ParseResult>('/api/ai/parse-document', {
                file_base64: base64,
                document_type: documentType,
                filename: file.name,
            });
            if (result?.success && result?.fields) {
                // Attach non-field metadata so callers can surface warnings if needed.
                if (result.warnings?.length) {
                    result.fields.warnings = result.warnings;
                }
                if (result.parse_source) {
                    result.fields.parse_source = result.parse_source;
                }
                return result.fields;
            }
            setError('解析失败：未获取到有效数据');
            return null;
        } catch (err: any) {
            const status = err?.status || err?.statusCode;
            if (status === 501) setError('文档识别未配置，请到 设置→文档识别 配置端点');
            else if (status === 413) setError('文件超过 10MB，请压缩后重试');
            else if (status === 415) setError('仅支持图片文件（image/*）');
            else if (status === 422) setError('结构化发票解析失败，请检查 XML/OFD/PDF 或改用清晰图片');
            else setError(err?.message || '文档解析失败，请稍后重试');
            return null;
        } finally {
            setIsParsing(false);
        }
    };

    return { parseFile, isParsing, error, enabled };
}

function isStructuredInvoiceFile(file: File): boolean {
    const name = file.name.toLowerCase();
    if (STRUCTURED_INVOICE_EXTS.some((ext) => name.endsWith(ext))) return true;
    if (STRUCTURED_INVOICE_MIMES.includes(file.type)) return true;
    return false;
}

function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result as string);
        reader.onerror = () => reject(new Error('文件读取失败'));
        reader.readAsDataURL(file);
    });
}

/**
 * Image preprocessing for vision OCR (#14):
 * - EXIF orientation via createImageBitmap({ imageOrientation: 'from-image' })
 * - Downscale longest edge to MAX_VISION_EDGE
 * - JPEG compress to reduce base64 payload
 */
export async function preprocessImageForOcr(file: File): Promise<string> {
    // Tiny / already-small files: still re-encode when possible so EXIF is applied.
    try {
        const bitmap = await createImageBitmap(file, {
            imageOrientation: 'from-image',
        } as ImageBitmapOptions);
        try {
            const { width, height } = bitmap;
            let tw = width;
            let th = height;
            const longest = Math.max(width, height);
            if (longest > MAX_VISION_EDGE) {
                const scale = MAX_VISION_EDGE / longest;
                tw = Math.max(1, Math.round(width * scale));
                th = Math.max(1, Math.round(height * scale));
            }
            const canvas = document.createElement('canvas');
            canvas.width = tw;
            canvas.height = th;
            const ctx = canvas.getContext('2d');
            if (!ctx) {
                bitmap.close();
                return fileToBase64(file);
            }
            ctx.drawImage(bitmap, 0, 0, tw, th);
            bitmap.close();
            const mime =
                file.type === 'image/png' && longest <= MAX_VISION_EDGE
                    ? 'image/png'
                    : 'image/jpeg';
            const dataUrl = canvas.toDataURL(
                mime,
                mime === 'image/jpeg' ? VISION_JPEG_QUALITY : undefined,
            );
            return dataUrl;
        } catch {
            bitmap.close();
            return fileToBase64(file);
        }
    } catch {
        // Fallback when createImageBitmap / EXIF not available.
        return fileToBase64(file);
    }
}
