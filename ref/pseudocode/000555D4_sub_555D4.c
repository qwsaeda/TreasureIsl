/* ea:   0x000555D4 */
/* name: sub_555D4 */
/* size: 340 */
_DWORD *__fastcall sub_555D4(int a1, int a2, int a3)
{
  _DWORD *v5; // r0
  __int16 v6; // r1
  int *v7; // r5
  int *j; // r6
  int v9; // r1
  int v10; // t1
  int *v11; // r5
  int *v12; // r6
  int v13; // r1
  int v14; // t1
  _DWORD *result; // r0
  int *v16; // r5
  int *i; // r6
  int v18; // r1
  int v19; // t1
  int v20; // [sp+0h] [bp-24h] BYREF
  int v21; // [sp+4h] [bp-20h]

  *(_DWORD *)(a1 + 1024) = 0;
  v5 = (_DWORD *)(a1 + 1024);
  v5[1] = 0;
  v5[2] = 0;
  v5[3] = 0;
  v5[4] = 0;
  v5[5] = 0;
  v5[6] = 0;
  v5[7] = 0;
  *(_DWORD *)(a1 + 1024) = *(_DWORD *)a2;
  v6 = *(_WORD *)(a2 + 28);
  *(_DWORD *)(a1 + 1060) = a3;
  *(_WORD *)(a1 + 1052) = v6;
  if ( a3 != 0 )
  {
    v16 = *(int **)(a2 + 4);
    for ( i = *(int **)(a2 + 8); v16 != i; ++*(_DWORD *)(a1 + 1064) )
    {
      v18 = v16[1];
      v19 = *v16;
      v16 += 2;
      v21 = v18;
      v20 = v19;
      sub_4DCD8(a1 + 1028, &v20);
    }
    result = sub_445F4(a1, 444);
    result[5] = 0;
  }
  else
  {
    v7 = *(int **)(a2 + 16);
    for ( j = *(int **)(a2 + 20); v7 != j; ++*(_DWORD *)(a1 + 1064) )
    {
      v9 = v7[1];
      v10 = *v7;
      v7 += 2;
      v21 = v9;
      v20 = v10;
      sub_4DCD8(a1 + 1040, &v20);
    }
    v11 = *(int **)(a2 + 4);
    v12 = *(int **)(a2 + 8);
    while ( v11 != v12 )
    {
      v13 = v11[1];
      v14 = *v11;
      v11 += 2;
      v21 = v13;
      v20 = v14;
      sub_4DCD8(a1 + 1028, &v20);
    }
    result = sub_445F4(a1, 555);
    result[5] = 0;
  }
  return result;
}

