/* ea:   0x0004BB34 */
/* name: sub_4BB34 */
/* size: 384 */
int __fastcall sub_4BB34(int a1, int a2, _WORD *a3)
{
  int v6; // r0
  int v7; // r0
  int v8; // r4
  int v9; // r0
  int v10; // r0
  bool v11; // zf
  __int16 v12; // r0
  __int16 v13; // r1
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
  *(_DWORD *)(a2 + 4) = 0;
  *(_DWORD *)(a2 + 20) = *(_DWORD *)(a1 + 8);
  *(_DWORD *)(a2 + 12) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 120);
  v7 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 4 * *(_DWORD *)(a1 + 8) + 80);
  *(_DWORD *)(a2 + 28) = 0;
  *(_DWORD *)(a2 + 24) = v7;
  *(_DWORD *)(a2 + 8) = 0;
  *(_DWORD *)(a2 + 16) = *(_DWORD *)(a1 + 36);
  v8 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v9 = rand();
  *(_WORD *)(a2 + 34) = 0;
  *(_WORD *)(a2 + 36) = 1;
  *(_WORD *)(a2 + 32) = (v9 % 30 + 1) * v8 / 0x64u;
  if ( (unsigned __int16)((v9 % 30 + 1) * v8 / 0x64u) == 0 )
    *(_WORD *)(a2 + 32) = 1;
  *(_WORD *)(a2 + 38) = (sub_16C9C((unsigned __int16 *)(a1 + 136), 0) + 4) / 5u;
  *(_WORD *)(a2 + 40) = sub_16C9C((unsigned __int16 *)(a1 + 136), 1) / 5u;
  v10 = *(_DWORD *)(a1 + 176);
  *(_WORD *)(a2 + 44) = 0;
  v11 = v10 == 8;
  v12 = 200;
  if ( !v11 )
    v12 = 100;
  v13 = v12;
  result = *(unsigned __int16 *)(a2 + 32);
  *(_WORD *)(a2 + 42) = v13;
  *a3 = result;
  return result;
}

