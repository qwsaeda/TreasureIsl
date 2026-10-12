/* ea:   0x00017BC8 */
/* name: sub_17BC8 */
/* size: 180 */
int __fastcall sub_17BC8(int a1, _DWORD *a2)
{
  unsigned int v4; // r0
  int v5; // r4
  unsigned __int8 v6; // r1
  int v7; // r0
  bool v8; // zf
  int v9; // r2
  int v10; // r3
  int v11; // r1
  int v12; // r0
  void (__fastcall *v13)(int, _DWORD, int, int); // r4

  v4 = *(unsigned __int8 *)(a1 + 49);
  if ( v4 >= 4 )
    return 0;
  v5 = *(_DWORD *)(a1 + 8);
  v6 = v4 + 1;
  v7 = *(_DWORD *)(a1 + 44);
  *(_BYTE *)(a1 + 49) = v6;
  v8 = v7 == 1;
  v9 = *(_DWORD *)(a1 + 12);
  v10 = *(_DWORD *)(a1 + 16);
  v11 = *a2 + 4 * v6;
  v12 = v5;
  v13 = *(void (__fastcall **)(int, _DWORD, int, int))(*(_DWORD *)v5 + 36);
  if ( v8 )
    v13(v12, *(_DWORD *)(v11 + 8), v9, v10);
  else
    v13(v12, *(_DWORD *)(v11 + 24), v9, v10);
  if ( *(_BYTE *)(a1 + 50) == 1 && *(unsigned __int8 *)(a1 + 49) > 1u )
    sub_17C7C(off_811A0, 3);
  return 1;
}

