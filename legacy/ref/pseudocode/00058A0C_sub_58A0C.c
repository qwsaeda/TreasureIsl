/* ea:   0x00058A0C */
/* name: sub_58A0C */
/* size: 780 */
int __fastcall sub_58A0C(int a1, void **a2, int *a3, char a4)
{
  char *v5; // r10
  unsigned int v6; // r0
  _BYTE *v7; // r11
  unsigned __int16 *v9; // lr
  _WORD *v10; // r8
  unsigned __int8 *v11; // r9
  BOOL v12; // r7
  unsigned __int8 *v13; // r10
  char v14; // t1
  unsigned __int8 v15; // t1
  __int16 v16; // r1
  unsigned __int8 *v17; // r10
  __int16 v18; // t1
  __int16 v19; // r1
  __int16 v20; // t1
  __int16 v21; // r1
  unsigned __int8 *v22; // r10
  __int16 v23; // t1
  __int16 v24; // r1
  __int16 v25; // t1
  __int16 v26; // r1
  __int16 v27; // t1
  __int16 v28; // r1
  __int16 v29; // t1
  _BYTE *v30; // r2
  __int16 *v31; // r5
  __int16 *v32; // r6
  int v33; // r0
  unsigned int v34; // r4
  char v35; // t1
  __int16 v36; // r0
  _BYTE *v37; // r11
  int v39; // [sp+4h] [bp-30h]
  void *v41; // [sp+Ch] [bp-28h]

  v5 = (char *)*a2;
  v41 = *a2;
  v39 = *a3;
  v6 = 2 * *a3 / *(unsigned __int16 *)(a1 + 52) * *(unsigned __int16 *)(a1 + 56) * *(unsigned __int16 *)(a1 + 42);
  *a3 = v6;
  v7 = operator new(v6);
  *a2 = v7;
  if ( v7 == nullptr )
    return -1;
  v9 = (unsigned __int16 *)a1;
  v10 = (_WORD *)(a1 + 88);
  v11 = (unsigned __int8 *)(a1 + 8 * ((*(_WORD *)(a1 + 42) == 2) + 11));
  if ( v39 >= *(unsigned __int16 *)(a1 + 52) )
  {
    v12 = *(unsigned __int16 *)(a1 + 42) == 2;
    do
    {
      v14 = *v5;
      v13 = (unsigned __int8 *)(v5 + 1);
      *(_BYTE *)v10 = v14;
      if ( v12 )
      {
        v15 = *v13++;
        *v11 = v15;
      }
      v16 = v13[1];
      v18 = *v13;
      v17 = v13 + 2;
      *(_WORD *)(a1 + 90) = v18 | (v16 << 8);
      if ( v12 )
      {
        v19 = v17[1];
        v20 = *v17;
        v17 += 2;
        *((_WORD *)v11 + 1) = v20 | (v19 << 8);
      }
      v21 = v17[1];
      v23 = *v17;
      v22 = v17 + 2;
      *(_WORD *)(a1 + 92) = v23 | (v21 << 8);
      if ( v12 )
      {
        v24 = v22[1];
        v25 = *v22;
        v22 += 2;
        *((_WORD *)v11 + 2) = v25 | (v24 << 8);
      }
      v26 = v22[1];
      v27 = *v22;
      v5 = (char *)(v22 + 2);
      *(_WORD *)(a1 + 94) = v27 | (v26 << 8);
      if ( v12 )
      {
        v28 = (unsigned __int8)v5[1];
        v29 = (unsigned __int8)*v5;
        v5 += 2;
        *((_WORD *)v11 + 3) = v29 | (v28 << 8);
      }
      v30 = v7 + 2;
      v31 = (__int16 *)&v9[2 * *(unsigned __int8 *)v10 + 30];
      v32 = (__int16 *)&v9[2 * *v11 + 30];
      *v7 = *(_WORD *)(a1 + 94);
      v7[1] = *(_BYTE *)(a1 + 95);
      if ( v12 )
      {
        *v30 = *((_WORD *)v11 + 3);
        v7[3] = v11[7];
        v30 = v7 + 4;
      }
      v7 = v30 + 2;
      *v30 = *(_WORD *)(a1 + 92);
      v30[1] = *(_BYTE *)(a1 + 93);
      if ( v12 )
      {
        *v7 = *((_WORD *)v11 + 2);
        v30[3] = v11[5];
        v7 = v30 + 4;
      }
      v33 = (v9[28] - 2) * v9[21];
      if ( v33 > 0 )
      {
        v34 = ((unsigned int)(v33 - 1) >> 1) + 1;
        do
        {
          *(_WORD *)v7 = sub_588E4((int)v9, v10, (unsigned __int8)*v5 >> 4, v31);
          v35 = *v5++;
          v36 = sub_588E4(a1, v11, v35 & 0xF, v32);
          v9 = (unsigned __int16 *)a1;
          v7[2] = v36;
          v37 = v7 + 2;
          --v34;
          v37[1] = HIBYTE(v36);
          v7 = v37 + 2;
        }
        while ( v34 != 0 );
      }
      v39 -= v9[26];
    }
    while ( v39 >= v9[26] );
  }
  if ( a4 != 0 )
    operator delete(v41);
  return 0;
}

