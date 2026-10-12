/* ea:   0x000641E8 */
/* name: sub_641E8 */
/* size: 412 */
int sub_641E8(int *a1, char *a2, _DWORD *a3, int a4, ...)
{
  unsigned int v7; // r7
  va_list v8; // r0
  unsigned int v9; // r10
  int v10; // r0
  char *v11; // r4
  int v12; // r9
  _DWORD *i; // r5
  int v14; // r1
  int v15; // r2
  int v16; // r3
  int v17; // r1
  int v18; // r2
  int v19; // r3
  _DWORD *j; // r4
  int v21; // r1
  int v22; // r2
  int v23; // r3
  char *v24; // r6
  _DWORD *v25; // r5
  int v26; // r4
  int v27; // r2
  int v28; // r3
  int v29; // r0
  int result; // r0
  unsigned int v31; // [sp+0h] [bp-3Ch] BYREF
  unsigned int v32; // [sp+3Ch] [bp+0h] BYREF
  va_list va; // [sp+3Ch] [bp+0h]
  int v34; // [sp+40h] [bp+4h]
  va_list va1; // [sp+44h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v32 = va_arg(va1, _DWORD);
  v34 = va_arg(va1, _DWORD);
  v7 = v32;
  v31 = (a1[1] - *a1) >> 4;
  va_copy(v8, va);
  if ( v31 >= v32 )
    v8 = (va_list)&v31;
  v9 = v31 + *(_DWORD *)v8;
  v10 = sub_64400(a1 + 2, v9);
  v11 = (char *)*a1;
  v12 = v10;
  for ( i = (_DWORD *)v10; v11 != a2; i += 4 )
  {
    if ( i != nullptr )
    {
      v14 = *((_DWORD *)v11 + 1);
      v15 = *((_DWORD *)v11 + 2);
      v16 = *((_DWORD *)v11 + 3);
      *i = *(_DWORD *)v11;
      i[1] = v14;
      i[2] = v15;
      i[3] = v16;
    }
    v11 += 16;
  }
  if ( v7 == 1 )
  {
    if ( i != nullptr )
    {
      v17 = a3[1];
      v18 = a3[2];
      v19 = a3[3];
      *i = *a3;
      i[1] = v17;
      i[2] = v18;
      i[3] = v19;
    }
    j = i + 4;
  }
  else
  {
    for ( j = i; v7 != 0; j += 4 )
    {
      if ( j != nullptr )
      {
        v21 = a3[1];
        v22 = a3[2];
        v23 = a3[3];
        *j = *a3;
        j[1] = v21;
        j[2] = v22;
        j[3] = v23;
      }
      --v7;
    }
  }
  if ( (_BYTE)v34 == 0 )
  {
    v24 = (char *)a1[1];
    v25 = j;
    if ( a2 != v24 )
    {
      v26 = a2 - (char *)j;
      do
      {
        if ( v25 != nullptr )
        {
          v27 = *(_DWORD *)((char *)v25 + v26 + 4);
          v28 = *(_DWORD *)((char *)v25 + v26 + 8);
          v29 = *(_DWORD *)((char *)v25 + v26 + 12);
          *v25 = *(_DWORD *)((char *)v25 + v26);
          v25[1] = v27;
          v25[2] = v28;
          v25[3] = v29;
        }
        v25 += 4;
      }
      while ( (char *)v25 + v26 != v24 );
    }
    j = v25;
  }
  result = sub_643EC(a1);
  *a1 = v12;
  a1[1] = (int)j;
  a1[2] = v12 + 16 * v9;
  return result;
}

