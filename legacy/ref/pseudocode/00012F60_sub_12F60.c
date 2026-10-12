/* ea:   0x00012F60 */
/* name: sub_12F60 */
/* size: 296 */
int sub_12F60(int **a1, int *a2, int *a3, int a4, ...)
{
  unsigned int v7; // r5
  va_list v8; // r0
  unsigned int v9; // r9
  int v10; // r0
  int *v11; // r1
  int v12; // r8
  int *i; // r2
  int *j; // r4
  int *v15; // r2
  int *v16; // r3
  int v17; // r1
  int result; // r0
  unsigned int v19; // [sp+0h] [bp-38h] BYREF
  unsigned int v20; // [sp+38h] [bp+0h] BYREF
  va_list va; // [sp+38h] [bp+0h]
  int v22; // [sp+3Ch] [bp+4h]
  va_list va1; // [sp+40h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v20 = va_arg(va1, _DWORD);
  v22 = va_arg(va1, _DWORD);
  v7 = v20;
  v19 = a1[1] - *a1;
  va_copy(v8, va);
  if ( v19 >= v20 )
    v8 = (va_list)&v19;
  v9 = v19 + *(_DWORD *)v8;
  v10 = sub_132C8(a1 + 2, v9);
  v11 = *a1;
  v12 = v10;
  for ( i = (int *)v10; v11 != a2; ++i )
  {
    if ( i != nullptr )
      v10 = *v11;
    ++v11;
    if ( i != nullptr )
      *i = v10;
  }
  if ( v7 == 1 )
  {
    if ( i != nullptr )
      v10 = *a3;
    j = i + 1;
    if ( i != nullptr )
      *i = v10;
  }
  else
  {
    for ( j = i; v7 != 0; ++j )
    {
      if ( j != nullptr )
        v10 = *a3;
      --v7;
      if ( j != nullptr )
        *j = v10;
    }
  }
  if ( (_BYTE)v22 == 0 )
  {
    v15 = a1[1];
    v16 = j;
    if ( a2 != v15 )
    {
      v17 = (char *)a2 - (char *)j;
      do
      {
        if ( v16 != nullptr )
          *v16 = *(int *)((char *)v16 + v17);
        ++v16;
      }
      while ( (int *)((char *)v16 + v17) != v15 );
    }
    j = v16;
  }
  result = sub_132B4(a1);
  *a1 = (int *)v12;
  a1[1] = j;
  a1[2] = (int *)(v12 + 4 * v9);
  return result;
}

