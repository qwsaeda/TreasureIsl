/* ea:   0x0004B9EC */
/* name: sub_4B9EC */
/* size: 328 */
int __fastcall sub_4B9EC(int result, int a2, int a3)
{
  int v3; // r5
  __int16 v6; // r6
  int v7; // r0
  int v8; // r2
  int v9; // r0
  int v10; // r0
  __int16 v11; // r1
  int v12; // [sp+0h] [bp-20h] BYREF

  v3 = result;
  if ( a3 != 0 )
  {
    v6 = 2;
    if ( dword_7FB6C == 2 )
    {
      v7 = 3;
    }
    else
    {
      v7 = 1;
      if ( *(_DWORD *)(v3 + 40) != *(_DWORD *)(v3 + 44) )
        v7 = 2;
    }
    *(_DWORD *)a2 = v7;
    *(_DWORD *)(a2 + 4) = *(_DWORD *)(v3 + 8);
    *(_DWORD *)(a2 + 20) = *(_DWORD *)(a3 + 20);
    v8 = *(_DWORD *)(a2 + 20);
    *(_DWORD *)(a2 + 12) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(v3 + 24) + 32) + 4 * *(_DWORD *)(v3 + 8) + 80);
    v9 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(v3 + 24) + 32) + 4 * v8 + 80);
    *(_DWORD *)(a2 + 28) = 0;
    *(_DWORD *)(a2 + 24) = v9;
    *(_DWORD *)(a2 + 8) = *(_DWORD *)(v3 + 36);
    v10 = *(_DWORD *)(v3 + 24);
    v12 = *(_DWORD *)(a3 + 20);
    *(_DWORD *)(a2 + 16) = *(_DWORD *)(*(_DWORD *)sub_1DD10((_DWORD *)(v10 + 496), &v12) + 36);
    *(_WORD *)(a2 + 32) = (unsigned int)(sub_16C9C((unsigned __int16 *)(v3 + 136), 0) + 1) >> 1;
    *(_WORD *)(a2 + 34) = (unsigned int)sub_16C9C((unsigned __int16 *)(v3 + 136), 1) >> 1;
    if ( *(_DWORD *)(v3 + 176) != 8 )
      v6 = 1;
    *(_WORD *)(a2 + 36) = v6;
    *(_WORD *)(a2 + 38) = *(_WORD *)(a3 + 12);
    v11 = *(_WORD *)(a3 + 14);
    *(_WORD *)(a2 + 42) = 500;
    *(_WORD *)(a2 + 40) = v11;
    *(_WORD *)(a2 + 44) = 80;
    return 500;
  }
  return result;
}

