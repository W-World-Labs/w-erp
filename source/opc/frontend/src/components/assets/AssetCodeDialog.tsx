import { useState, useEffect, useRef, useCallback } from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
    Download,
    Printer,
    QrCode,
    Barcode,
    Copy,
    Check,
    Edit,
} from 'lucide-react';
import { toast } from 'sonner';
import QRCodeLib from 'qrcode';

// ---- Code128 barcode renderer (self-contained, no external deps) ----

const CODE128_START_B = 104;
const CODE128_STOP = 106;

const CODE128_PATTERNS: number[][] = [
    [2, 1, 2, 2, 2, 2], [2, 2, 2, 1, 2, 2], [2, 2, 2, 2, 2, 1], [1, 2, 1, 2, 2, 3], [1, 2, 1, 3, 2, 2],
    [1, 3, 1, 2, 2, 2], [1, 2, 2, 2, 1, 3], [1, 2, 2, 3, 1, 2], [1, 3, 2, 2, 1, 2], [2, 2, 1, 2, 1, 3],
    [2, 2, 1, 3, 1, 2], [2, 3, 1, 2, 1, 2], [1, 1, 2, 2, 3, 2], [1, 2, 2, 1, 3, 2], [1, 2, 2, 2, 3, 1],
    [1, 1, 3, 2, 2, 2], [1, 2, 3, 1, 2, 2], [1, 2, 3, 2, 2, 1], [2, 2, 3, 2, 1, 1], [2, 2, 1, 1, 3, 2],
    [2, 2, 1, 2, 3, 1], [2, 1, 3, 2, 1, 2], [2, 2, 3, 1, 1, 2], [3, 1, 2, 1, 3, 1], [3, 1, 1, 2, 2, 2],
    [3, 2, 1, 1, 2, 2], [3, 2, 1, 2, 2, 1], [3, 1, 2, 2, 1, 2], [3, 2, 2, 1, 1, 2], [3, 2, 2, 2, 1, 1],
    [2, 1, 2, 1, 2, 3], [2, 1, 2, 3, 2, 1], [2, 3, 2, 1, 2, 1], [1, 1, 1, 3, 2, 3], [1, 3, 1, 1, 2, 3],
    [1, 3, 1, 3, 2, 1], [1, 1, 2, 3, 1, 3], [1, 3, 2, 1, 1, 3], [1, 3, 2, 3, 1, 1], [2, 1, 1, 3, 1, 3],
    [2, 3, 1, 1, 1, 3], [2, 3, 1, 3, 1, 1], [1, 1, 2, 1, 3, 3], [1, 1, 2, 3, 3, 1], [1, 3, 2, 1, 3, 1],
    [1, 1, 3, 1, 2, 3], [1, 1, 3, 3, 2, 1], [1, 3, 3, 1, 2, 1], [3, 1, 3, 1, 2, 1], [2, 1, 1, 3, 3, 1],
    [2, 3, 1, 1, 3, 1], [2, 1, 3, 1, 1, 3], [2, 1, 3, 3, 1, 1], [2, 1, 3, 1, 3, 1], [3, 1, 1, 1, 2, 3],
    [3, 1, 1, 3, 2, 1], [3, 3, 1, 1, 2, 1], [3, 1, 2, 1, 1, 3], [3, 1, 2, 3, 1, 1], [3, 3, 2, 1, 1, 1],
    [3, 1, 4, 1, 1, 1], [2, 2, 1, 4, 1, 1], [4, 3, 1, 1, 1, 1], [1, 1, 1, 2, 2, 4], [1, 1, 1, 4, 2, 2],
    [1, 2, 1, 1, 2, 4], [1, 2, 1, 4, 2, 1], [1, 4, 1, 1, 2, 2], [1, 4, 1, 2, 2, 1], [1, 1, 2, 2, 1, 4],
    [1, 1, 2, 4, 1, 2], [1, 2, 2, 1, 1, 4], [1, 2, 2, 4, 1, 1], [1, 4, 2, 1, 1, 2], [1, 4, 2, 2, 1, 1],
    [2, 4, 1, 2, 1, 1], [2, 2, 1, 1, 1, 4], [4, 1, 3, 1, 1, 1], [2, 4, 1, 1, 1, 2], [1, 3, 4, 1, 1, 1],
    [1, 1, 1, 2, 4, 2], [1, 2, 1, 1, 4, 2], [1, 2, 1, 2, 4, 1], [1, 1, 4, 2, 1, 2], [1, 2, 4, 1, 1, 2],
    [1, 2, 4, 2, 1, 1], [4, 1, 1, 2, 1, 2], [4, 2, 1, 1, 1, 2], [4, 2, 1, 2, 1, 1], [2, 1, 2, 1, 4, 1],
    [2, 1, 4, 1, 2, 1], [4, 1, 2, 1, 2, 1], [1, 1, 1, 1, 4, 3], [1, 1, 1, 3, 4, 1], [1, 3, 1, 1, 4, 1],
    [1, 1, 4, 1, 1, 3], [1, 1, 4, 3, 1, 1], [4, 1, 1, 1, 1, 3], [4, 1, 1, 3, 1, 1], [1, 1, 3, 1, 4, 1],
    [1, 1, 4, 1, 3, 1], [3, 1, 1, 1, 4, 1], [4, 1, 1, 1, 3, 1], [2, 1, 1, 4, 1, 2], [2, 1, 1, 2, 1, 4],
    [2, 1, 1, 2, 3, 2], [2, 3, 3, 1, 1, 1, 2],
];

