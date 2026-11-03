param(
    [Parameter(Mandatory)] [string] $Exe,
    [Parameter(Mandatory)] [string] $Out
)

$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force $Out | Out-Null
Add-Type -AssemblyName System.Drawing, System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Threading;

public static class Input {
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern void mouse_event(uint flags, int dx, int dy, int data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern uint GetDpiForSystem();

    public static void Move(int x, int y) {
        SetCursorPos(x, y);
        mouse_event(0x0001, 0, 0, 0, UIntPtr.Zero);
    }

    static void Press(int x, int y, uint down, uint up) {
        Move(x, y);
        Thread.Sleep(40);
        mouse_event(down, 0, 0, 0, UIntPtr.Zero);
        Thread.Sleep(60);
        mouse_event(up, 0, 0, 0, UIntPtr.Zero);
    }

    public static void Click(int x, int y) { Press(x, y, 0x0002, 0x0004); }
    public static void Middle(int x, int y) { Press(x, y, 0x0020, 0x0040); }

    public static void Wheel(int x, int y, int delta) {
        Move(x, y);
        Thread.Sleep(40);
        mouse_event(0x0800, 0, 0, delta, UIntPtr.Zero);
    }
}
'@

[Input]::SetProcessDPIAware() | Out-Null
$scale = [Input]::GetDpiForSystem() / 96.0
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$cx = $screen.X + [int]($screen.Width / 2)
$top = $screen.Y
"layar $($screen.Width)x$($screen.Height), skala $scale"

function Px([double] $dp) { [int][math]::Round($dp * $scale) }

function Shot([string] $name) {
    $w = [math]::Min($screen.Width, (Px 640))
    $h = Px 300
    $x = $cx - [int]($w / 2)
    $bmp = New-Object System.Drawing.Bitmap $w, $h
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $op = [System.Drawing.CopyPixelOperation]([int][System.Drawing.CopyPixelOperation]::SourceCopy -bor [int][System.Drawing.CopyPixelOperation]::CaptureBlt)
    $g.CopyFromScreen($x, $top, 0, 0, (New-Object System.Drawing.Size $w, $h), $op)
    $bmp.Save((Join-Path $Out "$name.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
    "foto $name"
}

function Wait([int] $ms) { Start-Sleep -Milliseconds $ms }

function Away { [Input]::Move($cx, $top + (Px 520)) }

$config = Join-Path $env:APPDATA 'Notchi\config.toml'
$log = Join-Path $env:LOCALAPPDATA 'Notchi\notchi.log'
Remove-Item $config, $log -ErrorAction SilentlyContinue

Away
$app = Start-Process -FilePath $Exe -PassThru
Wait 4000
if ($app.HasExited) { throw "Notchi keluar saat mulai (kode $($app.ExitCode))" }
Shot '01-autohide-diam'

[Input]::Move($cx, $top + 1)
Wait 1300
Shot '02-peek-idle'

$peekLeft = $cx - (Px 126)
[Input]::Move($peekLeft + (Px 20), $top + (Px 18))
Wait 250
[Input]::Click($peekLeft + (Px 20), $top + (Px 18))
Wait 220
Shot '03-peek-boop'

[Input]::Click($cx + (Px 30), $top + (Px 18))
Wait 1100
Shot '04-kartu-home'

$cardLeft = $cx - (Px 185.5)
[Input]::Click($cardLeft + (Px 117), $top + (Px 99))
Wait 1100
Shot '05-kartu-timer'

1..3 | ForEach-Object { [Input]::Wheel($cx, $top + (Px 50), 120); Wait 120 }
Wait 400
Shot '06-timer-scroll-tambah-3-menit'

[Input]::Middle($cx, $top + (Px 50))
Wait 500
Shot '07-timer-jeda-klik-tengah'
[Input]::Middle($cx, $top + (Px 50))
Wait 300

Away
Wait 2600
Shot '08-autohide-timer-masuk'

(Get-Content $config -Raw) -replace 'behavior = "autohide"', 'behavior = "sticky"' | Set-Content $config -NoNewline
Wait 1800
Shot '09-sticky-desktop'

$editor = Start-Process -FilePath notepad.exe -WindowStyle Maximized -PassThru
Wait 2500
Away
Wait 1200
Shot '10-sticky-tertutup-jendela'

[Input]::Move($cx, $top + 1)
Wait 1300
Shot '11-tertutup-hover-muncul'
Away
Wait 1500
Shot '12-tertutup-masuk-lagi'

Stop-Process -Id $editor.Id -ErrorAction SilentlyContinue
Wait 1500
[Input]::Click($cx, $top + (Px 18))
Wait 1100
Shot '13-kartu-timer-sticky'
[Input]::Click($cardLeft + (Px 331), $top + (Px 56))
Wait 1200
[Input]::Move($cx, $top + (Px 10))
Wait 1200
Shot '14-sticky-peek-idle'
Away
Wait 1200
Shot '15-sticky-idle'

if ($app.HasExited) { throw "Notchi berhenti di tengah uji (kode $($app.ExitCode))" }
Stop-Process -Id $app.Id
"Notchi tetap berjalan sepanjang uji"
if (Test-Path $log) {
    Copy-Item $log (Join-Path $Out 'notchi.log')
    "--- notchi.log ---"
    Get-Content $log
}
