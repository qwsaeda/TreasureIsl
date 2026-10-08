/* ea:   0x00054114 */
/* name: sub_54114 */
/* size: 296 */
void sub_54114(int a1, char *a2, _DWORD *a3, int a4, ...)
{
  unsigned int v7; // r5
  va_list v8; // r0
  int v9; // r9
  _DWORD *v10; // r0
  char *v11; // r1
  _DWORD *v12; // r8
  _DWORD *i; // r2
  _DWORD *j; // r4
  char *v15; // r2
  _DWORD *v16; // r3
  int v17; // r1
  unsigned int v18; // [sp+0h] [bp-38h] BYREF
  unsigned int v19; // [sp+38h] [bp+0h] BYREF
  va_list va; // [sp+38h] [bp+0h]
  int v21; // [sp+3Ch] [bp+4h]
  va_list va1; // [sp+40h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v19 = va_arg(va1, _DWORD);
  v21 = va_arg(va1, _DWORD);
  v7 = v19;
  v18 = (*(_DWORD *)(a1 + 4) - *(_DWORD *)a1) >> 2;
  va_copy(v8, va);
  if ( v18 >= v19 )
    v8 = (va_list)&v18;
  v9 = v18 + *(_DWORD *)v8;
  v10 = sub_4A7A8(a1 + 8, v9);
  v11 = *(char **)a1;
  v12 = v10;
  for ( i = v10; v11 != a2; ++i )
  {
    if ( i != nullptr )
      v10 = *(_DWORD **)v11;
    v11 += 4;
    if ( i != nullptr )
      *i = v10;
  }
  if ( v7 == 1 )
  {
    if ( i != nullptr )
      v10 = (_DWORD *)*a3;
    j = i + 1;
    if ( i != nullptr )
      *i = v10;
  }
  else
  {
    for ( j = i; v7 != 0; ++j )
    {
      if ( j != nullptr )
        v10 = (_DWORD *)*a3;
      --v7;
      if ( j != nullptr )
        *j = v10;
    }
  }
  if ( (_BYTE)v21 == 0 )
  {
    v15 = *(char **)(a1 + 4);
    v16 = j;
    if ( a2 != v15 )
    {
      v17 = a2 - (char *)j;
      do
      {
        if ( v16 != nullptr )
          *v16 = *(_DWORD *)((char *)v16 + v17);
        ++v16;
      }
      while ( (char *)v16 + v17 != v15 );
    }
    j = v16;
  }
  sub_4A794((void **)a1);
  *(_DWORD *)a1 = v12;
  *(_DWORD *)(a1 + 4) = j;
  *(_DWORD *)(a1 + 8) = &v12[v9];
}

