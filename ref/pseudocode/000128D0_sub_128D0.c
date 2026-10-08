/* ea:   0x000128D0 */
/* name: sub_128D0 */
/* size: 216 */
int __fastcall sub_128D0(int a1)
{
  _DWORD *v2; // r3
  _DWORD *v3; // r1
  int v4; // r1

  v2 = *(_DWORD **)(a1 + 1036);
  v3 = *(_DWORD **)(a1 + 1040);
  if ( v2 != v3 )
  {
    while ( *v2 != 0 )
    {
      if ( ++v2 == v3 )
        goto LABEL_6;
    }
    *v2 = *(_DWORD *)(a1 + 4 * (*(_DWORD *)(a1 + 1128) - *(_DWORD *)(a1 + 1132)) + 1168);
    v4 = *(_DWORD *)(a1 + 1024);
    ++*(_DWORD *)(a1 + 1264);
    *(_DWORD *)(a1 + 1024) = v4 - *(_DWORD *)(a1 + 1280);
    --*(_WORD *)(a1 + 1048);
  }
LABEL_6:
  sub_129D4(a1);
  *(_DWORD *)(a1 + 1248) = 10;
  *(_DWORD *)(a1 + 1128) = 1;
  *(_DWORD *)(a1 + 1132) = 0;
  return 1;
}

