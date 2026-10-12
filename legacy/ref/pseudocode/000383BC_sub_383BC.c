/* ea:   0x000383BC */
/* name: sub_383BC */
/* size: 400 */
int sub_383BC(int *a1, char *a2, _DWORD *a3, int a4, ...)
{
  unsigned int v7; // r5
  __int64 v8; // r0
  unsigned int v9; // r2
  va_list v10; // r0
  unsigned int v11; // r10
  int v12; // r0
  char *v13; // r3
  int v14; // r9
  _DWORD *i; // r4
  int v16; // r1
  int v17; // r2
  int v18; // r1
  int v19; // r2
  _DWORD *j; // r7
  int v21; // r1
  int v22; // r2
  char *v23; // r6
  _DWORD *v24; // r5
  int v25; // r4
  int v26; // r2
  int v27; // r3
  unsigned int v29; // [sp+0h] [bp-3Ch] BYREF
  unsigned int v30; // [sp+3Ch] [bp+0h] BYREF
  va_list va; // [sp+3Ch] [bp+0h]
  int v32; // [sp+40h] [bp+4h]
  va_list va1; // [sp+44h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v30 = va_arg(va1, _DWORD);
  v32 = va_arg(va1, _DWORD);
  v7 = v30;
  v8 = 715827883LL * (a1[1] - *a1);
  v9 = (SHIDWORD(v8) >> 1) + (HIDWORD(v8) >> 31);
  v29 = v9;
  va_copy(v10, va);
  if ( v9 >= v30 )
    v10 = (va_list)&v29;
  v11 = v9 + *(_DWORD *)v10;
  v12 = sub_3870C(a1 + 2, v11);
  v13 = (char *)*a1;
  v14 = v12;
  for ( i = (_DWORD *)v12; v13 != a2; i += 3 )
  {
    if ( i != nullptr )
    {
      v16 = *((_DWORD *)v13 + 1);
      v17 = *((_DWORD *)v13 + 2);
      *i = *(_DWORD *)v13;
      i[1] = v16;
      i[2] = v17;
    }
    v13 += 12;
  }
  if ( v7 == 1 )
  {
    if ( i != nullptr )
    {
      v18 = a3[1];
      v19 = a3[2];
      *i = *a3;
      i[1] = v18;
      i[2] = v19;
    }
    j = i + 3;
  }
  else
  {
    for ( j = i; v7 != 0; j += 3 )
    {
      if ( j != nullptr )
      {
        v21 = a3[1];
        v22 = a3[2];
        *j = *a3;
        j[1] = v21;
        j[2] = v22;
      }
      --v7;
    }
  }
  if ( (_BYTE)v32 == 0 )
  {
    v23 = (char *)a1[1];
    v24 = j;
    if ( a2 != v23 )
    {
      v25 = a2 - (char *)j;
      do
      {
        if ( v24 != nullptr )
        {
          v26 = *(_DWORD *)((char *)v24 + v25 + 4);
          v27 = *(_DWORD *)((char *)v24 + v25 + 8);
          *v24 = *(_DWORD *)((char *)v24 + v25);
          v24[1] = v26;
          v24[2] = v27;
        }
        v24 += 3;
      }
      while ( (char *)v24 + v25 != v23 );
    }
    j = v24;
  }
  sub_386F8(a1);
  *a1 = v14;
  a1[1] = (int)j;
  a1[2] = v14 + 12 * v11;
  return 3 * v11;
}

