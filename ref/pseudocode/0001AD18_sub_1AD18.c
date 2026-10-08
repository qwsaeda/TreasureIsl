/* ea:   0x0001AD18 */
/* name: sub_1AD18 */
/* size: 592 */
int sub_1AD18(int *a1, char *a2, _DWORD *a3, int a4, ...)
{
  unsigned int v7; // r7
  __int64 v8; // r0
  unsigned int v9; // r2
  va_list v10; // r0
  unsigned int v11; // r10
  int v12; // r0
  char *v13; // r4
  int v14; // r9
  _DWORD *i; // r5
  int v16; // r1
  int v17; // r2
  int v18; // r3
  int v19; // r1
  int v20; // r2
  int v21; // r3
  int v22; // r1
  int v23; // r2
  int v24; // r3
  int v25; // r1
  int v26; // r2
  int v27; // r3
  _DWORD *j; // r4
  int v29; // r1
  int v30; // r2
  int v31; // r3
  int v32; // r1
  int v33; // r2
  int v34; // r3
  char *v35; // r7
  _DWORD *v36; // r6
  int v37; // r5
  int v38; // r1
  int v39; // r2
  int v40; // r3
  int v41; // r1
  int v42; // r2
  int v43; // r3
  unsigned int v45; // [sp+0h] [bp-3Ch] BYREF
  unsigned int v46; // [sp+3Ch] [bp+0h] BYREF
  va_list va; // [sp+3Ch] [bp+0h]
  int v48; // [sp+40h] [bp+4h]
  va_list va1; // [sp+44h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v46 = va_arg(va1, _DWORD);
  v48 = va_arg(va1, _DWORD);
  v7 = v46;
  v8 = 954437177LL * (a1[1] - *a1);
  v9 = (SHIDWORD(v8) >> 3) + (HIDWORD(v8) >> 31);
  v45 = v9;
  va_copy(v10, va);
  if ( v9 >= v46 )
    v10 = (va_list)&v45;
  v11 = v9 + *(_DWORD *)v10;
  v12 = sub_1B034(a1 + 2, v11);
  v13 = (char *)*a1;
  v14 = v12;
  for ( i = (_DWORD *)v12; v13 != a2; i += 9 )
  {
    if ( i != nullptr )
    {
      v16 = *((_DWORD *)v13 + 1);
      v17 = *((_DWORD *)v13 + 2);
      v18 = *((_DWORD *)v13 + 3);
      *i = *(_DWORD *)v13;
      i[1] = v16;
      i[2] = v17;
      i[3] = v18;
      v19 = *((_DWORD *)v13 + 5);
      v20 = *((_DWORD *)v13 + 6);
      v21 = *((_DWORD *)v13 + 7);
      i[4] = *((_DWORD *)v13 + 4);
      i[5] = v19;
      i[6] = v20;
      i[7] = v21;
      i[8] = *((_DWORD *)v13 + 8);
    }
    v13 += 36;
  }
  if ( v7 == 1 )
  {
    if ( i != nullptr )
    {
      v22 = a3[1];
      v23 = a3[2];
      v24 = a3[3];
      *i = *a3;
      i[1] = v22;
      i[2] = v23;
      i[3] = v24;
      v25 = a3[5];
      v26 = a3[6];
      v27 = a3[7];
      i[4] = a3[4];
      i[5] = v25;
      i[6] = v26;
      i[7] = v27;
      i[8] = a3[8];
    }
    j = i + 9;
  }
  else
  {
    for ( j = i; v7 != 0; j += 9 )
    {
      if ( j != nullptr )
      {
        v29 = a3[1];
        v30 = a3[2];
        v31 = a3[3];
        *j = *a3;
        j[1] = v29;
        j[2] = v30;
        j[3] = v31;
        v32 = a3[5];
        v33 = a3[6];
        v34 = a3[7];
        j[4] = a3[4];
        j[5] = v32;
        j[6] = v33;
        j[7] = v34;
        j[8] = a3[8];
      }
      --v7;
    }
  }
  if ( (_BYTE)v48 == 0 )
  {
    v35 = (char *)a1[1];
    v36 = j;
    if ( a2 != v35 )
    {
      v37 = a2 - (char *)j;
      do
      {
        if ( v36 != nullptr )
        {
          v38 = *(_DWORD *)((char *)v36 + v37 + 4);
          v39 = *(_DWORD *)((char *)v36 + v37 + 8);
          v40 = *(_DWORD *)((char *)v36 + v37 + 12);
          *v36 = *(_DWORD *)((char *)v36 + v37);
          v36[1] = v38;
          v36[2] = v39;
          v36[3] = v40;
          v41 = *(_DWORD *)((char *)v36 + v37 + 20);
          v42 = *(_DWORD *)((char *)v36 + v37 + 24);
          v43 = *(_DWORD *)((char *)v36 + v37 + 28);
          v36[4] = *(_DWORD *)((char *)v36 + v37 + 16);
          v36[5] = v41;
          v36[6] = v42;
          v36[7] = v43;
          v36[8] = *(_DWORD *)((char *)v36 + v37 + 32);
        }
        v36 += 9;
      }
      while ( (char *)v36 + v37 != v35 );
    }
    j = v36;
  }
  sub_1B020(a1);
  *a1 = v14;
  a1[1] = (int)j;
  a1[2] = v14 + 36 * v11;
  return 9 * v11;
}

