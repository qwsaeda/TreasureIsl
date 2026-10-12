/* ea:   0x00046730 */
/* name: sub_46730 */
/* size: 340 */
int __fastcall sub_46730(int a1, int a2, int a3)
{
  __int16 v6; // r6
  int v7; // r0
  int v8; // r2
  int v9; // r0
  int v10; // r1
  int result; // r0

  v6 = 2;
  if ( dword_7FB6C == 2 )
  {
    v7 = 3;
  }
  else
  {
    v7 = 1;
    if ( *(_DWORD *)(a1 + 40) != *(_DWORD *)(a1 + 44) )
      v7 = 2;
  }
  *(_DWORD *)a2 = v7;
  *(_DWORD *)(a2 + 4) = *(_DWORD *)(a1 + 8);
  *(_DWORD *)(a2 + 12) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 4 * *(_DWORD *)(a1 + 8) + 80);
  *(_DWORD *)(a2 + 20) = *(_DWORD *)(a3 + 8);
  v8 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 84);
  *(_DWORD *)(a2 + 28) = 0;
  *(_DWORD *)(a2 + 24) = v8;
  *(_DWORD *)(a2 + 8) = *(_DWORD *)(a1 + 36);
  *(_DWORD *)(a2 + 16) = *(_DWORD *)(a3 + 36);
  *(_WORD *)(a2 + 32) = (unsigned int)(sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + 1) >> 1;
  *(_WORD *)(a2 + 34) = (unsigned int)sub_16C9C((unsigned __int16 *)(a1 + 136), 1) >> 1;
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v6 = 1;
  *(_WORD *)(a2 + 36) = v6;
  *(_WORD *)(a2 + 38) = (unsigned int)(sub_16C9C((unsigned __int16 *)(a3 + 136), 0) + 1) >> 1;
  *(_WORD *)(a2 + 40) = (unsigned int)sub_16C9C((unsigned __int16 *)(a3 + 136), 1) >> 1;
  v9 = 200;
  v10 = *(_DWORD *)(a3 + 176);
  *(_WORD *)(a2 + 44) = 0;
  if ( v10 != 8 )
    v9 = 100;
  result = v9 << 16;
  *(_WORD *)(a2 + 42) = HIWORD(result);
  return result;
}

