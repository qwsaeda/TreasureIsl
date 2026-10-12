# S1.4b: extract exact .Map records + sub_31424 event-id byte
# Control object layout (from sub_449C8/46294/44004 chain):
#   obj+20 = node*  ;  sub_31424(ctrl): byte = *( *(ctrl+12) + 22 ) = *(obj+20+22) = f3
#   event_id = (f3 >> 2) + 1   (when f3 <= 0xFF else saturates at 129)
# This script dumps, per UI scene:
#   - record count
#   - for each record: idx, f0..f5
# Output: ref/map_records.json
$files = Get-ChildItem 'C:\金银岛\data\*.Map' | Where-Object { $_.Name -notmatch '^glmapm' } | Sort-Object Name
$out = @{}
foreach ($f in $files) {
    $raw = [IO.File]::ReadAllBytes($f.FullName)
    $w = [uint32[]]::new($raw.Length / 4)
    for ($i = 0; $i -lt $w.Length; $i++) {
        $w[$i] = [BitConverter]::ToUInt32($raw, $i * 4)
    }
    $count = $w[4]
    $recs = @()
    for ($i = 0; $i -lt $count; $i++) {
        $b = 5 + $i * 6
        $f0 = $w[$b]; $f1 = $w[$b + 1]; $f2 = $w[$b + 2]; $f3 = $w[$b + 3]; $f4 = $w[$b + 4]; $f5 = $w[$b + 5]
        $ev = if ($f3 -le 0xFF) { [uint32](($f3 -shr 2) + 1) } else { [uint32]0xFFFFFFFF }
        $recs += [pscustomobject]@{
            idx = $i; f0 = $f0; f1 = $f1; f2 = $f2; f3 = $f3; f4 = $f4; f5 = $f5;
            event_id = $ev; event_byte = if ($f3 -le 0xFF) { [uint32]$f3 } else { [uint32]0xFF }
        }
    }
    $out[$f.Name -replace '\.Map$',''] = [pscustomobject]@{
        name = $f.BaseName
        total = $count
        f0 = @{}
        f1 = @{}
        f3 = @{}
        f5 = @{}
        records = $recs
    }
    foreach ($r in $recs) {
        $out[$f.BaseName].f0[$r.f0] = $true
        $out[$f.BaseName].f1[$r.f1] = $true
        $out[$f.BaseName].f3[$r.f3] = $true
        $out[$f.BaseName].f5[$r.f5] = $true
    }
}
# Print key stats
"=== per-scene: total, f3 distinct, f5 top, f0 top ==="
foreach ($k in ($out.Keys | Sort-Object)) {
    $o = $out[$k]
    $f3nz = $o.f3.Keys | Where-Object { $_ -ne 0 }
    "scene {0,-14} count={1,4}  f3!=0: {2}" -f $k, $o.total, (($f3nz | ForEach-Object { [int32]$_ }) -join ',')
}
$all = @()
foreach ($k in $out.Keys) { $all += $out[$k].records }
"`n=== global f3 frequency (non-zero) ==="
$all | Group-Object f3 | Where-Object { $_.Name -ne '0' } | Sort-Object { [uint32]$_.Name } | ForEach-Object { "  f3={0,-8} x{1}" -f [int32]$_.Name, $_.Count }
"`n=== global f3 event_id frequency (f3<=0xFF) ==="
$all | Where-Object { $_.f3 -le 0xFF } | Group-Object event_id | Sort-Object { [uint32]$_.Name } | ForEach-Object { "  ev={0,-4} x{1}" -f [int32]$_.Name, $_.Count }