function encodeCode128B(text: string): number[] {
    const codes: number[] = [CODE128_START_B];
    let checksum = CODE128_START_B;
    for (let i = 0; i < text.length; i++) {
        const code = text.charCodeAt(i) - 32;
        codes.push(code);
        checksum += code * (i + 1);
    }
    codes.push(checksum % 103);
    codes.push(CODE128_STOP);
    return codes;
}

function renderBarcode(canvas: HTMLCanvasElement, text: string, opts: { height?: number; moduleWidth?: number } = {}) {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const height = opts.height || 60;
    const moduleWidth = opts.moduleWidth || 2;
    const codes = encodeCode128B(text);

    // Calculate total width
    let totalModules = 0;
    for (const code of codes) {
        const pattern = CODE128_PATTERNS[code];
        if (pattern) {
            for (const m of pattern) totalModules += m;
        }
    }

    const padding = 10;
    canvas.width = totalModules * moduleWidth + padding * 2;
    canvas.height = height + 24; // extra space for text

    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    let x = padding;
    for (const code of codes) {
        const pattern = CODE128_PATTERNS[code];
        if (!pattern) continue;
        for (let j = 0; j < pattern.length; j++) {
            const w = pattern[j] * moduleWidth;
            if (j % 2 === 0) {
                ctx.fillStyle = '#000000';
                ctx.fillRect(x, 0, w, height);
            }
            x += w;
        }
    }

    // Draw text below barcode
    ctx.fillStyle = '#000000';
    ctx.font = '12px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(text, canvas.width / 2, height + 16);
}

// ---- Component ----

interface AssetInfo {
    id: string;
    name: string;
    serial_number?: string | null;
    asset_type?: string;
    asset_type_badge?: {
        label: string;
        variant?: 'default' | 'secondary' | 'outline' | 'destructive';
        class_name?: string;
    } | null;
    location?: string | null;
    category_name?: string | null;
    status?: string;
    status_badge?: {
        label: string;
        variant?: 'default' | 'secondary' | 'outline' | 'destructive';
        class_name?: string;
    } | null;
}

interface AssetCodeDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    asset: AssetInfo | null;
    onEdit?: (asset: AssetInfo) => void;
}

