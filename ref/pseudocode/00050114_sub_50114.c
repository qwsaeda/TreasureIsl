/* ea:   0x00050114 */
/* name: sub_50114 */
/* size: 240 */
char *sub_50114(int *a1, _BYTE *a2, _DWORD *a3, int a4, ...)
{
  unsigned int v7; // r5
  va_list v8; // r0
  unsigned int v9; // r9
  char *v10; // r0
  int v11; // r4
  char *v12; // r8
  char *v13; // r7
  _BYTE *v14; // r4
  char *result; // r0
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
  v16 = (a1[1] - *a1) >> 2;
  va_copy(v8, va);
  if ( v16 >= v17 )
    v8 = (va_list)&v16;
  v9 = *(_DWORD *)v8 + v16;
  v10 = (char *)sub_5022C(a1 + 2, v9);
  v12 = v10;
  if ( a2 == (_BYTE *)*a1 )
  {
    v13 = v10;
  }
  else
  {
    v11 = *a1;
    v13 = (char *)memmove(v10, (const void *)*a1, (size_t)&a2[-v11]) + (_DWORD)a2 - v11;
  }
  for ( ; v7 != 0; v13 += 4 )
  {
    --v7;
    *(_DWORD *)v13 = *a3;
  }
  if ( (_BYTE)v19 == 0 )
  {
    v14 = (_BYTE *)a1[1];
    if ( v14 != a2 )
      v13 = &v14[(_BYTE *)memmove(v13, a2, v14 - a2) - a2];
  }
  if ( *a1 != 0 )
    free((void *)*a1);
  *a1 = (int)v12;
  result = &v12[4 * v9];
  a1[1] = (int)v13;
  a1[2] = (int)result;
  return result;
}

