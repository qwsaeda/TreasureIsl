/* ea:   0x00066118 */
/* name: sub_66118 */
/* size: 176 */
_WORD *sub_66118(int *a1, ...)
{
  va_list v2; // r0
  int v3; // r6
  _WORD *v4; // r5
  _WORD *v5; // r4
  _WORD *result; // r0
  unsigned int v7; // [sp+4h] [bp-2Ch] BYREF
  unsigned int varg_r1; // [sp+24h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+24h] [bp-Ch]
  va_list varg_r2; // [sp+28h] [bp-8h] BYREF

  va_start(varg_r2, a1);
  va_start(varg_r1a, a1);
  varg_r1 = va_arg(varg_r2, _DWORD);
  if ( varg_r1 > 0x7FFFFFFE )
    TerminateProcess((HANDLE)0x42, 0);
  v7 = (a1[1] - *a1) >> 1;
  v2 = (va_list)&v7;
  if ( varg_r1 >= v7 )
    va_copy(v2, varg_r1a);
  v3 = *(_DWORD *)v2 + 1;
  v4 = sub_1343C((int)(a1 + 2), v3);
  v5 = sub_13524(*a1, a1[1], v4);
  *v5 = 0;
  if ( *a1 != 0 )
    free((void *)*a1);
  *a1 = (int)v4;
  result = &v4[v3];
  a1[1] = (int)v5;
  a1[2] = (int)result;
  return result;
}

