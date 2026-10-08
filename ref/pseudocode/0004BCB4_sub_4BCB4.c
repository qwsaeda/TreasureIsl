/* ea:   0x0004BCB4 */
/* name: sub_4BCB4 */
/* size: 292 */
int __fastcall sub_4BCB4(int a1, int a2)
{
  __int16 v4; // r6
  int v5; // r0
  int v6; // r0
  int v7; // r4
  int v8; // r3
  int v9; // r0
  int v10; // r2
  int v11; // r0
  __int16 v12; // r2

  v4 = 2;
  if ( dword_7FB6C == 2 )
  {
    v5 = 3;
  }
  else
  {
    v5 = 1;
    if ( *(_DWORD *)(a1 + 40) != *(_DWORD *)(a1 + 44) )
      v5 = 2;
  }
  *(_DWORD *)a2 = v5;
  v6 = *(_DWORD *)(a1 + 8);
  *(_DWORD *)(a2 + 20) = 0;
  *(_DWORD *)(a2 + 4) = v6;
  *(_DWORD *)(a2 + 12) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(a1 + 24) + 32) + 4 * *(_DWORD *)(a1 + 8) + 80);
  v7 = *(_DWORD *)(a1 + 24);
  v8 = *(_DWORD *)(a1 + 8);
  v9 = *(_DWORD *)(v7 + 32);
  if ( *(_DWORD *)(*(_DWORD *)(v7 + 264) + 4 * v8 - 4) == 4 )
    v10 = *(_DWORD *)(v9 + 4 * v8 + 80);
  else
    v10 = *(_DWORD *)(v9 + 100);
  *(_DWORD *)(a2 + 28) = 0;
  *(_DWORD *)(a2 + 24) = v10;
  v11 = *(_DWORD *)(a1 + 36);
  *(_DWORD *)(a2 + 16) = 0;
  *(_DWORD *)(a2 + 8) = v11;
  *(_WORD *)(a2 + 32) = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  *(_WORD *)(a2 + 34) = sub_16C9C((unsigned __int16 *)(a1 + 136), 1);
  if ( *(_DWORD *)(a1 + 176) != 8 )
    v4 = 1;
  *(_WORD *)(a2 + 36) = v4;
  v12 = word_706FA[dword_7FB6C];
  *(_WORD *)(a2 + 40) = 0;
  *(_WORD *)(a2 + 38) = v12;
  *(_WORD *)(a2 + 42) = 500;
  *(_WORD *)(a2 + 44) = 80;
  return 500;
}

