# Gera os icones do NR Manager (PNG + ICO) sem depender de ferramentas externas.
# Uso: powershell -ExecutionPolicy Bypass -File scripts\gen-icons.ps1
Add-Type -AssemblyName System.Drawing

$outDir = Join-Path $PSScriptRoot "..\src-tauri\icons"
$outDir = [System.IO.Path]::GetFullPath($outDir)
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

function New-NrBitmap([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

    $rect = New-Object System.Drawing.Rectangle(0, 0, $size, $size)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $r = [int]($size * 0.22)
    $path.AddArc(0, 0, $r, $r, 180, 90)
    $path.AddArc($size - $r, 0, $r, $r, 270, 90)
    $path.AddArc($size - $r, $size - $r, $r, $r, 0, 90)
    $path.AddArc(0, $size - $r, $r, $r, 90, 90)
    $path.CloseFigure()

    $bg = New-Object System.Drawing.Drawing2D.LinearGradientBrush($rect,
        [System.Drawing.Color]::FromArgb(255, 22, 24, 28),
        [System.Drawing.Color]::FromArgb(255, 11, 12, 14), 55.0)
    $g.FillPath($bg, $path)

    # Faixa vermelha AMD com gradiente
    $bandH = [int]($size * 0.20)
    $bandRect = New-Object System.Drawing.Rectangle(0, [int]($size * 0.62), $size, $bandH)
    $band = New-Object System.Drawing.Drawing2D.LinearGradientBrush($bandRect,
        [System.Drawing.Color]::FromArgb(255, 237, 28, 36),
        [System.Drawing.Color]::FromArgb(255, 176, 19, 26), 0.0)
    $g.SetClip($path)
    $g.FillRectangle($band, $bandRect)
    $g.ResetClip()

    # Letras "NR"
    $fontSize = $size * 0.42
    $font = New-Object System.Drawing.Font("Segoe UI", $fontSize, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $textRect = New-Object System.Drawing.RectangleF(0, -($size * 0.05), $size, $size)
    $g.DrawString("NR", $font, [System.Drawing.Brushes]::White, $textRect, $sf)

    $g.Dispose()
    return $bmp
}

function Save-Png([System.Drawing.Bitmap]$bmp, [string]$file) {
    $bmp.Save($file, [System.Drawing.Imaging.ImageFormat]::Png)
}

# PNGs exigidos pelo bundle
Save-Png (New-NrBitmap 32)  (Join-Path $outDir "32x32.png")
Save-Png (New-NrBitmap 128) (Join-Path $outDir "128x128.png")
Save-Png (New-NrBitmap 256) (Join-Path $outDir "128x128@2x.png")
Save-Png (New-NrBitmap 512) (Join-Path $outDir "icon.png")

# ICO com PNG embutido (256x256)
$png256 = New-NrBitmap 256
$ms = New-Object System.IO.MemoryStream
$png256.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
$pngBytes = $ms.ToArray()
$ms.Dispose()

$icoPath = Join-Path $outDir "icon.ico"
$fs = [System.IO.File]::Create($icoPath)
$bw = New-Object System.IO.BinaryWriter($fs)
# ICONDIR
$bw.Write([UInt16]0)   # reserved
$bw.Write([UInt16]1)   # type = icon
$bw.Write([UInt16]1)   # count
# ICONDIRENTRY
$bw.Write([Byte]0)     # width 0 => 256
$bw.Write([Byte]0)     # height 0 => 256
$bw.Write([Byte]0)     # colors
$bw.Write([Byte]0)     # reserved
$bw.Write([UInt16]1)   # planes
$bw.Write([UInt16]32)  # bpp
$bw.Write([UInt32]$pngBytes.Length)
$bw.Write([UInt32]22)  # offset
$bw.Write($pngBytes)
$bw.Flush()
$bw.Close()

Write-Host "Icones gerados em $outDir"
