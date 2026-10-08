/* ea:   0x0003AFD8 */
/* name: sub_3AFD8 */
/* size: 232 */
int sub_3AFD8(_DWORD *a1, unsigned int a2, ...)
{
  _WORD *v4; // r5
  va_list v5; // r0
  int v6; // r4
  unsigned int v7; // r3
  int v8; // r0
  unsigned int v10; // [sp+Ch] [bp-2Ch] BYREF
  unsigned int varg_r2; // [sp+30h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+30h] [bp-8h]
  _WORD *varg_r3; // [sp+34h] [bp-4h]
  va_list va1; // [sp+38h] [bp+0h] BYREF

  va_start(va1, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, _WORD *);
  v4 = varg_r3;
  if ( a2 > (a1[1] - *a1) >> 1 )
    TerminateProcess((HANDLE)0x42, 0);
  v10 = ((a1[1] - *a1) >> 1) - a2;
  v5 = (va_list)&v10;
  if ( v10 >= varg_r2 )
    va_copy(v5, varg_r2a);
  v6 = *(_DWORD *)v5;
  v7 = sub_12F04(varg_r3);
  if ( v7 > 0x7FFFFFFE || ((a1[1] - *a1) >> 1) - v6 >= 2147483646 - v7 )
    TerminateProcess((HANDLE)0x42, 0);
  v8 = sub_12F04(v4);
  return sub_3B368((int)a1, (void *)(*a1 + 2 * a2), (void *)(*a1 + 2 * (v6 + a2)), v4, (int)&v4[v8]);
}

