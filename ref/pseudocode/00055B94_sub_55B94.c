/* ea:   0x00055B94 */
/* name: sub_55B94 */
/* size: 312 */
int __fastcall sub_55B94(int a1, int a2, int a3)
{
  int v6; // r0
  unsigned __int16 *v7; // r4
  int v8; // r2
  int v9; // r0
  __int16 v10; // r0
  int v11; // r0
  bool v12; // zf
  int v13; // r0
  int result; // r0

  if ( dword_7FB6C == 2 )
  {
    v6 = 3;
  }
  else
  {
    v6 = 1;
    if ( *(_DWORD *)(a1 + 40) != *(_DWORD *)(a1 + 44) )
      v6 = 2;
  }
  *(_DWORD *)a2 = v6;
  v7 = (unsigned __int16 *)(a1 + 136);
  *(_DWORD *)(a2 + 4) = *(_DWORD *)(a1 + 8);
  *(_DWORD *)(a2 + 12) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 4 * *(_DWORD *)(a1 + 216) + 80);
  v8 = *(_DWORD *)(a3 + 8);
  *(_DWORD *)(a2 + 20) = v8;
  v9 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 4 * v8 + 80);
  *(_DWORD *)(a2 + 28) = 0;
  *(_DWORD *)(a2 + 24) = v9;
  *(_DWORD *)(a2 + 8) = *(_DWORD *)(a1 + 36);
  *(_DWORD *)(a2 + 16) = *(_DWORD *)(a3 + 36);
  *(_WORD *)(a2 + 32) = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v10 = sub_16C9C(v7, 1);
  *(_WORD *)(a2 + 36) = 1;
  *(_WORD *)(a2 + 34) = v10;
  *(_WORD *)(a2 + 38) = (unsigned int)(sub_16C9C((unsigned __int16 *)(a3 + 136), 0) + 1) >> 1;
  *(_WORD *)(a2 + 40) = (unsigned int)(sub_16C9C((unsigned __int16 *)(a3 + 136), 1) + 1) >> 1;
  v11 = *(_DWORD *)(a3 + 176);
  *(_WORD *)(a2 + 44) = 0;
  v12 = v11 == 8;
  v13 = 200;
  if ( !v12 )
    v13 = 100;
  result = v13 << 16;
  *(_WORD *)(a2 + 42) = HIWORD(result);
  return result;
}

