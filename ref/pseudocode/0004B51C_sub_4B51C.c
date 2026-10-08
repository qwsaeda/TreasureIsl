/* ea:   0x0004B51C */
/* name: sub_4B51C */
/* size: 536 */
int __fastcall sub_4B51C(int result, int a2, int a3)
{
  int v3; // r9
  __int16 v6; // r5
  int v7; // r0
  __int16 v8; // r8
  int v9; // r2
  int v10; // r0
  int v11; // r0
  __int16 v12; // r1
  int v13; // r7
  int v14; // r5
  unsigned int v15; // r6
  int v16; // r4
  _DWORD *v17; // r4
  int v18; // t1
  int v19; // r1
  unsigned __int16 v20; // r0
  int v21; // [sp+0h] [bp-28h] BYREF

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
    v8 = 0;
    *(_DWORD *)(a2 + 4) = *(_DWORD *)(v3 + 8);
    *(_DWORD *)(a2 + 20) = *(unsigned __int8 *)(a3 + 50);
    v9 = *(_DWORD *)(a2 + 20);
    *(_DWORD *)(a2 + 12) = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(v3 + 24) + 32) + 4 * *(_DWORD *)(v3 + 8) + 80);
    v10 = *(_DWORD *)(*(_DWORD *)(*(_DWORD *)(v3 + 24) + 32) + 4 * v9 + 80);
    *(_DWORD *)(a2 + 28) = 0;
    *(_DWORD *)(a2 + 24) = v10;
    *(_DWORD *)(a2 + 8) = *(_DWORD *)(v3 + 36);
    v11 = *(_DWORD *)(v3 + 24);
    v21 = *(unsigned __int8 *)(a3 + 50);
    *(_DWORD *)(a2 + 16) = *(_DWORD *)(*(_DWORD *)sub_1DD10((_DWORD *)(v11 + 496), &v21) + 36);
    *(_WORD *)(a2 + 32) = (unsigned int)(sub_16C9C((unsigned __int16 *)(v3 + 136), 0) + 1) >> 1;
    *(_WORD *)(a2 + 34) = (unsigned int)sub_16C9C((unsigned __int16 *)(v3 + 136), 1) >> 1;
    if ( *(_DWORD *)(v3 + 176) != 8 )
      v6 = 1;
    *(_WORD *)(a2 + 36) = v6;
    *(_WORD *)(a2 + 38) = *(_WORD *)(a3 + 22);
    v12 = *(_WORD *)(a3 + 24);
    *(_WORD *)(a2 + 42) = 0;
    *(_WORD *)(a2 + 40) = v12;
    *(_WORD *)(a2 + 44) = 0;
    if ( (unsigned int)*(unsigned __int8 *)(a3 + 49) - 1 <= 3 )
    {
      *(_WORD *)(a2 + 42) = 350;
      *(_WORD *)(a2 + 44) = 60;
    }
    v13 = 0;
    v14 = 0;
    v15 = 0;
    while ( 1 )
    {
      if ( v14 >= 2 )
      {
LABEL_20:
        v20 = *(_WORD *)(a2 + 44);
        *(_WORD *)(a2 + 42) += 50 * v8;
        *(_WORD *)(a2 + 44) = 50 * v13 + v20;
        return (50 * v13 + v20) << 16;
      }
      v16 = *(_DWORD *)(v3 + 20);
      v18 = *(_DWORD *)(v16 + 28);
      v17 = (_DWORD *)(v16 + 28);
      if ( v15 >= (v17[1] - v18) >> 2 )
        TerminateProcess((HANDLE)0x42, 0);
      v19 = *(_DWORD *)(*v17 + 4 * v15);
      if ( v19 == 4 )
        break;
      if ( v19 == 6 )
      {
        v8 = 1;
LABEL_18:
        ++v14;
      }
      if ( (int)++v15 >= 5 )
        goto LABEL_20;
    }
    v13 = 1;
    goto LABEL_18;
  }
  return result;
}