export function AssetCodeDialog({ open, onOpenChange, asset, onEdit }: AssetCodeDialogProps) {
    const [qrDataUrl, setQrDataUrl] = useState<string>('');
    const [copied, setCopied] = useState(false);
    const barcodeCanvasRef = useRef<HTMLCanvasElement>(null);
    const printRef = useRef<HTMLDivElement>(null);

    // 系统码前缀 — 用于扫码时区分“我们的码” vs “外部码”
    const BARCODE_PREFIX = 'OM:';
    const barcodeId = asset?.id?.slice(0, 12) || '';
    const barcodeEncoded = `${BARCODE_PREFIX}${barcodeId}`; // 编码内容（含前缀）
    const barcodeDisplay = asset?.serial_number || barcodeId; // 显示用的可读文本
    const assetTypeLabel = asset?.asset_type_badge?.label || '-';
    const statusBadge = asset && asset.status_badge ? asset.status_badge : {
        label: asset?.status || '-',
        variant: 'secondary' as const,
        class_name: '',
    };

    // Generate QR code
    useEffect(() => {
        if (!open || !asset) return;
        const qrPayload = JSON.stringify({
            type: 'asset',
            id: asset.id,
            name: asset.name,
            sn: asset.serial_number || '',
        });
        QRCodeLib.toDataURL(qrPayload, {
            width: 280,
            margin: 2,
            color: { dark: '#000000', light: '#ffffff' },
        }).then(setQrDataUrl).catch(console.error);
    }, [open, asset]);

    // Generate barcode
    useEffect(() => {
        if (!open || !barcodeEncoded || !barcodeCanvasRef.current) return;
        const timer = setTimeout(() => {
            if (barcodeCanvasRef.current) {
                renderBarcode(barcodeCanvasRef.current, barcodeEncoded, { height: 50, moduleWidth: 2 });
            }
        }, 100);
        return () => clearTimeout(timer);
    }, [open, barcodeEncoded]);

    const handleDownloadQR = useCallback(() => {
        if (!qrDataUrl || !asset) return;
        const link = document.createElement('a');
        link.href = qrDataUrl;
        link.download = `asset-${asset.serial_number || asset.id}-qr.png`;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        toast.success('二维码已下载');
    }, [qrDataUrl, asset]);

    const handleDownloadBarcode = useCallback(() => {
        if (!barcodeCanvasRef.current || !asset) return;
        const link = document.createElement('a');
        link.href = barcodeCanvasRef.current.toDataURL('image/png');
        link.download = `asset-${asset.serial_number || asset.id}-barcode.png`;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        toast.success('条形码已下载');
    }, [asset]);

    const handlePrint = useCallback(() => {
        const printContent = printRef.current;
        if (!printContent) return;

        const printWindow = window.open('', '_blank');
        if (!printWindow) {
            toast.error('无法打开打印窗口，请检查弹窗拦截设置');
            return;
        }

        printWindow.document.write(`
      <!DOCTYPE html>
      <html>
      <head>
        <title>资产标签 - ${asset?.name || ''}</title>
        <style>
          @page { size: 60mm 40mm; margin: 2mm; }
          body {
            margin: 0; padding: 4mm;
            font-family: -apple-system, 'Helvetica Neue', Arial, sans-serif;
            font-size: 10px; color: #000;
          }
          .label {
            border: 1px solid #ccc; border-radius: 4px; padding: 3mm;
            width: 54mm; box-sizing: border-box;
          }
          .company { font-size: 8px; color: #666; margin-bottom: 2mm; text-align: center; }
          .name { font-size: 13px; font-weight: 700; margin-bottom: 2mm; text-align: center; }
          .codes { display: flex; gap: 3mm; align-items: center; justify-content: center; margin: 2mm 0; }
          .qr img { width: 22mm; height: 22mm; }
          .barcode img { max-width: 28mm; height: auto; }
          .info { font-size: 8px; line-height: 1.6; }
          .info span { color: #666; }
        </style>
      </head>
      <body>
        <div class="label">
          <div class="company">OPC</div>
          <div class="name">${asset?.name || ''}</div>
          <div class="codes">
            <div class="qr"><img src="${qrDataUrl}" /></div>
            <div class="barcode"><img src="${barcodeCanvasRef.current?.toDataURL('image/png') || ''}" /></div>
          </div>
          <div class="info">
            <div><span>编号：</span>${asset?.serial_number || asset?.id?.slice(0, 8) || '-'}</div>
            <div><span>类型：</span>${assetTypeLabel} ${asset?.category_name ? '/ ' + asset.category_name : ''}</div>
            <div><span>位置：</span>${asset?.location || '-'}</div>
          </div>
        </div>
      </body>
      </html>
    `);
        printWindow.document.close();
        printWindow.onload = () => {
            printWindow.print();
            printWindow.close();
        };
        // Fallback for faster browsers
        setTimeout(() => {
            try {
                printWindow.print();
                printWindow.close();
            } catch (error) {
                console.warn('Asset print fallback failed', error);
            }
        }, 500);
    }, [asset, assetTypeLabel, qrDataUrl]);

    const handleCopyId = useCallback(() => {
        if (!asset) return;
        navigator.clipboard.writeText(asset.serial_number || asset.id);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
        toast.success('已复制到剪贴板');
    }, [asset]);

    if (!asset) return null;

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <QrCode className="h-5 w-5" />
                        资产标签 — {asset.name}
                    </DialogTitle>
                    <DialogDescription>
                        扫描二维码或条形码可快速识别此资产
                    </DialogDescription>
                </DialogHeader>

                {/* Asset info header */}
                <div className="flex items-center gap-3 px-1">
                    <Badge
                        variant={asset.asset_type_badge?.variant || 'outline'}
                        className={asset.asset_type_badge?.class_name}
                    >
                        {asset.asset_type_badge?.label || '未知类型'}
                    </Badge>
                    <Badge variant={statusBadge.variant} className={statusBadge.class_name}>
                        {statusBadge.label}
                    </Badge>
                    {asset.category_name && (
                        <span className="text-sm text-muted-foreground">{asset.category_name}</span>
                    )}
                    {onEdit && (
                        <Button
                            variant="outline"
                            size="sm"
                            className="ml-auto"
                            onClick={() => {
                                onOpenChange(false);
                                onEdit(asset);
                            }}
                        >
                            <Edit className="h-3.5 w-3.5 mr-1" />
                            编辑资产
                        </Button>
                    )}
                </div>

                <Tabs defaultValue="qr" className="mt-2">
                    <TabsList className="grid w-full grid-cols-3">
                        <TabsTrigger value="qr">
                            <QrCode className="h-4 w-4 mr-1" />
                            二维码
                        </TabsTrigger>
                        <TabsTrigger value="barcode">
                            <Barcode className="h-4 w-4 mr-1" />
                            条形码
                        </TabsTrigger>
                        <TabsTrigger value="label">
                            <Printer className="h-4 w-4 mr-1" />
                            标签
                        </TabsTrigger>
                    </TabsList>

                    {/* QR Code Tab */}
                    <TabsContent value="qr" className="flex flex-col items-center gap-4 py-4">
                        {qrDataUrl && (
                            <img src={qrDataUrl} alt="资产二维码" className="w-64 h-64 rounded-lg border p-2 bg-white" />
                        )}
                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            <span className="font-mono">{barcodeDisplay}</span>
                            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={handleCopyId}>
                                {copied ? <Check className="h-3 w-3 text-green-500" /> : <Copy className="h-3 w-3" />}
                            </Button>
                        </div>
                        <Button onClick={handleDownloadQR} className="w-full">
                            <Download className="mr-2 h-4 w-4" />
                            下载二维码
                        </Button>
                    </TabsContent>

                    {/* Barcode Tab */}
                    <TabsContent value="barcode" className="flex flex-col items-center gap-4 py-4">
                        <div className="bg-white rounded-lg border p-4">
                            <canvas ref={barcodeCanvasRef} />
                        </div>
                        <p className="text-xs text-muted-foreground">Code 128 · 可被标准扫码枪识别</p>
                        <Button onClick={handleDownloadBarcode} className="w-full">
                            <Download className="mr-2 h-4 w-4" />
                            下载条形码
                        </Button>
                    </TabsContent>

                    {/* Label Tab (print-ready) */}
                    <TabsContent value="label" className="py-4">
                        <div ref={printRef} className="border rounded-lg p-4 bg-white mx-auto" style={{ maxWidth: '280px' }}>
                            <p className="text-[10px] text-center text-muted-foreground mb-1">OPC</p>
                            <p className="text-base font-bold text-center mb-2">{asset.name}</p>
                            <div className="flex gap-2 items-start justify-center mb-2">
                                {qrDataUrl && (
                                    <img src={qrDataUrl} alt="QR" className="w-20 h-20" />
                                )}
                                <div className="flex-1 flex items-center justify-center">
                                    <canvas
                                        ref={(el) => {
                                            if (el && barcodeId) {
                                                renderBarcode(el, barcodeEncoded, { height: 36, moduleWidth: 1.5 });
                                            }
                                        }}
                                        className="max-w-full"
                                        style={{ maxHeight: '56px' }}
                                    />
                                </div>
                            </div>
                            <div className="text-[10px] space-y-0.5">
                                <div><span className="text-muted-foreground">编号：</span>{barcodeDisplay}</div>
                                <div><span className="text-muted-foreground">类型：</span>{asset.asset_type_badge?.label || '-'}{asset.category_name ? ` / ${asset.category_name}` : ''}</div>
                                <div><span className="text-muted-foreground">位置：</span>{asset.location || '-'}</div>
                            </div>
                        </div>
                        <Button onClick={handlePrint} className="w-full mt-4">
                            <Printer className="mr-2 h-4 w-4" />
                            打印资产标签
                        </Button>
                    </TabsContent>
                </Tabs>
            </DialogContent>
        </Dialog>
    );
}
