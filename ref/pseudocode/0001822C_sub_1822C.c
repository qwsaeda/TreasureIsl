/* ea:   0x0001822C */
/* name: sub_1822C */
/* size: 240 */
int __fastcall sub_1822C(int a1, _DWORD *a2)
{
  _DWORD *v4; // r2
  int v5; // t1
  char v6; // r0
  int v7; // r5
  int v8; // r4
  int v9; // r2
  int v10; // r3
  int v11; // r0

  if ( *(unsigned __int8 *)(a1 + 49) <= 1u )
    return 0;
  if ( *(unsigned __int8 *)(a1 + 49) + 1 <= sub_17B98(a1) )
  {
    v4 = *(_DWORD **)(a1 + 32);
    while ( v4 != *(_DWORD **)(a1 + 28) )
    {
      v5 = *--v4;
      if ( v5 != 0 )
      {
        *v4 = 0;
        break;
      }
    }
  }
  v6 = *(_BYTE *)(a1 + 49);
  v7 = (unsigned __int8)(v6 - 1);
  *(_BYTE *)(a1 + 49) = v7;
  if ( v6 != 1 )
  {
    v8 = *(_DWORD *)(a1 + 8);
    v9 = *(_DWORD *)(a1 + 12);
    v10 = *(_DWORD *)(a1 + 16);
    v11 = *a2 + 4 * v7;
    if ( *(_DWORD *)(a1 + 44) == 1 )
    {
      (*(void (__fastcall **)(_DWORD, _DWORD, int, int))(*(_DWORD *)v8 + 36))(
        *(_DWORD *)(a1 + 8),
        *(_DWORD *)(v11 + 8),
        v9,
        v10);
      return 1;
    }
    (*(void (__fastcall **)(_DWORD, _DWORD, int, int))(*(_DWORD *)v8 + 36))(
      *(_DWORD *)(a1 + 8),
      *(_DWORD *)(v11 + 24),
      v9,
      v10);
  }
  return 1;
}

