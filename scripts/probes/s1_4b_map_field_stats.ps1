# S1.4b 扫描：UI 场景 .Map 控件记录 6 字段全量统计
$files = Get-ChildItem data\*.Map | Where-Object { $_.Name -notmatch '^glmapm' } | Sort-Object Name
$all = @()
foreach ($f in $files) {
    $raw = [IO.File]::ReadAllBytes($f.FullName)
    $w = [uint32[]]::new($raw.Length / 4)
    for ($i = 0; $i -lt $w.Length; $i++) {
        $w[$i] = [BitConverter]::ToUInt32($raw, $i * 4)
    }
    $count = $w[4]
    for ($i = 0; $i -lt $count; $i++) {
        $b = 5 + $i * 6
        $all += [pscustomobject]@{
            scene = $f.Name -replace '\.Map$', ''
            idx   = $i
            f0 = $w[$b]; f1 = $w[$b + 1]; f2 = $w[$b + 2]; f3 = $w[$b + 3]; f4 = $w[$b + 4]; f5 = $w[$b + 5]
        }
    }
}
"total records: $($all.Count)  scenes: $($files.Count)"

function DistinctLine([string]$label, [object[]]$vals, [string]$scene) {
    $g = $vals | Group-Object | Sort-Object { [uint32]$_.Name }
    $parts = foreach ($x in $g) { if ($scene) { "${x.Name}($($x.Count))" } else { "$($x.Name):$($x.Count)" } }
    "  $label[$scene] : $($parts -join ' ')"
}

"`n=== f0 distinct ==="
DistinctLine 'f0' $all.f0 ''
"`n=== f2 distinct ==="
DistinctLine 'f2' $all.f2 ''
"`n=== f4 distinct ==="
DistinctLine 'f4' $all.f4 ''
"`n=== f3 distinct ==="
DistinctLine 'f3' $all.f3 ''
"`n=== f5 distinct ==="
DistinctLine 'f5' $all.f5 ''
"`n=== f1 distinct ==="
DistinctLine 'f1' $all.f1 ''

"`n=== per-scene f3 non-zero (f3 = sub_31424 <<8 低字节候选) ==="
foreach ($f in $files) {
    $name = $f.Name -replace '\.Map$', ''
    $recs = @($all | Where-Object { $_.scene -eq $name })
    $nz = @($recs | Where-Object { $_.f3 -ne 0 })
    if ($nz.Count -gt 0) {
        $pairs = foreach ($r in $nz) { "idx$($r.idx):f3=$($r.f3),f0=$($r.f0),f1=$($r.f1),f2=$($r.f2),f5=$($r.f5)" }
        "  $name => $($pairs -join ' | ')"
    } else {
        "  $name => (no f3!=0)"
    }
}

"`n=== per-scene f5 distinct ==="
foreach ($f in $files) {
    $name = $f.Name -replace '\.Map$', ''
    $recs = @($all | Where-Object { $_.scene -eq $name })
    DistinctLine 'f5' @($recs.f5) $name
}
