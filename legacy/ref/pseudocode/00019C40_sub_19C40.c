/* ea:   0x00019C40 */
/* name: sub_19C40 */
/* size: 348 */
int sub_19C40(int *a1, char *a2, _DWORD *a3, int a4, ...)
{
  unsigned int v7; // r5
  va_list v8; // r0
  unsigned int v9; // r9
  int v10; // r0
  char *v11; // r2
  int v12; // r8
  _DWORD *i; // r3
  int v14; // r1
  int v15; // r1
  _DWORD *j; // r6
  int v17; // r1
  char *v18; // r5
  _DWORD *v19; // r4
  int v20; // r3
  int v21; // r2
  int result; // r0
  unsigned int v23; // [sp+0h] [bp-38h] BYREF
  unsigned int v24; // [sp+38h] [bp+0h] BYREF
  va_list va; // [sp+38h] [bp+0h]
  int v26; // [sp+3Ch] [bp+4h]
  va_list va1; // [sp+40h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v24 = va_arg(va1, _DWORD);
  v26 = va_arg(va1, _DWORD);
  v7 = v24;
  v23 = (a1[1] - *a1) >> 3;
  va_copy(v8, va);
  if ( v23 >= v24 )
    v8 = (va_list)&v23;
  v9 = v23 + *(_DWORD *)v8;
  v10 = sub_19DB0(a1 + 2, v9);
  v11 = (char *)*a1;
  v12 = v10;
  for ( i = (_DWORD *)v10; v11 != a2; i += 2 )
  {
    if ( i != nullptr )
    {
      v14 = *((_DWORD *)v11 + 1);
      *i = *(_DWORD *)v11;
      i[1] = v14;
    }
    v11 += 8;
  }
  if ( v7 == 1 )
  {
    if ( i != nullptr )
    {
      v15 = a3[1];
      *i = *a3;
      i[1] = v15;
    }
    j = i + 2;
  }
  else
  {
    for ( j = i; v7 != 0; j += 2 )
    {
      if ( j != nullptr )
      {
        v17 = a3[1];
        *j = *a3;
        j[1] = v17;
      }
      --v7;
    }
  }
  if ( (_BYTE)v26 == 0 )
  {
    v18 = (char *)a1[1];
    v19 = j;
    if ( a2 != v18 )
    {
      v20 = a2 - (char *)j;
      do
      {
        if ( v19 != nullptr )
        {
          v21 = *(_DWORD *)((char *)v19 + v20 + 4);
          *v19 = *(_DWORD *)((char *)v19 + v20);
          v19[1] = v21;
        }
        v19 += 2;
      }
      while ( (char *)v19 + v20 != v18 );
    }
    j = v19;
  }
  result = sub_19D9C(a1);
  *a1 = v12;
  a1[1] = (int)j;
  a1[2] = v12 + 8 * v9;
  return result;
}

