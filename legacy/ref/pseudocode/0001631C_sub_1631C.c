/* ea:   0x0001631C */
/* name: sub_1631C */
/* size: 260 */
char *sub_1631C(int *a1, _BYTE *a2, _BYTE *a3, int a4, ...)
{
  unsigned int v7; // r6
  va_list v8; // r0
  int v9; // r0
  size_t v10; // r9
  _BYTE *v11; // r8
  int v12; // r4
  _BYTE *v13; // r5
  _BYTE *v14; // r4
  unsigned int v16; // [sp+0h] [bp-3Ch] BYREF
  unsigned int v17; // [sp+3Ch] [bp+0h] BYREF
  va_list va; // [sp+3Ch] [bp+0h]
  int v19; // [sp+40h] [bp+4h]
  va_list va1; // [sp+44h] [bp+8h] BYREF

  va_start(va1, a4);
  va_start(va, a4);
  v17 = va_arg(va1, _DWORD);
  v19 = va_arg(va1, _DWORD);
  v7 = v17;
  v16 = a1[1] - *a1;
  va_copy(v8, va);
  if ( v16 >= v17 )
    v8 = (va_list)&v16;
  v9 = *(_DWORD *)v8;
  v10 = v9 + v16;
  if ( v9 + v16 != 0 )
  {
    v11 = malloc(v9 + v16);
    if ( v11 == nullptr )
      v11 = sub_115D8(v10);
  }
  else
  {
    v11 = nullptr;
  }
  if ( a2 == (_BYTE *)*a1 )
  {
    v13 = v11;
  }
  else
  {
    v12 = *a1;
    v13 = (char *)memmove(v11, (const void *)*a1, (size_t)&a2[-v12]) + (_DWORD)a2 - v12;
  }
  for ( ; v7 != 0; ++v13 )
  {
    --v7;
    *v13 = *a3;
  }
  if ( (_BYTE)v19 == 0 )
  {
    v14 = (_BYTE *)a1[1];
    if ( v14 != a2 )
      v13 = &v14[(_BYTE *)memmove(v13, a2, v14 - a2) - a2];
  }
  if ( *a1 != 0 )
    free((void *)*a1);
  *a1 = (int)v11;
  a1[1] = (int)v13;
  a1[2] = (int)&v11[v10];
  return &v11[v10];
}

