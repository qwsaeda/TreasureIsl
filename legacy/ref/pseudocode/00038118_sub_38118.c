/* ea:   0x00038118 */
/* name: sub_38118 */
/* size: 328 */
int sub_38118(int *a1, char *a2, _DWORD *a3, int a4, ...)
{
  unsigned int v7; // r6
  __int64 v8; // r0
  unsigned int v9; // r2
  va_list v10; // r0
  unsigned int v11; // r10
  int v12; // r0
  char *v13; // r4
  int v14; // r8
  _DWORD *i; // r5
  _DWORD *j; // r4
  char *v17; // r6
  _DWORD *v18; // r5
  int v19; // r4
  unsigned int v21; // [sp+0h] [bp-3Ch] BYREF
  unsigned int v22; // [sp+3Ch] [bp+0h] BYREF
  va_list va; // [sp+3Ch] [bp+0h]
  int v24; // [sp+40h] [bp+4h]
  va_list va1; // [sp+44h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v22 = va_arg(va1, _DWORD);
  v24 = va_arg(va1, _DWORD);
  v7 = v22;
  v8 = 715827883LL * (a1[1] - *a1);
  v9 = (SHIDWORD(v8) >> 1) + (HIDWORD(v8) >> 31);
  v21 = v9;
  va_copy(v10, va);
  if ( v9 >= v22 )
    v10 = (va_list)&v21;
  v11 = v9 + *(_DWORD *)v10;
  v12 = sub_38678(a1 + 2, v11);
  v13 = (char *)*a1;
  v14 = v12;
  for ( i = (_DWORD *)v12; v13 != a2; i += 3 )
  {
    if ( i != nullptr )
      sub_132FC(i, v13);
    v13 += 12;
  }
  if ( v7 == 1 )
  {
    sub_399D0(i, a3);
    j = i + 3;
  }
  else
  {
    for ( j = i; v7 != 0; j += 3 )
    {
      if ( j != nullptr )
        sub_132FC(j, a3);
      --v7;
    }
  }
  if ( (_BYTE)v24 == 0 )
  {
    v17 = (char *)a1[1];
    v18 = j;
    if ( a2 != v17 )
    {
      v19 = a2 - (char *)j;
      do
      {
        if ( v18 != nullptr )
          sub_132FC(v18, (_DWORD *)((char *)v18 + v19));
        v18 += 3;
      }
      while ( (char *)v18 + v19 != v17 );
    }
    j = v18;
  }
  sub_3863C(a1);
  *a1 = v14;
  a1[1] = (int)j;
  a1[2] = v14 + 12 * v11;
  return 3 * v11;
}

