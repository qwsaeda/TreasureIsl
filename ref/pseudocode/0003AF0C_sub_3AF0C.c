/* ea:   0x0003AF0C */
/* name: sub_3AF0C */
/* size: 200 */
int sub_3AF0C(_DWORD *a1, unsigned int a2, ...)
{
  int *v4; // r6
  int v5; // r3
  int v6; // r1
  va_list v7; // r0
  int v8; // r5
  unsigned int v10; // [sp+Ch] [bp-30h] BYREF
  unsigned int varg_r2; // [sp+34h] [bp-8h] BYREF
  va_list varg_r2a; // [sp+34h] [bp-8h]
  int *varg_r3; // [sp+38h] [bp-4h]
  va_list va1; // [sp+3Ch] [bp+0h] BYREF

  va_start(va1, a2);
  va_start(varg_r2a, a2);
  varg_r2 = va_arg(va1, _DWORD);
  varg_r3 = va_arg(va1, int *);
  v4 = varg_r3;
  if ( a2 > (a1[1] - *a1) >> 1 )
    TerminateProcess((HANDLE)0x42, 0);
  v5 = (a1[1] - *a1) >> 1;
  v6 = *varg_r3;
  v10 = v5 - a2;
  v7 = (va_list)&v10;
  if ( v5 - a2 >= varg_r2 )
    va_copy(v7, varg_r2a);
  v8 = *(_DWORD *)v7;
  if ( v5 - *(_DWORD *)v7 >= (unsigned int)(2147483646 - ((varg_r3[1] - v6) >> 1)) )
    TerminateProcess((HANDLE)0x42, 0);
  return sub_3B2A8((int)a1, *a1 + 2 * a2, (void *)(*a1 + 2 * (v8 + a2)), *v4, v4[1]);
}

